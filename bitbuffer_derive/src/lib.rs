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
//! use bitbuffer::BitRead;
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
//! use bitbuffer::BitReadSized;
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
//! # use bitbuffer::BitRead;
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
//! # use bitbuffer::BitRead;
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
//! # use bitbuffer::BitReadSized;
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
//! # use bitbuffer::{BitRead, Endianness, BitReadStream};
//! #
//! #[derive(BitRead)]
//! #[endianness = "E"]
//! struct EndiannessStruct<E: Endianness> {
//!     size: u8,
//!     #[size = "size"]
//!     stream: BitReadStream<E>,
//! }
//! ```
//!
//! This is also required if you specify which endianness the struct has
//! ```
//! # use bitbuffer::{BitRead, BigEndian, BitReadStream};
//! #
//! #[derive(BitRead)]
//! #[endianness = "BigEndian"]
//! struct EndiannessStruct {
//!     size: u8,
//!     #[size = "size"]
//!     stream: BitReadStream<BigEndian>,
//! }
//! ```
extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, parse_str, Attribute, Data, DataStruct, DeriveInput, Expr,
    Field, Fields, Ident, Lit, LitStr, Path, Variant,
};
use syn_util::get_attribute_value;

/// See the [crate documentation](index.html) for details
#[proc_macro_derive(
    BitRead,
    attributes(size, size_bits, discriminant_bits, discriminant, endianness)
)]
pub fn derive_bitread(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_bitread_trait(input, "BitRead".to_owned(), None)
}

//
/// See the [crate documentation](index.html) for details
#[proc_macro_derive(
    BitReadSized,
    attributes(size, size_bits, discriminant_bits, discriminant, endianness)
)]
pub fn derive_bitread_sized(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let extra_param = parse_str::<TokenStream>(", input_size: usize").unwrap();
    derive_bitread_trait(input, "BitReadSized".to_owned(), Some(extra_param))
}

/// See the [crate documentation](index.html) for details
#[proc_macro_derive(
    BitWrite,
    attributes(size, size_bits, discriminant_bits, discriminant, endianness)
)]
pub fn derive_bitwrite(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_bitwrite_trait(input, "BitWrite".to_owned(), None)
}

fn derive_bitread_trait(
    input: proc_macro::TokenStream,
    trait_name: String,
    extra_param: Option<TokenStream>,
) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let endianness = get_attribute_value(&input.attrs, &["endianness"]);
    let mut trait_generics = input.generics.clone();
    // we need these separate generics to only add out Endianness param to the 'impl'
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();
    if endianness.is_none() {
        trait_generics
            .params
            .push(parse_quote!(_E: ::bitbuffer::Endianness));
    }
    let (impl_generics, _, _) = trait_generics.split_for_impl();
    let span = input.span();

    let size = size(
        input.data.clone(),
        &name,
        &input.attrs,
        extra_param.is_some(),
    );
    let parsed = parse(input.data.clone(), &name, &input.attrs, false);
    let parsed_unchecked = parse(input.data.clone(), &name, &input.attrs, true);

    let endianness_placeholder = endianness.unwrap_or_else(|| "_E".to_owned());
    let trait_def_str = format!("::bitbuffer::{}<{}>", trait_name, &endianness_placeholder);
    let trait_def = parse_str::<Path>(&trait_def_str).unwrap();

    let endianness_ident = Ident::new(&endianness_placeholder, span);

    let size_extra_param = if extra_param.is_some() {
        Some(quote!(input_size: usize))
    } else {
        None
    };

    let extra_param_call = if extra_param.is_some() {
        Some(quote!(input_size))
    } else {
        None
    };

    let size_method_name = Ident::new(
        if extra_param.is_some() {
            "bit_size_sized"
        } else {
            "bit_size"
        },
        Span::call_site(),
    );
    //
    let expanded = quote! {
        impl #impl_generics #trait_def for #name #ty_generics #where_clause {
            fn read(stream: &mut ::bitbuffer::BitReadStream<#endianness_ident>#extra_param) -> ::bitbuffer::Result<Self> {
                // if the read has a predicable size, we can do the bounds check in one go
                match <Self as #trait_def>::#size_method_name(#extra_param_call) {
                    Some(size) => {
                        stream.check_read(size)?;
                        unsafe {
                            <Self as #trait_def>::read_unchecked(stream, #extra_param_call)
                        }
                    },
                    None => {
                        #parsed
                    }
                }
            }

            unsafe fn read_unchecked(stream: &mut ::bitbuffer::BitReadStream<#endianness_ident>#extra_param) -> ::bitbuffer::Result<Self> {
                #parsed_unchecked
            }

            fn #size_method_name(#size_extra_param) -> Option<usize> {
                #size
            }
        }
    };

    // panic!("{}", TokenStream::to_string(&expanded));

    proc_macro::TokenStream::from(expanded)
}

fn parse(data: Data, struct_name: &Ident, attrs: &[Attribute], unchecked: bool) -> TokenStream {
    let span = struct_name.span();

    match data {
        Data::Struct(DataStruct { fields, .. }) => {
            let values = fields.iter().map(|f| {
                // Get attributes `#[..]` on each field
                let size = get_field_size(&f.attrs, f.span(), true);
                let field_type = &f.ty;
                let span = f.span();
                if unchecked {
                    match size {
                        Some(size) => {
                            quote_spanned! { span =>
                                {
                                    let _size: usize = #size;
                                    stream.read_sized_unchecked::<#field_type>(_size)?
                                }
                            }
                        }
                        None => {
                            quote_spanned! { span =>
                                stream.read_unchecked::<#field_type>()?
                            }
                        }
                    }
                } else {
                    match size {
                        Some(size) => {
                            quote_spanned! { span =>
                                {
                                    let _size: usize = #size;
                                    stream.read_sized::<#field_type>(_size)?
                                }
                            }
                        }
                        None => {
                            quote_spanned! { span =>
                                stream.read::<#field_type>()?
                            }
                        }
                    }
                }
            });

            match &fields {
                Fields::Named(fields) => {
                    let definitions = fields.named.iter().zip(values).map(|(f, value)| {
                        let name = &f.ident;
                        quote_spanned! { f.span() =>
                            let #name = #value;
                        }
                    });
                    let struct_definition = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote_spanned! { f.span() =>
                            #name,
                        }
                    });
                    quote_spanned! { span =>
                        #(#definitions)*

                        Ok(#struct_name {
                            #(#struct_definition)*
                        })
                    }
                }
                Fields::Unnamed(_) => quote_spanned! { span =>
                    Ok(#struct_name(
                        #(#values ,)*
                    ))
                },
                Fields::Unit => quote_spanned! {span=>
                    Ok(#struct_name)
                },
            }
        }
        Data::Enum(data) => {
            let discriminant_bits: u64 = get_attribute_value(attrs, &["discriminant_bits"]).expect(
                "'discriminant_bits' attribute is required when deriving `BinRead` for enums",
            );

            let mut last_discriminant = -1;
            let match_arms = data.variants.iter().map(|variant| {
                let span = variant.span();
                let variant_name = &variant.ident;
                let read_fields = match &variant.fields {
                    Fields::Unit => quote_spanned! {span=>
                        #struct_name::#variant_name
                    },
                    Fields::Unnamed(f) => {
                        let size = get_field_size(&variant.attrs, f.span(), true);
                        match size {
                            Some(size) => {
                                quote_spanned! { span =>
                                    #struct_name::#variant_name({
                                        let _size:usize = #size;
                                        stream.read_sized(_size)?
                                    })
                                }
                            }
                            None => {
                                quote_spanned! { span =>
                                    #struct_name::#variant_name(stream.read()?)
                                }
                            }
                        }
                    }
                    _ => unimplemented!(),
                };

                let discriminant_token = get_discriminant_token(variant, &mut last_discriminant);
                quote_spanned! {span=>
                    #discriminant_token => #read_fields,
                }
            });

            let span = data.enum_token.span();

            let enum_name = Lit::Str(LitStr::new(&struct_name.to_string(), struct_name.span()));
            quote_spanned! {span=>
                let discriminant:usize = stream.read_int(#discriminant_bits as usize)?;
                Ok(match discriminant {
                    #(#match_arms)*
                    _ => {
                        return Err(::bitbuffer::BitError::UnmatchedDiscriminant{discriminant, enum_name: #enum_name.to_string()})
                    }
                })
            }
        }
        _ => unimplemented!(),
    }
}

fn size(data: Data, struct_name: &Ident, attrs: &[Attribute], has_input_size: bool) -> TokenStream {
    let span = struct_name.span();

    match data {
        Data::Struct(DataStruct { fields, .. }) => {
            let sizes = fields.iter().map(|f| {
                // Get attributes `#[..]` on each field
                if is_const_size(&f.attrs, has_input_size) {
                    let size = get_field_size(&f.attrs, f.span(), true);
                    let field_type = &f.ty;
                    let span = f.span();
                    match size {
                        Some(size) => {
                            quote_spanned! { span =>
                                <#field_type as ::bitbuffer::BitReadSized<::bitbuffer::LittleEndian>>::bit_size_sized(#size)
                            }
                        }
                        None => {
                            quote_spanned! { span =>
                                <#field_type as ::bitbuffer::BitRead<::bitbuffer::LittleEndian>>::bit_size()
                            }
                        }
                    }
                } else {
                    quote_spanned! { span =>
                        None
                    }
                }
            });

            match &fields {
                Fields::Named(_) => quote_spanned! { span =>
                    Some(0usize)#(.and_then(|sum: usize| #sizes.map(|size: usize| sum + size)))*
                },
                Fields::Unnamed(_) => quote_spanned! { span =>
                    Some(0usize)#(.and_then(|sum: usize| #sizes.map(|size: usize| sum + size)))*
                },
                Fields::Unit => quote_spanned! {span=>
                    Some(0usize)
                },
            }
        }
        Data::Enum(data) => {
            let discriminant_bits = get_attribute_value::<u64>(attrs, &["discriminant_bits"])
                .expect(
                    "'discriminant_bits' attribute is required when deriving `BinRead` for enums",
                ) as usize;

            let is_unit = data.variants.iter().all(|variant| match &variant.fields {
                Fields::Unit => true,
                _ => false,
            });

            if is_unit {
                quote_spanned! {span=>
                    Some(#discriminant_bits)
                }
            } else {
                quote_spanned! {span=>
                    None
                }
            }
        }
        _ => unimplemented!(),
    }
}

fn get_discriminant_token(variant: &Variant, last_discriminant: &mut isize) -> TokenStream {
    let span = variant.span();
    match Discriminant::from(variant) {
        Discriminant::Int(discriminant) => {
            *last_discriminant = discriminant as isize;
            quote_spanned! { span => #discriminant }
        }
        Discriminant::Wildcard => quote_spanned! { span => _ },
        Discriminant::Default => {
            let new_discriminant = (*last_discriminant + 1) as usize;
            *last_discriminant += 1;
            quote_spanned! { span => #new_discriminant }
        }
    }
}

fn derive_bitwrite_trait(
    input: proc_macro::TokenStream,
    trait_name: String,
    extra_param: Option<TokenStream>,
) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let endianness = get_attribute_value(&input.attrs, &["endianness"]);
    let mut trait_generics = input.generics.clone();
    // we need these separate generics to only add out Endianness param to the 'impl'
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();
    if endianness.is_none() {
        trait_generics
            .params
            .push(parse_quote!(_E: ::bitbuffer::Endianness));
    }
    let (impl_generics, _, _) = trait_generics.split_for_impl();
    let span = input.span();

    let size = size(
        input.data.clone(),
        &name,
        &input.attrs,
        extra_param.is_some(),
    );
    let write = write(input.data.clone(), &name, &input.attrs);
    let parsed_unchecked = parse(input.data.clone(), &name, &input.attrs, true);

    let endianness_placeholder = endianness.unwrap_or_else(|| "_E".to_owned());
    let trait_def_str = format!("::bitbuffer::{}<{}>", trait_name, &endianness_placeholder);
    let trait_def = parse_str::<Path>(&trait_def_str).unwrap();

    let endianness_ident = Ident::new(&endianness_placeholder, span);

    let size_extra_param = if extra_param.is_some() {
        Some(quote!(input_size: usize))
    } else {
        None
    };

    let extra_param_call = if extra_param.is_some() {
        Some(quote!(input_size))
    } else {
        None
    };

    //
    let expanded = quote! {
        impl #impl_generics #trait_def for #name #ty_generics #where_clause {
            fn write(&self, stream: &mut ::bitbuffer::BitWriteStream<#endianness_ident>#extra_param) -> ::bitbuffer::Result<()> {
                #write
            }
        }
    };

    //    panic!("{}", TokenStream::to_string(&expanded));

    proc_macro::TokenStream::from(expanded)
}

fn write(data: Data, struct_name: &Ident, attrs: &[Attribute]) -> TokenStream {
    let span = struct_name.span();

    match data {
        Data::Struct(DataStruct { fields, .. }) => {
            let destructure = fields.iter().map(|field| {
                let span = field.span();
                if let Some(name) = &field.ident {
                    quote_spanned! { span => let #name = &self.#name; }
                } else {
                    quote! {}
                }
            });
            let write_field = |index: usize, field: &Field| {
                let span = field.span();
                let size = get_field_size(&field.attrs, span, false);
                let field_type = &field.ty;
                let name = field
                    .ident
                    .as_ref()
                    .map(|name| quote_spanned! { span => #name})
                    .unwrap_or(quote_spanned! { span => 0});
                match size {
                    Some(size) => {
                        quote_spanned! { span =>
                            {
                                let _size: usize = #size;
                                stream.write_sized::<#field_type>(&self.#name, _size)?
                            };
                        }
                    }
                    None => {
                        quote_spanned! { span =>
                            stream.write::<#field_type>(&self.#name)?;
                        }
                    }
                }
            };

            let writes = fields
                .iter()
                .enumerate()
                .map(|(index, field)| write_field(index, field));

            quote_spanned! { span =>
                #(#destructure)*
                #(#writes)*
                Ok(())
            }
        }
        Data::Enum(data) => {
            let discriminant_bits: u64 = get_attribute_value(attrs, &["discriminant_bits"]).expect(
                "'discriminant_bits' attribute is required when deriving `BinWrite` for enums",
            );

            let mut last_discriminant = -1;
            let match_arms = data.variants.iter().map(|variant| {
                let discriminant_token = get_discriminant_token(variant, &mut last_discriminant);

                let span = variant.span();
                let variant_name = &variant.ident;
                match &variant.fields {
                    Fields::Unit => quote_spanned! {span=>
                        #struct_name::#variant_name => stream.write_int(#discriminant_token, #discriminant_bits as usize)
                    },
                    Fields::Unnamed(f) => {
                        let size = get_field_size(&variant.attrs, f.span(), false);
                        match size {
                            Some(size) => {
                                quote_spanned! { span =>
                                     #struct_name::#variant_name(inner) => {
                                        stream.write_int(#discriminant_token, #discriminant_bits as usize)?;
                                        stream.write_sized(inner, #size)
                                    }
                                }
                            }
                            None => {
                                quote_spanned! { span =>
                                    #struct_name::#variant_name(inner) => {
                                        stream.write_int(#discriminant_token, #discriminant_bits as usize)?;
                                        stream.write(inner)
                                    }
                                }
                            }
                        }
                    }
                    _ => unimplemented!(),
                }
            });

            let span = data.enum_token.span();

            let enum_name = Lit::Str(LitStr::new(&struct_name.to_string(), struct_name.span()));
            quote_spanned! {span=>
                match self {
                    #(#match_arms),*
                }
            }
        }
        _ => unimplemented!(),
    }
}

fn is_const_size(attrs: &[Attribute], has_input_size: bool) -> bool {
    if get_attribute_value::<Lit>(attrs, &["size_bits"]).is_some() {
        return false;
    }
    get_attribute_value(attrs, &["size"])
        .map(|size_lit| match size_lit {
            Lit::Int(_) => true,
            Lit::Str(size_field) => &size_field.value() == "input_size" && has_input_size,
            _ => panic!("Unsupported value for size attribute"),
        })
        .unwrap_or(true)
}

fn get_field_size(attrs: &[Attribute], span: Span, is_read: bool) -> Option<TokenStream> {
    get_attribute_value(attrs, &["size"])
        .map(|size_lit| match size_lit {
            Lit::Int(size) => {
                quote_spanned! {span =>
                    #size
                }
            }
            Lit::Str(size_field) => {
                let size = parse_str::<Expr>(&size_field.value()).unwrap();
                if !is_read {
                    // we borrow the field so we need to deref
                    quote_spanned! {span =>
                        *(#size) as usize
                    }
                } else {
                    quote_spanned! {span =>
                        (#size) as usize
                    }
                }
            }
            _ => panic!("Unsupported value for size attribute"),
        })
        .or_else(|| {
            get_attribute_value::<Lit>(attrs, &["size_bits"]).map(|size_bits_lit| {
                if is_read {
                    quote_spanned! {span =>
                        stream.read_int::<usize> (#size_bits_lit) ?
                    }
                } else {
                    panic!("size_bits is not allowed here")
                }
            })
        })
}

enum Discriminant {
    Int(usize),
    Default,
    Wildcard,
}

impl From<Lit> for Discriminant {
    fn from(lit: Lit) -> Self {
        match lit {
            Lit::Int(lit) => Discriminant::Int(lit.base10_parse::<usize>().unwrap()),
            Lit::Str(lit) => match lit.value().as_str() {
                "_" => Discriminant::Wildcard,
                _ => panic!("discriminant is required to be an integer literal or \"_\""),
            },
            _ => panic!("discriminant is required to be an integer literal or \"_\""),
        }
    }
}

impl From<&Variant> for Discriminant {
    fn from(variant: &Variant) -> Self {
        variant
            .discriminant
            .as_ref()
            .map(|(_, expr)| match expr {
                Expr::Lit(expr_lit) => expr_lit.lit.clone(),
                _ => panic!("discriminant is required to be an integer literal"),
            })
            .or_else(|| get_attribute_value(&variant.attrs, &["discriminant"]))
            .map(Discriminant::from)
            .unwrap_or(Discriminant::Default)
    }
}
