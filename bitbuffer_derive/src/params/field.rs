use crate::params::{parse_attrs, Alignment, Size};
use merge::Merge;
use proc_macro2::{Ident, Span};
use structmeta::StructMeta;
use syn::spanned::Spanned;
use syn::{Expr, Field, Index, LitInt, Member, Result, Type};

#[derive(Default, StructMeta, Merge)]
struct FieldAttrs {
    size: Option<Expr>,
    size_bits: Option<LitInt>,
    #[merge(strategy = merge::bool::overwrite_false)]
    align: bool,
}

pub struct FieldParam {
    pub span: Span,
    pub field_name: Option<Ident>,
    pub size: Option<Size>,
    pub align: Alignment,
    pub ty: Type,
}

impl FieldParam {
    /// Whether the size of the field can be determined without having to read further bits
    pub fn size_can_be_predicted(&self) -> bool {
        if self.align == Alignment::Auto {
            return false;
        }
        match &self.size {
            Some(size) => size.is_const(),
            None => true,
        }
    }

    pub fn parse(input: &Field) -> Result<FieldParam> {
        let attrs: FieldAttrs = parse_attrs(&input.attrs)?;
        let field_name = input.ident.clone();
        let align = attrs.align.into();
        let size = Size::from_attrs(attrs.size, attrs.size_bits, input.span())?;
        let ty = input.ty.clone();

        Ok(FieldParam {
            span: input.span(),
            field_name,
            size,
            align,
            ty,
        })
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn member(&self, index: u32) -> Member {
        match self.field_name.as_ref() {
            Some(name) => Member::Named(name.clone()),
            None => Member::Unnamed(Index {
                index,
                span: self.span(),
            }),
        }
    }

    pub fn is_int(&self) -> bool {
        if let Type::Path(path) = &self.ty {
            if let Some(ident) = path.path.get_ident() {
                let name = ident.to_string();
                matches!(
                    name.as_str(),
                    "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32" | "i64" | "isize"
                )
            } else {
                false
            }
        } else {
            false
        }
    }
}
