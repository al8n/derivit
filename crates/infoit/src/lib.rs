#![cfg_attr(feature = "nightly", feature(const_type_name))]

pub use infoit_derive::Info;
use viewit::viewit;

#[viewit(setters(skip), getters(style = "move"))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FieldInfo {
  name: &'static str,
  ty: &'static str,
  vis: &'static str,
  tags: Tags,
  size: usize,
}

#[viewit(setters(skip), getters(style = "move"))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StructInfo {
  name: &'static str,
  vis: &'static str,
  fields: &'static [FieldInfo],
  #[cfg_attr(feature = "serde", serde(rename = "type"))]
  ty: &'static str,
  size: usize,
  tags: Tags,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Tags {
  tags: &'static [(&'static str, &'static str)],
}

#[cfg(feature = "serde")]
impl serde::Serialize for Tags {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeMap;
    let mut map = serializer.serialize_map(Some(self.tags.len()))?;
    for (k, v) in self.tags {
      map.serialize_entry(k, v)?;
    }
    map.end()
  }
}

impl Tags {
  #[inline]
  pub const fn new(tags: &'static [(&'static str, &'static str)]) -> Self {
    Self { tags }
  }

  #[inline]
  pub fn iter(&self) -> core::slice::Iter<'_, (&'static str, &'static str)> {
    self.tags.iter()
  }

  #[inline]
  pub fn keys(&self) -> impl Iterator<Item = &'static str> {
    self.tags.iter().map(|(k, _)| *k)
  }

  #[inline]
  pub fn values(&self) -> impl Iterator<Item = &'static str> {
    self.tags.iter().map(|(_, v)| *v)
  }

  #[inline]
  pub fn get(&self, key: &'static str) -> Option<&'static str> {
    self
      .tags
      .iter()
      .find_map(|(k, v)| if *k == key { Some(*v) } else { None })
  }
}
