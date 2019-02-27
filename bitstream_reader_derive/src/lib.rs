//! Automatically generate `Read` implementations for structs
//!
//! The implementation can be derived as long as every field in the struct implements `Read` or `ReadSized`
//!
//! # Examples
//!
//! ```
//! use bitstream_reader_derive::Read;
//!
//! #[derive(Read)]
//! struct TestStruct {
//!    foo: u8,
//!    str: String,
//!    #[size = 2] // when `size` is set, the attributed will be read using `read_sized`
//!    truncated: String,
//!    bar: u16,
//!    float: f32,
//!    #[size = 3]
//!    asd: u8,
//!    #[size_bits = 2] // first read 2 bits, then use the resulting value as size as size for the read
//!    dynamic_length: u8,
//!    #[size = "asd"] // use a previously defined field as size
//!    previous_field: u8,
//! }
//! ```
extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Field, Fields, GenericParam, Generics,
    Ident, Lit, Meta,
};

/// See the [crate documentation](index.html) for details
#[proc_macro_derive(Read, attributes(size, size_bits))]
pub fn derive_helper_attr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let mut trait_generics = generics.clone();
    // we need these separate generics to only add out Endianness param to the 'impl'
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    trait_generics
        .params
        .push(parse_quote!(_E: ::bitstream_reader::Endianness));
    let (impl_generics, _, _) = trait_generics.split_for_impl();

    let parse = parse(&input.data, &name);

    let expanded = quote! {
        impl #impl_generics ::bitstream_reader::Read<_E> for #name #ty_generics #where_clause {
            fn read(stream: &mut ::bitstream_reader::BitStream<_E>) -> ::bitstream_reader::Result<Self> {
                #parse
            }
        }
    };

    // panic!("{}", TokenStream::to_string(&expanded));

    proc_macro::TokenStream::from(expanded)
}

// Add a bound `T: Read` to every type parameter T.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(::bitstream_reader::Read));
        }
    }
    generics
}

fn parse(data: &Data, struct_name: &Ident) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let definitions = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        // Get attributes `#[..]` on each field
                        let size = get_field_size(f);
                        let field_type = &f.ty;
                        match size {
                            Some(size) => {
                                quote_spanned! { f.span() =>
                                    let size: usize = #size;
                                    let #name:#field_type  = stream.read_sized(size)?;
                                }
                            }
                            None => {
                                quote_spanned! { f.span() =>
                                    let #name:#field_type = stream.read()?;
                                }
                            }
                        }
                    });
                    let struct_definition = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote_spanned! { f.span() =>
                            #name,
                        }
                    });
                    quote! {
                        #(#definitions)*

                        Ok(#struct_name {
                            #(#struct_definition)*
                        })
                    }
                }
                _ => unimplemented!(),
            }
        }
        _ => unimplemented!(),
    }
}

fn get_field_size(field: &Field) -> Option<TokenStream> {
    get_field_attr(field, "size")
        .map(|size_lit| match size_lit {
            Lit::Int(size) => {
                quote! {
                    #size
                }
            }
            Lit::Str(size_field) => {
                let size = Ident::new(&size_field.value(), Span::call_site());
                quote! {
                    #size as usize
                }
            }
            _ => panic!("Unsupported value for size attribute"),
        })
        .or_else(|| {
            get_field_attr(field, "size_bits").map(|size_bits_lit| {
                quote_spanned! { field.span() =>
                    stream.read_int::<usize>(#size_bits_lit)?
                }
            })
        })
}

fn get_field_attr(field: &Field, name: &str) -> Option<Lit> {
    for attr in field.attrs.iter() {
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
