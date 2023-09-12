use crate::params::StructParam;
use crate::write::field::write_struct;
use proc_macro2::TokenStream;
use quote::quote;

pub fn derive_encode_struct(params: &StructParam) -> TokenStream {
    let body = write_struct(&params.fields, params.span());

    quote!(
        #body
        Ok(())
    )
}
