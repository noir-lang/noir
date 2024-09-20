fn main() {}

#[requires(x as u32 > 5)] // as u32 because fields can not be compared
fn foo(x: Field) -> Field {
  x + x
}
