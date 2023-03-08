use darling::{FromDeriveInput, FromField, FromMeta};
use derivit_core::{
  getter::{FieldGetter, FieldGetterOptions, StructGetterOptions},
  setter::{FieldSetter, FieldSetterOptions, StructSetterOptions},
};
use quote::{format_ident, quote};
use syn::parse_macro_input;

trait ViewIt {
  fn vis_all(&self) -> Option<&syn::Visibility>;
  fn setter(&self) -> &StructSetterOptions;
  fn getter(&self) -> &StructGetterOptions;
}

#[derive(FromDeriveInput)]
#[darling(attributes(view), supports(struct_named))]
struct ViewItDerive {
  vis: syn::Visibility,
  #[darling(default, rename = "setters")]
  setter: StructSetterOptions,
  #[darling(default, rename = "getters")]
  getter: StructGetterOptions,
  debug: Option<derivit_core::Debug>,
}

impl ViewIt for ViewItDerive {
  fn vis_all(&self) -> Option<&syn::Visibility> {
    Some(&self.vis)
  }
  fn setter(&self) -> &StructSetterOptions {
    &self.setter
  }
  fn getter(&self) -> &StructGetterOptions {
    &self.getter
  }
}

struct ViewItAttribute {
  vis_all: Option<syn::Visibility>,
  setter: StructSetterOptions,
  getter: StructGetterOptions,
  debug: Option<derivit_core::Debug>,
}

impl ViewIt for ViewItAttribute {
  fn vis_all(&self) -> Option<&syn::Visibility> {
    self.vis_all.as_ref()
  }
  fn setter(&self) -> &StructSetterOptions {
    &self.setter
  }
  fn getter(&self) -> &StructGetterOptions {
    &self.getter
  }
}

impl FromMeta for ViewItAttribute {
  fn from_list(items: &[syn::NestedMeta]) -> darling::Result<Self> {
    let mut vis_all: (bool, Option<syn::Visibility>) = (false, None);
    let mut getters = (false, None);
    let mut setters = (false, None);
    let mut debug = (false, None);

    for item in items {
      match item {
        syn::NestedMeta::Meta(inner) => {
          let name = darling::util::path_to_string(inner.path());
          match name.as_str() {
            "vis_all" => derivit_core::parser::Parser::parse(&name, inner, &mut vis_all)?,
            "setters" => derivit_core::parser::Parser::parse(&name, inner, &mut setters)?,
            "getters" => derivit_core::parser::Parser::parse(&name, inner, &mut getters)?,
            "debug" => derivit_core::parser::Parser::parse(&name, inner, &mut debug)?,
            other => {
              return Err(
                darling::Error::unknown_field_with_alts(other, &["getters", "setters", "vis_all"])
                  .with_span(inner),
              );
            }
          }
        }
        syn::NestedMeta::Lit(inner) => {
          return Err(darling::Error::unsupported_format("literal").with_span(inner))
        }
      }
    }

    Ok(Self {
      vis_all: vis_all.1,
      setter: setters.1.unwrap_or_default(),
      getter: getters.1.unwrap_or_default(),
      debug: debug.1,
    })
  }
}

#[derive(FromField)]
#[darling(attributes(viewit))]
struct ViewField {
  #[darling(rename = "vis")]
  vis_: Option<syn::Visibility>,
  #[darling(default)]
  getter: FieldGetterOptions,
  #[darling(default)]
  setter: FieldSetterOptions,
}

fn handle_fields<'a>(
  viewit: &impl ViewIt,
  fields: impl Iterator<Item = &'a mut syn::Field>,
) -> darling::Result<(Vec<syn::Field>, Vec<FieldGetter>, Vec<FieldSetter>)> {
  let mut struct_fields = Vec::new();
  let mut struct_getters = Vec::new();
  let mut struct_setters = Vec::new();
  for f in fields {
    let field_name = f.ident.as_ref().unwrap();
    let field = match ViewField::from_field(f) {
      Ok(field) => field,
      Err(e) => return Err(e),
    };

    match (viewit.getter().ignore, field.getter.ignore) {
      (true, true) | (false, true) | (true, false) => {}
      (false, false) => {
        let vis = field.getter.vis.as_ref().unwrap_or_else(|| {
          viewit
            .getter()
            .vis_all
            .as_ref()
            .unwrap_or_else(|| viewit.vis_all().unwrap_or(&f.vis))
        });
        let fn_name = field.getter.rename.clone().unwrap_or_else(|| {
          if let Some(p) = &viewit.getter().prefix {
            format_ident!("{}_{}", p, field_name)
          } else {
            field_name.clone()
          }
        });

        let style = field.getter.style.unwrap_or(viewit.getter().style);
        struct_getters.push(FieldGetter {
          field_name: field_name.clone(),
          field_ty: f.ty.clone(),
          style,
          vis: vis.clone(),
          fn_name,
          converter: field.getter.result.clone(),
        });
      }
    }

    match (viewit.setter().ignore, field.setter.ignore) {
      (true, true) | (false, true) | (true, false) => {}
      (false, false) => {
        let vis = field.setter.vis.as_ref().unwrap_or_else(|| {
          viewit
            .setter()
            .vis_all
            .as_ref()
            .unwrap_or_else(|| viewit.vis_all().unwrap_or(&f.vis))
        });
        let fn_name = field.setter.rename.clone().unwrap_or_else(|| {
          format_ident!(
            "{}_{}",
            viewit
              .setter()
              .prefix
              .clone()
              .unwrap_or_else(|| format_ident!("set")),
            field_name
          )
        });

        let style = field.setter.style.unwrap_or(viewit.setter().style);
        struct_setters.push(FieldSetter {
          field_name: field_name.clone(),
          field_ty: f.ty.clone(),
          style,
          vis: vis.clone(),
          fn_name,
          bound: field.setter.bound.bound.clone(),
        });
      }
    }

    f.attrs.retain(|x| !x.path.is_ident("viewit"));
    f.vis = viewit
      .vis_all()
      .unwrap_or_else(|| field.vis_.as_ref().unwrap_or(&f.vis))
      .clone();
    struct_fields.push(f.clone());
  }

  Ok((struct_fields, struct_getters, struct_setters))
}

#[proc_macro_derive(View, attributes(view))]
pub fn view(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let mut input = parse_macro_input!(input as syn::DeriveInput);
  let viewit = match ViewItDerive::from_derive_input(&input) {
    Ok(viewit) => viewit,
    Err(e) => return e.write_errors().into(),
  };

  let data = if let syn::Data::Struct(data) = &mut input.data {
    data
  } else {
    return syn::Error::new_spanned(input, "expected struct")
      .to_compile_error()
      .into();
  };

  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
  let name = &input.ident;
  match &mut data.fields {
    syn::Fields::Named(fields) => {
      let (_, struct_getters, struct_setters) =
        match handle_fields(&viewit, fields.named.iter_mut()) {
          Ok(x) => x,
          Err(e) => return e.write_errors().into(),
        };

      let ts = quote! {
        impl #impl_generics #name #ty_generics #where_clause {

          #(#struct_getters)*

          #(#struct_setters)*
        }
      };
      if let Some(ref debug) = viewit.debug {
        if let Err(e) = debug.write(&ts) {
          return e.to_compile_error().into();
        }
      }

      ts.into()
    }
    _ => unreachable!(),
  }
}

#[proc_macro_attribute]
pub fn viewit(
  args: proc_macro::TokenStream,
  input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let mut input = parse_macro_input!(input as syn::DeriveInput);
  let args = parse_macro_input!(args as syn::AttributeArgs);
  let struct_attrs = &input.attrs;
  let mut viewit = match ViewItAttribute::from_list(&args) {
    Ok(viewit) => viewit,
    Err(e) => return e.write_errors().into(),
  };
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
  let generics = &input.generics;
  let name = &input.ident;
  let vis = &input.vis;

  // by default, apply struct visibility to fields.
  viewit.vis_all.get_or_insert(vis.clone());

  let data = if let syn::Data::Struct(data) = &mut input.data {
    data
  } else {
    return syn::Error::new_spanned(input, "expected struct")
      .to_compile_error()
      .into();
  };

  match &mut data.fields {
    syn::Fields::Named(fields) => {
      let (struct_fields, struct_getters, struct_setters) =
        match handle_fields(&viewit, fields.named.iter_mut()) {
          Ok(x) => x,
          Err(e) => return e.write_errors().into(),
        };

      let ts = quote! {
        #(#struct_attrs)*
        #vis struct #name #generics {
          #(#struct_fields),*
        }

        impl #impl_generics #name #ty_generics #where_clause {

          #(#struct_getters)*

          #(#struct_setters)*
        }
      };

      if let Some(ref debug) = viewit.debug {
        if let Err(e) = debug.write(&ts) {
          return e.to_compile_error().into();
        }
      }

      ts.into()
    }
    syn::Fields::Unnamed(fields) => {
      syn::Error::new_spanned(fields, "tuple structs are not supported")
        .to_compile_error()
        .into()
    }
    syn::Fields::Unit => quote! {
      #(#struct_attrs)*
      #vis struct #name #generics;
    }
    .into(),
  }
}
