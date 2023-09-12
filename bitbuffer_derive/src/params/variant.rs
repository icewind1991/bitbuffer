use crate::discriminant::Discriminant;
use crate::err;
use crate::params::field::FieldParam;
use crate::params::{parse_attrs, Alignment, Size};
use merge::Merge;
use proc_macro2::{Ident, Span};
use std::convert::TryFrom;
use structmeta::StructMeta;
use syn::spanned::Spanned;
use syn::{Expr, ExprLit, Fields, Lit, LitInt, Result, Variant};

#[derive(Default, StructMeta, Merge)]
struct VariantAttrs {
    size: Option<Expr>,
    size_bits: Option<LitInt>,
    #[merge(strategy = merge::bool::overwrite_false)]
    align: bool,
    discriminant: Option<Lit>,
}

pub struct VariantParam {
    pub span: Span,
    pub variant_name: Ident,
    pub body: VariantBody,
    pub discriminant: Discriminant,
}

pub enum VariantBodyType {
    Unit,
    Unnamed,
    Named,
}

pub enum VariantBody {
    Unit,
    Fields(Vec<FieldParam>),
}

impl VariantBody {
    pub fn body_type(&self) -> VariantBodyType {
        match self {
            VariantBody::Unit => VariantBodyType::Unit,
            VariantBody::Fields(fields) => {
                let named = fields.iter().any(|f| f.field_name.is_some());
                if named {
                    VariantBodyType::Named
                } else {
                    VariantBodyType::Unnamed
                }
            }
        }
    }
}

impl VariantParam {
    /// Whether the size of the variant can be determined without having to read further bits
    pub fn size_can_be_predicted(&self) -> bool {
        match &self.body {
            VariantBody::Fields(fields) => fields.iter().all(|field| field.size_can_be_predicted()),
            VariantBody::Unit => true,
        }
    }

    pub fn parse(input: &Variant) -> Result<VariantParam> {
        let attrs: VariantAttrs = parse_attrs(&input.attrs)?;
        let variant_name = input.ident.clone();
        let align = attrs.align.into();
        let size = Size::from_attrs(attrs.size, attrs.size_bits, input.span())?;

        if attrs.discriminant.is_some() && input.discriminant.is_some() {
            err(
                "variant has both discriminant and discriminant attribute set",
                input.span(),
            )?;
        }

        let discriminant = attrs
            .discriminant
            .map(|lit| {
                Expr::Lit(ExprLit {
                    attrs: Vec::new(),
                    lit,
                })
            })
            .or_else(|| {
                input
                    .discriminant
                    .clone()
                    .map(|(_, discriminant)| discriminant)
            })
            .map(Discriminant::try_from)
            .transpose()?
            .unwrap_or(Discriminant::Default);

        let body = if matches!(input.fields, Fields::Unit) {
            if align == Alignment::Auto {
                err(
                    "'align' attribute is not allowed on unit variants",
                    input.span(),
                )?;
            }
            if size.is_some() {
                err(
                    "'size' attribute is not allowed on unit variants",
                    input.span(),
                )?;
            }
            VariantBody::Unit
        } else {
            let mut fields = input
                .fields
                .iter()
                .map(FieldParam::parse)
                .collect::<Result<Vec<FieldParam>>>()?;

            // align and size attributes on the variant go to the first field
            match (fields.first_mut(), align) {
                (Some(field), Alignment::Auto) => {
                    field.align = align;
                }
                _ => {}
            }
            match (fields.first_mut(), size) {
                (Some(field), Some(size)) => {
                    field.size = Some(size);
                }
                _ => {}
            }
            VariantBody::Fields(fields)
        };

        Ok(VariantParam {
            span: input.span(),
            variant_name,
            discriminant,
            body,
        })
    }

    pub fn span(&self) -> Span {
        self.span
    }
}
