use crate::params::FieldParam;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote_spanned;
use syn::Path;

pub fn read_struct_or_enum(
    struct_name: &Path,
    fields: &[FieldParam],
    span: Span,
    unchecked: bool,
) -> TokenStream {
    let named = fields.iter().any(|f| f.field_name.is_some());
    let values = fields.iter().map(|f| {
        let align = &f.align;
        let field_type = &f.ty;
        let span = f.span();
        let read_fn = Ident::new(if unchecked { "read_unchecked" } else { "read" }, span);
        let read_sized_fn = Ident::new(
            if unchecked {
                "read_sized_unchecked"
            } else {
                "read_sized"
            },
            span,
        );
        let end_param = if unchecked {
            Some(quote_spanned!(span => end))
        } else {
            None
        };
        match &f.size {
            Some(size) => {
                quote_spanned! { span =>
                    {
                        #align
                        let _size: usize = #size;
                        __stream.#read_sized_fn::<#field_type>(_size, #end_param)?
                    }
                }
            }
            None => {
                quote_spanned! { span =>
                    {
                        #align
                        __stream.#read_fn::<#field_type>(#end_param)?
                    }
                }
            }
        }
    });

    if named {
        let definitions = fields.iter().zip(values).map(|(f, value)| {
            let name = &f.field_name;
            quote_spanned! { span =>
                let #name = #value;
            }
        });
        let struct_definition = fields.iter().map(|f| {
            let name = f
                .field_name
                .as_ref()
                .expect("unnamed field in named struct?");
            quote_spanned! { span =>
                #name,
            }
        });
        quote_spanned! { span =>
            #(#definitions)*

            Ok(#struct_name {
                #(#struct_definition)*
            })
        }
    } else {
        quote_spanned! { span =>
            Ok(#struct_name(
                #(#values ,)*
            ))
        }
    }
}
