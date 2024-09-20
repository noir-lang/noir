fn main() {}

#[ensures(result as u32 > 5)]  // result can be only used in the ensures attribute
fn foo(x: Field) -> Field {
  x + x
}
