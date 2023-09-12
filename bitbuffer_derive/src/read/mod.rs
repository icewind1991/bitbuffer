mod r#enum;
mod field;
mod r#struct;

use self::r#enum::derive_encode_enum;
use self::r#struct::derive_encode_struct;
use crate::params::{InputInnerParams, InputParams};
use crate::size_hint::SizeHint;
use crate::Derivable;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::Result;

fn parse_impl(params: &InputParams, unchecked: bool) -> Result<TokenStream> {
    Ok(match &params.inner {
        InputInnerParams::Struct(inner) => derive_encode_struct(inner, unchecked),
        InputInnerParams::Enum(inner) => derive_encode_enum(inner, unchecked),
    })
}

pub struct Read;

impl Derivable for Read {
    type Params = InputParams;

    fn derive(params: Self::Params) -> Result<TokenStream> {
        let (impl_generics, ty_generics, where_clause) = params.generics_for_impl();

        let parse = parse_impl(&params, false)?;
        let parse_unchecked = parse_impl(&params, true)?;
        let size = params.size_hint();
        let lifetime = params.lifetime.clone();
        let endianness = params.endianness();
        let name = params.ident.clone();
        let align = params.align;
        let span = params.span;

        Ok(quote_spanned! {span =>
            impl #impl_generics ::bitbuffer::BitRead<#lifetime, #endianness> for #name #ty_generics #where_clause {
                #[allow(unused_braces, unused_variables)]
                fn read(__stream: &mut ::bitbuffer::BitReadStream<#lifetime, #endianness>) -> ::bitbuffer::Result<Self> {
                    // if the read has a predicable size, we can do the bounds check in one go
                    match <Self as ::bitbuffer::BitRead<#endianness>>::bit_size() {
                        Some(size) => {
                            let end = __stream.check_read(size)?;
                            unsafe {
                                <Self as ::bitbuffer::BitRead<#endianness>>::read_unchecked(__stream, end)
                            }
                        },
                        None => {
                            #align
                            #parse
                        }
                    }
                }

                #[allow(unused_braces, unused_variables)]
                unsafe fn read_unchecked(__stream: &mut ::bitbuffer::BitReadStream<#lifetime, #endianness>, end: bool) -> ::bitbuffer::Result<Self> {
                    #align
                    #parse_unchecked
                }

                fn bit_size() -> Option<usize> {
                    #size
                }
            }
        })
    }
}

pub struct ReadSized;

impl Derivable for ReadSized {
    type Params = InputParams;

    fn derive(params: Self::Params) -> Result<TokenStream> {
        let (impl_generics, ty_generics, where_clause) = params.generics_for_impl();

        let parse = parse_impl(&params, false)?;
        let parse_unchecked = parse_impl(&params, true)?;
        let size = params.size_hint();
        let lifetime = params.lifetime.clone();
        let endianness = params.endianness();
        let name = params.ident.clone();
        let align = params.align;

        Ok(quote! {
            impl #impl_generics ::bitbuffer::BitReadSized<#lifetime, #endianness> for #name #ty_generics #where_clause {
                #[allow(unused_braces)]
                fn read(__stream: &mut ::bitbuffer::BitReadStream<#lifetime, #endianness>, input_size: usize) -> ::bitbuffer::Result<Self> {
                    // if the read has a predicable size, we can do the bounds check in one go
                    match <Self as ::bitbuffer::BitReadSized<#endianness>>::bit_size_sized(input_size) {
                        Some(size) => {
                            let end = __stream.check_read(size)?;
                            unsafe {
                                <Self as ::bitbuffer::BitReadSized<#endianness>>::read_unchecked(__stream, input_size, end)
                            }
                        },
                        None => {
                            #align
                            #parse
                        }
                    }
                }

                #[allow(unused_braces)]
                unsafe fn read_unchecked(__stream: &mut ::bitbuffer::BitReadStream<#lifetime, #endianness>, input_size: usize, end: bool) -> ::bitbuffer::Result<Self> {
                    #align
                    #parse_unchecked
                }

                fn bit_size_sized(input_size: usize) -> Option<usize> {
                    #size
                }
            }
        })
    }
}
