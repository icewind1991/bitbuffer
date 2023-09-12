use crate::err;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use std::convert::{TryFrom, TryInto};
use syn::spanned::Spanned;
use syn::{Error, Expr, ExprLit, Lit, LitInt};

#[derive(Copy, Clone)]
pub enum Discriminant {
    Int(usize),
    Default,
    Wildcard,
}

impl TryFrom<Expr> for Discriminant {
    type Error = Error;

    fn try_from(value: Expr) -> Result<Self, Self::Error> {
        match value {
            Expr::Lit(ExprLit { lit, .. }) => lit.try_into(),
            _ => err("non literal discriminant", value.span())?,
        }
    }
}

impl TryFrom<Lit> for Discriminant {
    type Error = Error;

    fn try_from(value: Lit) -> Result<Self, Self::Error> {
        let span = value.span();
        match value {
            Lit::Int(lit) => Ok(Discriminant::Int(lit.base10_parse()?)),
            Lit::Str(lit) => match lit.value().as_str() {
                "_" => Ok(Discriminant::Wildcard),
                _ => err(
                    "discriminant is required to be an integer literal or \"_\"",
                    span,
                ),
            },
            _ => err(
                "discriminant is required to be an integer literal or \"_\"",
                span,
            ),
        }
    }
}

impl Discriminant {
    pub fn read_token(&self, last_discriminant: &mut isize, span: Span) -> TokenStream {
        match self {
            Discriminant::Int(discriminant) => {
                let lit = LitInt::new(&format!("{}", discriminant), span);
                *last_discriminant = *discriminant as isize;
                quote! { #lit }
            }
            Discriminant::Wildcard => quote! { _ },
            Discriminant::Default => {
                let new_discriminant = (*last_discriminant + 1) as usize;
                let lit = LitInt::new(&format!("{}", new_discriminant), span);
                *last_discriminant += 1;
                quote! { #lit }
            }
        }
    }
    pub fn write_token(
        &self,
        last_discriminant: &mut isize,
        max_discriminant: usize,
        span: Span,
    ) -> TokenStream {
        match self {
            Discriminant::Int(discriminant) => {
                let lit = LitInt::new(&format!("{}", discriminant), span);
                *last_discriminant = *discriminant as isize;
                quote_spanned! { span => #lit }
            }
            Discriminant::Wildcard => {
                let free_discriminant = max_discriminant + 1;
                let lit = LitInt::new(&format!("{}", free_discriminant), span);
                quote_spanned! { span => #lit }
            }
            Discriminant::Default => {
                let new_discriminant = (*last_discriminant + 1) as usize;
                let lit = LitInt::new(&format!("{}", new_discriminant), span);
                *last_discriminant += 1;
                quote_spanned! { span => #lit }
            }
        }
    }

    pub fn max_value(&self, last_discriminant: &mut isize) -> usize {
        match self {
            Discriminant::Int(discriminant) => {
                *last_discriminant = *discriminant as isize;
                *discriminant
            }
            Discriminant::Wildcard => 0,
            Discriminant::Default => {
                let new_discriminant = (*last_discriminant + 1) as usize;
                *last_discriminant += 1;
                new_discriminant
            }
        }
    }
}
