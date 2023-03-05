use infoit::Info;

#[derive(Debug, Clone, Info)]
#[info(tags(foo = "bar", size = 1, bytes = b"hello", json(k1 = "1", k2 = "2")))]
pub(crate) struct MyView {
  foo: u64,
  bar: u32,
}

#[derive(Debug, Clone, Info)]
pub(crate) struct MyTupleView(u64, u32);

fn main() {}
