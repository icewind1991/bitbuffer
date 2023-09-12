use crate::params::StructParam;
use crate::read::field::read_struct_or_enum;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Path;

pub fn derive_encode_struct(params: &StructParam, unchecked: bool) -> TokenStream {
    let path = Path::from(params.ident.clone());
    if params.is_unit {
        quote!(Ok(#path))
    } else {
        read_struct_or_enum(&path, &params.fields, params.span(), unchecked)
    }
}
