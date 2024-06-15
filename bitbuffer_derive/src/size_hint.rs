use crate::params::{
    Alignment, EnumParam, FieldParam, InputInnerParams, InputParams, StructParam, VariantBody,
    VariantParam,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};

pub trait SizeHint {
    fn size_hint(&self) -> TokenStream;
}

impl SizeHint for FieldParam {
    fn size_hint(&self) -> TokenStream {
        let span = self.span;
        let field_type = &self.ty;
        if !self.size_can_be_predicted() {
            return quote_spanned! { span => None::<usize>};
        }
        match &self.size {
            Some(size) => {
                quote_spanned! { span =>
                    <#field_type as ::bitbuffer::BitReadSized<'_, ::bitbuffer::LittleEndian>>::bit_size_sized(#size)
                }
            }
            None => quote_spanned! { span =>
                <#field_type as ::bitbuffer::BitRead<'_, ::bitbuffer::LittleEndian>>::bit_size()
            },
        }
    }
}

impl SizeHint for VariantParam {
    fn size_hint(&self) -> TokenStream {
        match &self.body {
            VariantBody::Unit => quote!(Some(0)),
            VariantBody::Fields(fields) => product_size_hint(fields, self.span),
        }
    }
}

impl SizeHint for StructParam {
    fn size_hint(&self) -> TokenStream {
        product_size_hint(&self.fields, self.span)
    }
}

impl SizeHint for EnumParam {
    fn size_hint(&self) -> TokenStream {
        let fields = sum_size_hint(&self.variants, self.span);
        let bits = self.discriminant_bits;
        quote_spanned!(self.span => {
            Some(#bits + #fields?)
        })
    }
}

impl SizeHint for InputParams {
    fn size_hint(&self) -> TokenStream {
        match (self.align, &self.inner) {
            (Alignment::Auto, _) => quote!(None),
            (_, InputInnerParams::Struct(inner)) => inner.size_hint(),
            (_, InputInnerParams::Enum(inner)) => inner.size_hint(),
        }
    }
}

fn product_size_hint<T: SizeHint>(children: &[T], span: Span) -> TokenStream {
    let sizes = children.iter().map(|child| child.size_hint());
    quote_spanned!(span => Some(0usize)#(.and_then(|sum: usize| Some(sum + #sizes?)))*)
}

// sum types have a fixed size if all children have the same fixed size
fn sum_size_hint<T: SizeHint>(children: &[T], span: Span) -> TokenStream {
    // todo, some actual clever logic that can be const folded away
    let mut sizes = children.iter().map(|child| child.size_hint());
    let Some(first) = sizes.next() else {
        return quote!(Some(0));
    };
    quote_spanned!(span => #first#(.and_then(|prev: usize| if prev == #sizes? {
        Some(prev)
    } else {
        None
    }))*)
}
