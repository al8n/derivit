use darling::{FromDeriveInput, FromField, FromMeta, ToTokens};
use indexmap::IndexMap;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(FromDeriveInput)]
#[darling(attributes(info))]
struct Info {
  ident: syn::Ident,
  vis: syn::Visibility,
  #[darling(rename = "vis")]
  vis_: Option<syn::Visibility>,
  #[darling(default)]
  tags: Tags,
  debug: Option<derivit_core::Debug>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Tags {
  #[serde(flatten)]
  map: IndexMap<String, String>,
}

impl ToTokens for Tags {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    let mut streams = Vec::new();
    for (k, v) in self.map.iter() {
      streams.push(quote! { (#k, #v) });
    }
    tokens.extend(quote! { #(#streams),* });
  }
}

impl FromMeta for Tags {
  fn from_list(items: &[syn::NestedMeta]) -> darling::Result<Self> {
    let mut map = IndexMap::new();
    for item in items.iter() {
      match item {
        syn::NestedMeta::Meta(inner) => {
          let key = inner
            .path()
            .get_ident()
            .ok_or_else(|| darling::Error::custom("missing ident").with_span(inner))?
            .to_string();
          let value = match inner {
            syn::Meta::NameValue(inner) => match &inner.lit {
              syn::Lit::Str(v) => v.value(),
              syn::Lit::ByteStr(bs) => String::from_utf8_lossy(&bs.value()).to_string(),
              syn::Lit::Byte(b) => b.value().to_string(),
              syn::Lit::Char(c) => c.value().to_string(),
              syn::Lit::Int(v) => v.base10_digits().to_string(),
              syn::Lit::Float(f) => f.base10_digits().to_string(),
              syn::Lit::Bool(b) => b.value.to_string(),
              syn::Lit::Verbatim(v) => v.to_string(),
            },
            syn::Meta::Path(path) => path
              .get_ident()
              .ok_or_else(|| darling::Error::custom("missing ident").with_span(path))?
              .to_string(),
            syn::Meta::List(l) => {
              let this = Self::from_list(&l.nested.iter().cloned().collect::<Vec<_>>())?;
              serde_json::to_string(&this)
                .map_err(|e| darling::Error::custom(e.to_string()).with_span(l))?
            }
          };
          map.insert(key, value);
        }
        syn::NestedMeta::Lit(inner) => {
          return Err(darling::Error::unsupported_format("literal").with_span(inner))
        }
      }
    }
    Ok(Self { map })
  }
}

#[derive(FromField)]
struct InfoField {
  ident: Option<syn::Ident>,
  ty: syn::Type,
  #[darling(default)]
  tags: Tags,
}

#[proc_macro_derive(Info, attributes(info))]
pub fn info(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
  let info = match Info::from_derive_input(&input) {
    Ok(info) => info,
    Err(e) => return e.write_errors().into(),
  };
  let name = &info.ident;
  let name_str = name.to_string();
  let vis = info.vis_.as_ref().unwrap_or(&info.vis);
  let vis_str = vis.to_token_stream().to_string();
  let tags = &info.tags;

  match &input.data {
    syn::Data::Struct(data) => {
      let mut fields = Vec::new();
      let mut ctr = 0;
      for f in &data.fields {
        let info_field = match InfoField::from_field(f) {
          Ok(info_field) => info_field,
          Err(e) => return e.write_errors().into(),
        };
        let ty_str = get_type_name(&info_field.ty);
        let ty = &info_field.ty;
        let tags = &info_field.tags;
        let vis_str = f.vis.to_token_stream().to_string();
        fields.push(match &info_field.ident {
          Some(name) => {
            let name_str = name.to_string();
            quote! {
              ::infoit::FieldInfo {
                name: #name_str,
                ty: #ty_str,
                tags: ::infoit::Tags::new(&[#tags]),
                vis: #vis_str,
                size: ::core::mem::size_of::<#ty>(),
              }
            }
          }
          None => {
            let name_str = ctr.to_string();
            ctr += 1;
            quote! {
              ::infoit::FieldInfo {
                name: #name_str,
                ty: #ty_str,
                tags: ::infoit::Tags::new(&[#tags]),
                vis: #vis_str,
                size: ::core::mem::size_of::<#ty>(),
              }
            }
          }
        });
      }
      let ty = if data.fields.is_empty() {
        "unit"
      } else if data.fields.iter().next().unwrap().ident.is_some() {
        "tuple"
      } else {
        "struct"
      };

      let ts = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
          #vis const INFO: ::infoit::StructInfo = ::infoit::StructInfo {
            name: #name_str,
            fields: &[#(#fields),*],
            size: ::core::mem::size_of::<Self>(),
            tags: ::infoit::Tags::new(&[#tags]),
            vis: #vis_str,
            ty: #ty,
          };
        }
      };

      if let Some(debug) = &info.debug {
        match debug.write(&ts) {
          Ok(_) => {}
          Err(e) => return e.to_compile_error().into(),
        }
      }

      ts.into()
    }
    syn::Data::Enum(_data) => syn::Error::new_spanned(
      input,
      "enums are not supported currently, will be supported in future versions",
    )
    .to_compile_error()
    .into(),
    syn::Data::Union(data) => syn::Error::new_spanned(data.union_token, "unions are not supported")
      .to_compile_error()
      .into(),
  }
}

#[inline]
fn get_type_name(ty: &syn::Type) -> proc_macro2::TokenStream {
  // TODO: remove this when feature(const_type_name) stable
  // #[cfg(feature = "nightly")]
  // {
  //   quote! { ::core::any::type_name::<#ty>() }
  // }
  let ty_str = ty.to_token_stream().to_string();
  quote! { #ty_str }
}
