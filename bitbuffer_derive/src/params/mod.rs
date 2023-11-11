mod r#enum;
mod field;
mod r#struct;
mod variant;

pub use crate::params::field::FieldParam;
pub use crate::params::r#enum::EnumParam;
pub use crate::params::r#struct::StructParam;
pub use crate::params::variant::{VariantBody, VariantBodyType, VariantParam};
use crate::{err, DeriveParams};
use merge::Merge;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};
use std::any::type_name;
use std::fmt::Debug;
use structmeta::StructMeta;
use syn::__private::{bool, IntoSpans};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::token::Paren;
use syn::{
    parse_quote, parse_str, Attribute, Data, DeriveInput, Expr, ExprLit, ExprPath, GenericParam,
    Generics, ImplGenerics, Lifetime, Lit, LitBool, LitInt, LitStr, MacroDelimiter, Meta, MetaList,
    Result, TypeGenerics, WhereClause,
};

pub enum Size {
    Expression(Expr, Span),
    Bits(usize, Span),
}

impl Size {
    pub fn is_const(&self) -> bool {
        match self {
            Size::Expression(
                Expr::Lit(ExprLit {
                    lit: Lit::Int(_), ..
                }),
                _,
            ) => true,
            Size::Expression(Expr::Path(ExprPath { path, .. }), _) => path.is_ident("input_size"),
            _ => false,
        }
    }

    pub fn from_attrs(
        size: Option<Expr>,
        size_bits: Option<LitInt>,
        span: Span,
    ) -> Result<Option<Self>> {
        Ok(match (size, size_bits) {
            (
                Some(Expr::Lit(ExprLit {
                    lit: Lit::Str(field),
                    ..
                })),
                None,
            ) => Some(Size::Expression(parse_str(&field.value())?, span)),
            (Some(size), None) => Some(Size::Expression(size, span)),
            (None, Some(bits)) => Some(Size::Bits(bits.base10_parse()?, span)),
            (Some(_), Some(_)) => err("#[size] and #[size_bits] are mutually exclusive", span)?,
            (None, None) => None,
        })
    }
}

impl ToTokens for Size {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Size::Expression(expr, span) => {
                let span = *span;
                tokens.append_all(quote_spanned! {span => {
                        #[allow(clippy::unnecessary_cast)]
                        let __size = (#expr) as usize;
                        __size
                    }
                });
            }
            Size::Bits(bits, span) => {
                let span = *span;
                tokens.append_all(quote_spanned! {span => {
                        __stream.read_int::<usize>(#bits)?
                    }
                });
            }
        }
    }
}

#[derive(Default, PartialOrd, PartialEq, Copy, Clone, Debug)]
pub enum Alignment {
    #[default]
    None,
    Auto,
}

impl Alignment {
    pub fn write(&self) -> TokenStream {
        match self {
            Alignment::Auto => quote! {
                __stream.align();
            },
            Alignment::None => quote!(),
        }
    }
}

impl From<bool> for Alignment {
    fn from(value: bool) -> Self {
        match value {
            true => Alignment::Auto,
            false => Alignment::None,
        }
    }
}

impl Parse for Alignment {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(LitBool::parse(input)?.value.into())
    }
}

impl Merge for Alignment {
    fn merge(&mut self, other: Self) {
        if other == Alignment::Auto {
            *self = Alignment::Auto
        }
    }
}

impl ToTokens for Alignment {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Alignment::Auto => tokens.append_all(quote! {
                __stream.align()?;
            }),
            Alignment::None => {}
        }
    }
}

#[derive(Default, StructMeta, Merge, Debug)]
struct InputAttrs {
    endianness: Option<LitStr>,
    #[merge(strategy = merge::bool::overwrite_false)]
    align: bool,
}

pub struct InputParams {
    pub ident: Ident,
    pub span: Span,
    endianness: Option<String>,
    pub align: Alignment,
    pub generics: Generics,
    pub generics_with_endianness: Generics,
    pub inner: InputInnerParams,
    pub lifetime: Lifetime,
}

pub enum InputInnerParams {
    Struct(StructParam),
    Enum(EnumParam),
}

impl DeriveParams for InputParams {
    fn parse(input: &DeriveInput) -> Result<Self> {
        let attrs: InputAttrs = parse_attrs(&input.attrs)?;
        let inner = match &input.data {
            Data::Struct(data) => InputInnerParams::Struct(StructParam::parse(
                data,
                input.ident.clone(),
                &input.attrs,
                input.span(),
            )?),
            Data::Enum(data) => InputInnerParams::Enum(EnumParam::parse(
                data,
                input.ident.clone(),
                &input.attrs,
                input.span(),
            )?),
            _ => return err("Only structs and enums are supported", input.span()),
        };

        let endianness = attrs.endianness.map(|lit| lit.value());
        let align = attrs.align.into();

        let generics = input.generics.clone();
        let mut generics_with_endianness = generics.clone();
        let mut lifetimes = input
            .generics
            .params
            .iter()
            .filter_map(|param| match param {
                GenericParam::Lifetime(lifetime) => Some(lifetime),
                _ => None,
            });
        let lifetime = match (lifetimes.next(), lifetimes.next()) {
            (_, Some(_)) => {
                return err("Only a single lifetime generic is supported", input.span())
            }
            (Some(param), None) => param.lifetime.clone(),
            (None, None) => {
                let lifetime = Lifetime::new("'a", input.span());
                generics_with_endianness
                    .params
                    .push(GenericParam::Lifetime(parse_str("'a").unwrap()));
                lifetime
            }
        };

        if endianness.is_none() {
            generics_with_endianness
                .params
                .push(parse_quote!(_E: ::bitbuffer::Endianness));
        }

        Ok(InputParams {
            ident: input.ident.clone(),
            span: input.span(),
            endianness,
            align,
            generics,
            generics_with_endianness,
            lifetime,
            inner,
        })
    }
}

impl InputParams {
    #[allow(dead_code)]
    pub fn size_can_be_predicted(&self) -> bool {
        match &self.inner {
            InputInnerParams::Struct(inner) => inner.size_can_be_predicted(),
            InputInnerParams::Enum(inner) => inner.size_can_be_predicted(),
        }
    }

    pub fn generics_for_impl(&self) -> (ImplGenerics, TypeGenerics, Option<&WhereClause>) {
        // we need these separate generics to only add out Endianness param to the 'impl'
        let (_, ty_generics, where_clause) = self.generics.split_for_impl();
        let (impl_generics, _, _) = self.generics_with_endianness.split_for_impl();

        (impl_generics, ty_generics, where_clause)
    }

    pub fn endianness(&self) -> Ident {
        Ident::new(self.endianness.as_deref().unwrap_or("_E"), self.span)
    }
}

const BARE_ATTRS: &[&str] = &[
    "size",
    "size_bits",
    "discriminant_bits",
    "discriminant",
    "endianness",
    "align",
];

fn parse_attrs<T: Parse + Default + Merge>(attrs: &[Attribute]) -> Result<T> {
    let mut result = T::default();
    for attr in attrs {
        let parsed = if BARE_ATTRS
            .iter()
            .any(|name| attr.meta.path().is_ident(name))
        {
            let wrapped_meta = Meta::List(MetaList {
                path: parse_str("bitbuffer").unwrap(),
                delimiter: MacroDelimiter::Paren(Paren {
                    span: attr.span().into_spans(),
                }),
                tokens: attr.meta.clone().into_token_stream(),
            });
            let wrapped = Attribute {
                pound_token: attr.pound_token,
                style: attr.style,
                bracket_token: attr.bracket_token,
                meta: wrapped_meta,
            };
            wrapped.parse_args()
        } else {
            attr.parse_args()
        };
        match parsed {
            Ok(parsed) => {
                result.merge(parsed);
            }
            Err(e) => {
                // since we first parse our attrs as InputAttrs, and then the same attrs as either an Struct or EnumAttrs
                // when doing the first pass we expect a bunch of extra parameters
                let is_first_pass = type_name::<T>() == type_name::<InputAttrs>();
                if !e.to_string().starts_with("cannot find parameter") && !is_first_pass {
                    return Err(e);
                }
            }
        }
    }
    Ok(result)
}
