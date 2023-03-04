#![allow(clippy::wrong_self_convention)]

pub mod getter;
pub mod parser;
pub mod setter;

#[derive(Default, Clone)]
pub struct FnGenerics {
  pub bound: Option<syn::Generics>,
}

impl darling::FromMeta for FnGenerics {
  fn from_value(value: &syn::Lit) -> darling::Result<Self> {
    if let syn::Lit::Str(ref s) = value {
      if s.value().is_empty() {
        Ok(Self { bound: None })
      } else {
        let tt = format!("<{}>", s.value());
        let bound = syn::parse_str::<syn::Generics>(&tt)?;
        Ok(Self { bound: Some(bound) })
      }
    } else {
      Err(darling::Error::custom("expected str literal").with_span(value))
    }
  }
}
