use crate::params::field::FieldParam;
use proc_macro2::{Ident, Span};
use syn::{Attribute, DataStruct, Fields, Result};

pub struct StructParam {
    pub span: Span,
    pub ident: Ident,
    pub fields: Vec<FieldParam>,
    pub is_unit: bool,
}

impl StructParam {
    pub fn size_can_be_predicted(&self) -> bool {
        self.fields
            .iter()
            .all(|field| field.size_can_be_predicted())
    }

    pub fn parse(
        data: &DataStruct,
        ident: Ident,
        _attrs: &[Attribute],
        span: Span,
    ) -> Result<StructParam> {
        let fields = data
            .fields
            .iter()
            .map(FieldParam::parse)
            .collect::<Result<Vec<FieldParam>>>()?;

        let is_unit = matches!(data.fields, Fields::Unit);

        Ok(StructParam {
            span,
            ident,
            fields,
            is_unit,
        })
    }

    pub fn span(&self) -> Span {
        self.span
    }
}
