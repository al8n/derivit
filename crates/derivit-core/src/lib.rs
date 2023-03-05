#![allow(clippy::wrong_self_convention)]

use std::path::PathBuf;

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

#[derive(Default, Clone)]
pub enum DebugOutput {
  #[default]
  StdOut,
  StdErr,
  File(PathBuf),
}

#[derive(Default, Clone)]
pub struct Debug {
  pub debug: DebugOutput,
}

impl From<DebugOutput> for Debug {
  fn from(debug: DebugOutput) -> Self {
    Self { debug }
  }
}

impl darling::FromMeta for Debug {
  fn from_meta(item: &syn::Meta) -> darling::Result<Self> {
    match item {
      syn::Meta::Path(_) => Ok(Self::default()),
      syn::Meta::List(l) => Err(
        darling::Error::custom(
          "expected path or name value pair, but found attribute list for debug",
        )
        .with_span(l),
      ),
      syn::Meta::NameValue(val) => {
        if let syn::Lit::Str(val) = &val.lit {
          match val.value().as_str() {
            "stdout" | "out" => Ok(Self::from(DebugOutput::StdOut)),
            "stderr" | "err" | "error" => Ok(Self::from(DebugOutput::StdErr)),
            file => Ok(Self::from(DebugOutput::File(PathBuf::from(file)))),
          }
        } else {
          Err(darling::Error::custom("expected str literal").with_span(val))
        }
      }
    }
  }
}

impl Debug {
  pub fn write(&self, ts: &proc_macro2::TokenStream) -> syn::Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;
    syn::parse_file(&ts.to_string()).and_then(|file| {
      let src = prettyplease::unparse(&file);
      match &self.debug {
        DebugOutput::StdOut => {
          println!("{src}");
          Ok(())
        }
        DebugOutput::StdErr => {
          eprintln!("{src}");
          Ok(())
        }
        DebugOutput::File(path) => {
          let mut opts = OpenOptions::new();
          opts
            .write(true)
            .create(true)
            .truncate(true)
            .read(true)
            .open(path)
            .and_then(|mut file| {
              file.write_all(src.as_bytes())?;
              file.flush()
            })
            .map_err(|e| syn::Error::new_spanned(ts, e.to_string()))
        }
      }
    })
  }
}
