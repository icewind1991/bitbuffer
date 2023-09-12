use crate::params::{EnumParam, VariantBody};
use crate::read::field::read_struct_or_enum;
use proc_macro2::{Ident, TokenStream};
use quote::quote_spanned;
use syn::Path;

pub fn derive_encode_enum(params: &EnumParam, unchecked: bool) -> TokenStream {
    let discriminant_bits = params.discriminant_bits;
    let repr = params.discriminant_repr();
    let ident = params.ident.clone();
    let span = params.span;

    let match_arms = params
        .variants
        .iter()
        .zip(params.read_discriminant_tokens())
        .map(|(variant, discriminant_token)| {
            let span = variant.span();
            let variant_name = &variant.variant_name;
            let mut variant_path = Path::from(params.ident.clone());
            variant_path
                .segments
                .push(variant.variant_name.clone().into());
            let read_variant = match &variant.body {
                VariantBody::Unit => quote_spanned! { span =>
                    Ok(#ident::#variant_name)
                },
                VariantBody::Fields(fields) => {
                    read_struct_or_enum(&variant_path, fields, span.clone(), unchecked)
                }
            };

            quote_spanned! {span=>
                #discriminant_token => #read_variant,
            }
        });

    let read_fn = Ident::new(
        if unchecked {
            "read_int_unchecked"
        } else {
            "read_int"
        },
        span,
    );
    let end_param = if unchecked {
        Some(quote_spanned!(span => end))
    } else {
        None
    };
    let error_handle = if unchecked {
        None
    } else {
        Some(quote_spanned!(span => ?))
    };

    let name = ident.to_string();

    quote_spanned! {span =>
        #[allow(clippy::unnecessary_cast)]
        let discriminant:#repr = __stream.#read_fn(#discriminant_bits as usize, #end_param)#error_handle;
        match discriminant {
            #(#match_arms)*
            _ => {
                #[allow(clippy::unnecessary_cast)]
                return Err(::bitbuffer::BitError::UnmatchedDiscriminant{discriminant: discriminant as usize, enum_name: #name.to_string()})
            }
        }
    }
}
