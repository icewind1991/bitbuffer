use crate::params::parse_attrs;
use crate::params::variant::VariantParam;
use merge::Merge;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use structmeta::StructMeta;
use syn::{Attribute, DataEnum, Error, LitInt, Result};

#[derive(Default, StructMeta, Merge, Debug)]
struct EnumAttrs {
    discriminant_bits: Option<LitInt>,
}

pub struct EnumParam {
    pub span: Span,
    pub ident: Ident,
    pub variants: Vec<VariantParam>,
    pub discriminant_bits: usize,
}

impl EnumParam {
    pub fn size_can_be_predicted(&self) -> bool {
        self.variants
            .iter()
            .all(|field| field.size_can_be_predicted())
    }

    pub fn parse(
        data: &DataEnum,
        ident: Ident,
        attrs: &[Attribute],
        span: Span,
    ) -> Result<EnumParam> {
        let attrs: EnumAttrs = parse_attrs(attrs)?;
        let variants = data
            .variants
            .iter()
            .map(VariantParam::parse)
            .collect::<Result<Vec<VariantParam>>>()?;
        let discriminant_bits = attrs
            .discriminant_bits
            .ok_or_else(|| {
                Error::new(
                    span,
                    "'discriminant_bits' attribute is required when deriving `BinRead` for enums",
                )
            })?
            .base10_parse()?;

        Ok(EnumParam {
            span,
            ident,
            variants,
            discriminant_bits,
        })
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn read_discriminant_tokens(&self) -> impl Iterator<Item = TokenStream> + '_ {
        ReadDiscriminantTokenIter {
            last: -1,
            variants: self.variants.iter(),
        }
    }

    pub fn write_discriminant_tokens(&self) -> impl Iterator<Item = TokenStream> + '_ {
        WriteDiscriminantTokenIter {
            last: -1,
            max: self.max_discriminant(),
            variants: self.variants.iter(),
        }
    }

    pub fn max_discriminant(&self) -> usize {
        let mut last_discriminant = -1;

        self.variants
            .iter()
            .map(|variant| variant.discriminant.max_value(&mut last_discriminant))
            .max()
            .unwrap_or(0)
    }

    pub fn discriminant_repr(&self) -> TokenStream {
        if self.discriminant_bits <= 8 {
            quote!(u8)
        } else if self.discriminant_bits <= 16 {
            quote!(u16)
        } else if self.discriminant_bits <= 32 {
            quote!(u32)
        } else {
            quote!(u64)
        }
    }
}

pub struct ReadDiscriminantTokenIter<'a> {
    last: isize,
    variants: std::slice::Iter<'a, VariantParam>,
}

impl Iterator for ReadDiscriminantTokenIter<'_> {
    type Item = TokenStream;

    fn next(&mut self) -> Option<Self::Item> {
        let variant = self.variants.next()?;
        Some(
            variant
                .discriminant
                .read_token(&mut self.last, variant.span()),
        )
    }
}

pub struct WriteDiscriminantTokenIter<'a> {
    last: isize,
    max: usize,
    variants: std::slice::Iter<'a, VariantParam>,
}

impl Iterator for WriteDiscriminantTokenIter<'_> {
    type Item = TokenStream;

    fn next(&mut self) -> Option<Self::Item> {
        let variant = self.variants.next()?;
        Some(
            variant
                .discriminant
                .write_token(&mut self.last, self.max, variant.span()),
        )
    }
}
