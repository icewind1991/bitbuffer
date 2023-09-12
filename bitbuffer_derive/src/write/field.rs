use crate::params::FieldParam;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote_spanned;
use syn::Path;

pub fn write_struct(fields: &[FieldParam], span: Span) -> TokenStream {
    let expand = fields
        .iter()
        .enumerate()
        .zip(names(fields))
        .map(|((index, field), name)| {
            let member = field.member(index as u32);
            let size_field = match (field.is_int(), field.field_name.as_ref()) {
                (true, Some(name)) => Some(quote_spanned! { field.span() =>
                    #[allow(unused_variables)]
                    let #name = self.#member;
                }),
                _ => None,
            };
            quote_spanned! { field.span() =>
                #size_field
                #[allow(unused_variables)]
                let #name = &self.#member;
            }
        });
    let writes = writes(fields);

    quote_spanned! {span=>
        #(#expand)*
        #(#writes)*
    }
}

fn names(fields: &[FieldParam]) -> impl Iterator<Item = Ident> + '_ {
    fields
        .iter()
        .enumerate()
        .map(|(index, field)| Ident::new(&format!("__field_{}", index), field.span()))
}

fn writes(fields: &[FieldParam]) -> impl Iterator<Item = TokenStream> + '_ {
    let names = names(fields);
    fields.iter().zip(names).map(|(field, name)| {
        let align = &field.align.write();
        let span = field.span();
        match &field.size {
            Some(size) => {
                quote_spanned! { span =>
                    {
                        #align
                        let _size: usize = #size;
                        __stream.write_sized(#name, _size)?;
                    }
                }
            }
            None => {
                quote_spanned! { span =>
                    {
                        #align
                        __stream.write(#name)?;
                    }
                }
            }
        }
    })
}

pub fn write_enum_variant(variant: Path, fields: &[FieldParam], span: Span) -> TokenStream {
    let names = names(fields);
    let named = fields.iter().any(|f| f.field_name.is_some());
    let writes = writes(fields);
    if named {
        quote_spanned!(span => #variant{#(#names,)*} => {
            #(#writes;)*
        })
    } else {
        quote_spanned!(span => #variant(#(#names,)*) => {
            #(#writes;)*
        })
    }
}
