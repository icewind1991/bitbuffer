use syn::{Expr, Lit, Variant};
use syn_util::get_attribute_value;

pub enum Discriminant {
    Int(usize),
    Default,
    Wildcard,
}

impl From<Lit> for Discriminant {
    fn from(lit: Lit) -> Self {
        match lit {
            Lit::Int(lit) => Discriminant::Int(lit.base10_parse::<usize>().unwrap()),
            Lit::Str(lit) => match lit.value().as_str() {
                "_" => Discriminant::Wildcard,
                _ => panic!("discriminant is required to be an integer literal or \"_\""),
            },
            _ => panic!("discriminant is required to be an integer literal or \"_\""),
        }
    }
}

impl From<&Variant> for Discriminant {
    fn from(variant: &Variant) -> Self {
        variant
            .discriminant
            .as_ref()
            .map(|(_, expr)| match expr {
                Expr::Lit(expr_lit) => expr_lit.lit.clone(),
                _ => panic!("discriminant is required to be an integer literal"),
            })
            .or_else(|| get_attribute_value(&variant.attrs, &["discriminant"]))
            .map(Discriminant::from)
            .unwrap_or(Discriminant::Default)
    }
}
