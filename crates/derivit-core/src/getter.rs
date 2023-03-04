use darling::FromMeta;
use quote::{quote, ToTokens};

#[derive(Default, FromMeta, Clone, Copy)]
pub enum Style {
  #[darling(rename = "ref")]
  Ref,
  #[darling(rename = "move")]
  #[default]
  Move,
}

impl ToTokens for Style {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    match self {
      Style::Ref => tokens.extend(quote! { & }),
      Style::Move => tokens.extend(quote! {}),
    }
  }
}

#[derive(Default, FromMeta, Clone)]
pub struct FieldConverter {
  pub style: Option<Style>,
  #[darling(rename = "fn")]
  pub func: Option<syn::Path>,
}

#[derive(Default, FromMeta)]
pub struct FieldGetterOptions {
  pub rename: Option<syn::Ident>,
  pub style: Option<Style>,
  #[darling(default, rename = "skip")]
  pub ignore: bool,
  pub vis: Option<syn::Visibility>,
  pub result: Option<GetterConverter>,
}

#[derive(FromMeta)]
pub struct StructGetterOptions {
  pub prefix: Option<syn::Ident>,
  #[darling(default)]
  pub style: Style,
  #[darling(default, rename = "skip")]
  pub ignore: bool,
  pub vis_all: Option<syn::Visibility>,
}

impl Default for StructGetterOptions {
  fn default() -> Self {
    Self {
      prefix: None,
      style: Style::Ref,
      ignore: false,
      vis_all: None,
    }
  }
}

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

#[derive(FromMeta, Clone)]
pub struct GetterConverter {
  #[darling(rename = "type")]
  pub ty: Option<syn::Type>,
  pub converter: FieldConverter,
  #[darling(default)]
  pub bound: FnGenerics,
}

impl GetterConverter {
  pub fn to_getter_fn(
    &self,
    field_name: &syn::Ident,
    field_ty: &syn::Type,
    style: Style,
    vis: &syn::Visibility,
    fn_name: &syn::Ident,
  ) -> proc_macro2::TokenStream {
    let field_ty = self.ty.as_ref().unwrap_or(field_ty);
    let bound = self.bound.bound.as_ref();
    let result = match self.converter.style.unwrap_or(style) {
      Style::Ref => match &self.converter.func {
        Some(conv) => quote! {
          #conv(&self.#field_name)
        },
        None => quote! {
          &self.#field_name
        },
      },
      Style::Move => match &self.converter.func {
        Some(conv) => quote! {
          #conv(self.#field_name)
        },
        None => quote! {
          self.#field_name
        },
      },
    };
    match style {
      Style::Ref => quote! {
        #[inline]
        #vis fn #fn_name #bound (&self) -> #field_ty {
          #result
        }
      },
      Style::Move => quote! {
        #[inline]
        #vis fn #fn_name #bound (self) ->  {
          #result
        }
      },
    }
  }
}

pub struct FieldGetter {
  pub field_name: syn::Ident,
  pub field_ty: syn::Type,
  pub style: Style,
  pub vis: syn::Visibility,
  pub fn_name: syn::Ident,
  pub converter: Option<GetterConverter>,
}

impl ToTokens for FieldGetter {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    let vis = &self.vis;
    let fn_name = &self.fn_name;
    let field_name = &self.field_name;
    let field_ty = self
      .converter
      .as_ref()
      .map(|conv| conv.ty.as_ref().unwrap_or(&self.field_ty))
      .unwrap_or(&self.field_ty);

    match &self.converter {
      Some(converter) => {
        let bound = converter.bound.bound.as_ref();
        let result = match self.style {
          Style::Ref => match &converter.converter.func {
            Some(conv) => quote! {
              #conv(&self.#field_name)
            },
            None => quote! {
              &self.#field_name
            },
          },
          Style::Move => match &converter.converter.func {
            Some(conv) => quote! {
              #conv(self.#field_name)
            },
            None => quote! {
              self.#field_name
            },
          },
        };

        tokens.extend(match self.style {
          Style::Ref => quote! {
            #[inline]
            #vis fn #fn_name #bound (&self) -> #field_ty {
              #result
            }
          },
          Style::Move => quote! {
            #[inline]
            #vis fn #fn_name #bound (self) ->  {
              #result
            }
          },
        });
      }
      None => {
        let style = self.style;
        tokens.extend(quote! {
            #[inline]
            #vis fn #fn_name(&self) -> #style #field_ty {
              #style self.#field_name
            }
        });
      }
    }
  }
}
