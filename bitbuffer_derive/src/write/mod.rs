pub mod r#enum;
pub mod field;
pub mod r#struct;

use self::r#enum::derive_encode_enum;
use self::r#struct::derive_encode_struct;
use crate::params::{InputInnerParams, InputParams};
use crate::Derivable;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Result;

fn encode_impl(params: &InputParams) -> Result<TokenStream> {
    Ok(match &params.inner {
        InputInnerParams::Struct(inner) => derive_encode_struct(inner),
        InputInnerParams::Enum(inner) => derive_encode_enum(inner),
    })
}

pub struct Write;

impl Derivable for Write {
    type Params = InputParams;

    fn derive(params: Self::Params) -> Result<TokenStream> {
        let (impl_generics, ty_generics, where_clause) = params.generics_for_impl();

        let encode = encode_impl(&params)?;
        let endianness = params.endianness();
        let name = params.ident.clone();
        let align = params.align.write();

        Ok(quote! {
            impl #impl_generics ::bitbuffer::BitWrite<#endianness> for #name #ty_generics #where_clause {
                #[allow(unused_braces)]
                fn write(&self, __stream: &mut ::bitbuffer::BitWriteStream<#endianness>) -> ::bitbuffer::Result<()> {
                    #align
                    #encode
                }
            }
        })
    }
}

pub struct WriteSized;

impl Derivable for WriteSized {
    type Params = InputParams;

    fn derive(params: Self::Params) -> Result<TokenStream> {
        let (impl_generics, ty_generics, where_clause) = params.generics_for_impl();

        let encode = encode_impl(&params)?;
        let endianness = params.endianness();
        let name = params.ident.clone();
        let align = params.align.write();

        Ok(quote! {
            impl #impl_generics ::bitbuffer::BitWriteSized<#endianness> for #name #ty_generics #where_clause {
                #[allow(unused_braces)]
                fn write_sized(&self, __stream: &mut ::bitbuffer::BitWriteStream<#endianness>, input_size: usize) -> ::bitbuffer::Result<()> {
                    #align
                    #encode
                }
            }
        })
    }
}
