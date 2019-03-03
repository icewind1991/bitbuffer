//! Automatically generate `BitRead` and `BitReadSized` implementations for structs and enums
//!
//! # Structs
//!
//! The implementation can be derived for a struct as long as every field in the struct implements `BitRead` or `BitReadSized`
//!
//! The struct is read field by field in the order they are defined in, if the size for a field is set `stream.read_sized()`
//! will be used, otherwise `stream_read()` will be used.
//!
//! The size for a field can be set using 3 different methods
//!  - set the size as an integer using the `size` attribute,
//!  - use a previously defined field as the size using the `size` attribute
//!  - read a set number of bits as an integer, using the resulting value as size using the `read_bits` attribute
//!
//! When deriving `BitReadSized` the input size can be used in the size attribute as the `input_size` field.
//!
//! ## Examples
//!
//! ```
//! use bitstream_reader_derive::BitRead;
//!
//! #[derive(BitRead)]
//! struct TestStruct {
//!     foo: u8,
//!     str: String,
//!     #[size = 2] // when `size` is set, the attributed will be read using `read_sized`
//!     truncated: String,
//!     bar: u16,
//!     float: f32,
//!     #[size = 3]
//!     asd: u8,
//!     #[size_bits = 2] // first read 2 bits as unsigned integer, then use the resulting value as size for the read
//!     dynamic_length: u8,
//!     #[size = "asd"] // use a previously defined field as size
//!     previous_field: u8,
//! }
//! ```
//!
//! ```
//! use bitstream_reader_derive::BitReadSized;
//!
//! #[derive(BitReadSized, PartialEq, Debug)]
//! struct TestStructSized {
//!     foo: u8,
//!     #[size = "input_size"]
//!     string: String,
//!     #[size = "input_size"]
//!     int: u8,
//! }
//! ```
//!
//! # Enums
//!
//! The implementation can be derived for an enum as long as every variant of the enum either has no field, or an unnamed field that implements `BitRead` or `BitReadSized`
//!
//! The enum is read by first reading a set number of bits as the discriminant of the enum, then the variant for the read discriminant is read.
//!
//! For details about setting the input size for fields implementing `BitReadSized` see the block about size in the `Structs` section above.
//!
//! The discriminant for the variants defaults to incrementing by one for every field, starting with `0`.
//! You can overwrite the discriminant for a field, which will also change the discriminant for every following field.
//!
//! ## Examples
//!
//! ```
//! # use bitstream_reader_derive::BitRead;
//! #
//! #[derive(BitRead)]
//! #[discriminant_bits = 2]
//! enum TestBareEnum {
//!     Foo,
//!     Bar,
//!     Asd = 3, // manually set the discriminant value for a field
//! }
//! ```
//!
//! ```
//! # use bitstream_reader_derive::BitRead;
//! #
//! #[derive(BitRead)]
//! #[discriminant_bits = 2]
//! enum TestUnnamedFieldEnum {
//!     #[size = 5]
//!     Foo(i8),
//!     Bar(bool),
//!     #[discriminant = 3] // since rust only allows setting the discriminant on field-less enums, you can use an attribute instead
//!     Asd(u8),
//! }
//! ```
//!
//! ```
//! # use bitstream_reader_derive::BitReadSized;
//! #
//! #[derive(BitReadSized, PartialEq, Debug)]
//! #[discriminant_bits = 2]
//! enum TestUnnamedFieldEnumSized {
//!     #[size = 5]
//!     Foo(i8),
//!     Bar(bool),
//!     #[discriminant = 3]
//!     #[size = "input_size"]
//!     Asd(u8),
//! }
//! ```
//!
//! # Endianness
//!
//! If the struct that `BitRead` or `BitReadSized` is derived for requires a Endianness type parameter, you need to tell the derive macro the name of the type parameter used
//!
//! ```
//! # use bitstream_reader_derive::BitRead;
//! # use bitstream_reader::{Endianness, BitStream};
//! #
//! #[derive(BitRead)]
//! #[endianness = "E"]
//! struct EndiannessStruct<E: Endianness> {
//!     size: u8,
//!     #[size = "size"]
//!     stream: BitStream<E>,
//! }
//! ```
//!
//! This is also required if you specify which endianness the struct has
//! ```
//! # use bitstream_reader_derive::BitRead;
//! # use bitstream_reader::{BigEndian, BitStream};
//! #
//! #[derive(BitRead)]
//! #[endianness = "BigEndian"]
//! struct EndiannessStruct {
//!     size: u8,
//!     #[size = "size"]
//!     stream: BitStream<BigEndian>,
//! }
//! ```
extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, parse_str, Attribute, Data, DeriveInput, Expr, Fields, Ident,
    Lit, LitStr, Meta, Pat, Path,
};

/// See the [crate documentation](index.html) for details
#[proc_macro_derive(
    BitRead,
    attributes(size, size_bits, discriminant_bits, discriminant, endianness)
)]
pub fn derive_bitread(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_bitread_trait(input, "BitRead".to_owned(), None)
}

/// See the [crate documentation](index.html) for details
#[proc_macro_derive(
    BitReadSized,
    attributes(size, size_bits, discriminant_bits, discriminant, endianness)
)]
pub fn derive_bitread_sized(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let extra_param = parse_str::<TokenStream>(", input_size: usize").unwrap();
    derive_bitread_trait(input, "BitReadSized".to_owned(), Some(extra_param))
}

fn derive_bitread_trait(
    input: proc_macro::TokenStream,
    trait_name: String,
    extra_param: Option<TokenStream>,
) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let endianness = get_attr(&input.attrs, "endianness").map(|lit| match lit {
        Lit::Str(str) => str.value(),
        _ => panic!("endianness attribute is required to be a string"),
    });
    let mut trait_generics = input.generics.clone();
    // we need these separate generics to only add out Endianness param to the 'impl'
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();
    if endianness.is_none() {
        trait_generics
            .params
            .push(parse_quote!(_E: ::bitstream_reader::Endianness));
    }
    let (impl_generics, _, _) = trait_generics.split_for_impl();

    let parse = parse(&input.data, &name, &input.attrs);

    let endianness_placeholder = endianness.unwrap_or("_E".to_owned());
    let trait_def_str = format!(
        "::bitstream_reader::{}<{}>",
        trait_name, &endianness_placeholder
    );
    let trait_def = parse_str::<Path>(&trait_def_str).unwrap();

    let endianness_ident = Ident::new(&endianness_placeholder, input.span());

    let expanded = quote! {
        impl #impl_generics #trait_def for #name #ty_generics #where_clause {
            fn read(stream: &mut ::bitstream_reader::BitStream<#endianness_ident>#extra_param) -> ::bitstream_reader::Result<Self> {
                #parse
            }
        }
    };

    // panic!("{}", TokenStream::to_string(&expanded));

    proc_macro::TokenStream::from(expanded)
}

fn parse(data: &Data, struct_name: &Ident, attrs: &Vec<Attribute>) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let definitions = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        // Get attributes `#[..]` on each field
                        let size = get_field_size(&f.attrs, f.span());
                        let field_type = &f.ty;
                        let span = f.span();
                        match size {
                            Some(size) => {
                                quote_spanned! {span=>
                                    let #name:#field_type = {
                                        let _size: usize = #size;
                                        stream.read_sized(_size)?
                                    };
                                }
                            }
                            None => {
                                quote_spanned! {span=>
                                    let #name:#field_type = stream.read()?;
                                }
                            }
                        }
                    });
                    let struct_definition = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote_spanned! {f.span()=>
                            #name,
                        }
                    });
                    let span = data.struct_token.span();
                    quote_spanned! {span=>
                        #(#definitions)*

                        Ok(#struct_name {
                            #(#struct_definition)*
                        })
                    }
                }
                _ => unimplemented!(),
            }
        }
        Data::Enum(ref data) => {
            let discriminant_bits = match get_attr(attrs, "discriminant_bits") {
                Some(bits_lit) => match bits_lit {
                    Lit::Int(bits) => bits.value(),
                    _ => {
                        panic!("'discriminant_bits' attribute is required to be an integer literal")
                    }
                },
                None => panic!(
                    "'discriminant_bits' attribute is required when deriving `BinRead` for enums"
                ),
            };
            let discriminant_read = quote! {
                let discriminant:usize = stream.read_int(#discriminant_bits as usize)?;
            };

            let mut last_discriminant = -1;
            let mut discriminants = Vec::with_capacity(data.variants.len());
            for variant in &data.variants {
                let discriminant: Option<usize> = variant
                    .discriminant
                    .clone()
                    .map(|(_, expr)| match expr {
                        Expr::Lit(expr_lit) => expr_lit.lit,
                        _ => panic!("discriminant is required to be an integer literal"),
                    })
                    .or_else(|| get_attr(&variant.attrs, "discriminant"))
                    .map(|lit| match lit {
                        Lit::Int(lit) => Some(lit.value() as usize),
                        Lit::Str(lit) => match lit.value().as_str() {
                            "_" => None,
                            _ => {
                                panic!("discriminant is required to be an integer literal or \"_\"")
                            }
                        },
                        _ => panic!("discriminant is required to be an integer literal or \"_\""),
                    })
                    .unwrap_or_else(|| Some((last_discriminant + 1) as usize));
                if let Some(disc) = discriminant {
                    last_discriminant = disc as isize;
                }
                discriminants.push(discriminant)
            }
            let match_arms =
                data.variants
                    .iter()
                    .zip(discriminants.iter())
                    .map(|(variant, discriminant)| {
                        let span = variant.span();
                        let variant_name = &variant.ident;
                        let discriminant_string = discriminant
                            .map(|value| value.to_string())
                            .unwrap_or("_".to_string());
                        let discriminant = parse_str::<Pat>(discriminant_string.as_str()).unwrap();
                        let read_fields = match &variant.fields {
                            Fields::Unit => quote_spanned! {span=>
                                #struct_name::#variant_name
                            },
                            Fields::Unnamed(f) => {
                                let size = get_field_size(&variant.attrs, f.span());
                                match size {
                                    Some(size) => {
                                        quote_spanned! {span=>
                                            #struct_name::#variant_name({
                                                let _size:usize = #size;
                                                stream.read_sized(_size)?
                                            })
                                        }
                                    }
                                    None => {
                                        quote_spanned! {span=>
                                            #struct_name::#variant_name(stream.read()?)
                                        }
                                    }
                                }
                            }
                            _ => unimplemented!(),
                        };
                        quote_spanned! {span=>
                            #discriminant => #read_fields,
                        }
                    });

            let span = data.enum_token.span();

            let enum_name = Lit::Str(LitStr::new(&struct_name.to_string(), struct_name.span()));
            quote_spanned! {span=>
                #discriminant_read
                Ok(match discriminant {
                    #(#match_arms)*
                    _ => {
                        return Err(::bitstream_reader::ReadError::UnmatchedDiscriminant{discriminant, enum_name: #enum_name.to_string()})
                    }
                })
            }
        }
        _ => unimplemented!(),
    }
}

fn get_field_size(attrs: &Vec<Attribute>, span: Span) -> Option<TokenStream> {
    get_attr(attrs, "size")
        .map(|size_lit| match size_lit {
            Lit::Int(size) => {
                quote_spanned! {span=>
                    #size
                }
            }
            Lit::Str(size_field) => {
                let size = parse_str::<Expr>(&size_field.value()).unwrap();
                quote_spanned! {span=>
                    (#size) as usize
                }
            }
            _ => panic!("Unsupported value for size attribute"),
        })
        .or_else(|| {
            get_attr(attrs, "size_bits").map(|size_bits_lit| {
                quote_spanned! {span=>
                    stream.read_int::<usize>(#size_bits_lit)?
                }
            })
        })
}

fn get_attr(attrs: &Vec<Attribute>, name: &str) -> Option<Lit> {
    for attr in attrs.iter() {
        let meta = attr.parse_meta().unwrap();
        match meta {
            Meta::NameValue(ref name_value) if name_value.ident == name => {
                return Some(name_value.lit.clone());
            }
            _ => (),
        }
    }
    None
}

/// See the [crate documentation](index.html) for details
#[proc_macro_derive(BitSize, attributes(size))]
pub fn derive_bitsize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_bitsize_trait(input, "BitSize".to_owned(), None)
}

/// See the [crate documentation](index.html) for details
#[proc_macro_derive(BitSizeSized, attributes(size))]
pub fn derive_bitsize_sized(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let extra_param = parse_str::<TokenStream>("input_size: usize").unwrap();
    derive_bitsize_trait(input, "BitSizeSized".to_owned(), Some(extra_param))
}

fn derive_bitsize_trait(
    input: proc_macro::TokenStream,
    trait_name: String,
    extra_param: Option<TokenStream>,
) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let sum = bit_size_sum(&input.data);

    let trait_def_str = format!("::bitstream_reader::{}", trait_name);
    let trait_def = parse_str::<Path>(&trait_def_str).unwrap();

    let expanded = quote! {
        impl #impl_generics #trait_def for #name #ty_generics #where_clause {
            fn bit_size(#extra_param) -> usize {
                #sum
            }
        }
    };

    // panic!("{}", TokenStream::to_string(&expanded));

    proc_macro::TokenStream::from(expanded)
}

// Generate an expression to sum up the heap size of each field.
fn bit_size_sum(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let field_type = &f.ty;
                    let size = get_field_size(&f.attrs, f.span());
                    match size {
                        Some(size) => {
                            quote_spanned! {f.span()=>
                                ::bitstream_reader::bit_size_of_sized::<#field_type>(#size)
                            }
                        }
                        None => {
                            quote_spanned! {f.span()=>
                                ::bitstream_reader::bit_size_of::<#field_type>()
                            }
                        }
                    }
                });
                quote! {
                    0 #(+ #recurse)*
                }
            }
            Fields::Unit => quote!(0),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}
