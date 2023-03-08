use viewit::viewit;

struct FromString {
  src: String,
}

impl From<&String> for FromString {
  fn from(src: &String) -> Self {
    Self { src: src.clone() }
  }
}


fn vec_to_string(src: &[u8]) -> String {
  String::from_utf8_lossy(src).to_string()
}

#[viewit(
  // set this, then this visibility will be applied to the fields
  vis_all = "pub(crate)",
  // this will not generate setters
  setters(
    // change the prefix for all setters
    prefix = "with",
    // change the setters fn style, available values here are ref, into, tryinto or move
    style = "ref",
    // if you do not want to generate getters, you can use skip
    // skip, 
  ),
  getters(
    // change the prefix for all getters
    prefix = "get",
    // change the getters fn style, available values here are ref and move
    style = "ref",
    // if you do not want to generate getters, you can use skip
    // skip,
  ),
  // print the generated code to std out, other available values here are: stderr or "path/to/output/file"
  debug = "stdout"
)]
struct Foo {
  #[viewit(
    getter(
      style = "move",
      rename = "get_first_field",
      vis = "pub" // we do not want the getter for the first field is public, then we can custom field getter
    ),
    setter(
      skip, // we do not want the setter for the field, then we skip it.
    )
  )]
  f1: u8,
  #[viewit(
    getter(
      skip, // we do not want the getter for the field, then we skip it
    )
  )]
  f2: u16,

  #[viewit(
    getter(
      result(
        // sometimes we may want to convert the f4 field to a generic type
        type = "T",
        converter(
          style = "ref", // take the ownership of the field
          fn = "T::from", // the fn used to do the conversion
        ),
        // set the trait bound
        bound = "T: for<'a> From<&'a String>"
      )
    )
  )]
  f3: String,

  #[viewit(
    getter(
      result(
        // we want to convert the f3 field to String
        type = "String",
        converter(
          style = "ref", // take the reference of the field
          fn = "vec_to_string" // the fn used to do the conversion
        ),
      )
    )
  )]
  f4: Vec<u8>,
}


fn main() {}