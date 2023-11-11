use crate::params::{EnumParam, VariantBody, VariantBodyType};
use crate::write::field::write_enum_variant;
use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::Path;

pub fn derive_encode_enum(params: &EnumParam) -> TokenStream {
    let discriminant_bits = params.discriminant_bits;
    let repr = params.discriminant_repr();
    let ident = params.ident.clone();
    let span = params.span();

    let discriminant_value = params
        .variants
        .iter()
        .zip(params.write_discriminant_tokens())
        .map(|(variant, discriminant_token)| {
            let span = variant.span();
            let variant_name = &variant.variant_name;
            match variant.body.body_type() {
                VariantBodyType::Unit => quote_spanned! {span =>
                    #ident::#variant_name => #discriminant_token
                },
                VariantBodyType::Unnamed => {
                    quote_spanned! { span =>
                        #ident::#variant_name(_) => #discriminant_token
                    }
                }
                VariantBodyType::Named => {
                    quote_spanned! { span =>
                        #ident::#variant_name{..} => #discriminant_token
                    }
                }
            }
        });

    let write_inner = params.variants.iter().map(|variant| {
        let span = variant.span();
        let mut path = Path::from(ident.clone());
        path.segments.push(variant.variant_name.clone().into());

        match &variant.body {
            VariantBody::Unit => {
                quote_spanned! {span =>
                    #path => {},
                }
            }
            VariantBody::Fields(fields) => write_enum_variant(path, fields, span),
        }
    });

    quote_spanned! {span=>
        let discriminant:#repr = match &self {
            #(#discriminant_value),*
        };
        #[allow(clippy::unnecessary_cast)]
        __stream.write_int(discriminant, #discriminant_bits as usize)?;
        match &self {
            #(#write_inner)*
        }
        Ok(())
    }
}
