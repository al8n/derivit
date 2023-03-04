use super::FnGenerics;
use darling::FromMeta;
use quote::{format_ident, quote, ToTokens};

#[derive(Default, FromMeta)]
pub struct FieldSetterOptions {
  pub rename: Option<syn::Ident>,
  pub style: Option<SetterStyle>,
  #[darling(default, rename = "skip")]
  pub ignore: bool,
  pub vis: Option<syn::Visibility>,
  #[darling(default)]
  pub bound: FnGenerics,
}

#[derive(FromMeta)]
pub struct StructSetterOptions {
  pub prefix: Option<syn::Ident>,
  #[darling(default)]
  pub style: SetterStyle,
  #[darling(default, rename = "skip")]
  pub ignore: bool,
  pub vis_all: Option<syn::Visibility>,
}

impl Default for StructSetterOptions {
  fn default() -> Self {
    Self {
      prefix: Some(format_ident!("set")),
      style: SetterStyle::Move,
      ignore: false,
      vis_all: None,
    }
  }
}

#[derive(Default, FromMeta, Clone, Copy)]
pub enum SetterStyle {
  #[darling(rename = "ref")]
  Ref,
  #[darling(rename = "move")]
  #[default]
  Move,
  #[darling(rename = "into")]
  Into,
  #[darling(rename = "try_into")]
  TryInto,
}

impl SetterStyle {
  fn to_setter(
    &self,
    fn_vis: &syn::Visibility,
    bound: Option<&syn::Generics>,
    field_name: &syn::Ident,
    field_ty: &syn::Type,
    fn_name: &syn::Ident,
  ) -> proc_macro2::TokenStream {
    match self {
      Self::Ref => quote! {
        #[inline]
        #fn_vis fn #fn_name #bound (&mut self, val: #field_ty) {
          self.#field_name = val;
        }

      },
      Self::Move => quote! {
        #[inline]
        #fn_vis fn #fn_name #bound (mut self, val: #field_ty) -> Self {
          self.#field_name = val;
          self
        }

      },
      Self::Into => quote! {
        #[inline]
        #fn_vis fn #fn_name #bound (mut self, val: impl core::convert::Into<#field_ty>) -> Self {
          self.#field_name = ::core::convert::Into::into(val);
          self
        }

      },
      Self::TryInto => {
        let bound = bound.map(|tt| {
          let bound = format!(
            "{}, Error>",
            tt.to_token_stream().to_string().trim_end_matches('>')
          );
          syn::parse_str::<syn::Generics>(&bound).unwrap()
        });
        quote! {
          #[inline]
          #fn_vis fn #fn_name #bound (mut self, val: impl ::core::convert::TryInto<#field_ty, Error = Error>) -> ::core::result::Result<Self, Error> {
            self.#field_name = ::core::convert::TryInto::try_into(val)?;
            ::core::result::Result::Ok(self)
          }

        }
      }
    }
  }
}

pub struct FieldSetter {
  pub vis: syn::Visibility,
  pub bound: Option<syn::Generics>,
  pub field_name: syn::Ident,
  pub field_ty: syn::Type,
  pub fn_name: syn::Ident,
  pub style: SetterStyle,
}

impl ToTokens for FieldSetter {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    let fn_vis = &self.vis;
    let bound = self.bound.as_ref();
    let field_name = &self.field_name;
    let field_ty = &self.field_ty;
    let fn_name = &self.fn_name;
    let style = &self.style;
    tokens.extend(style.to_setter(fn_vis, bound, field_name, field_ty, fn_name));
  }
}
