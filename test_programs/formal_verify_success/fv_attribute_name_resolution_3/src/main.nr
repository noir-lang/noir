fn main() {}

#[ensures(result == x + x)]
fn foo(x: Field) -> Field {
  x + x
}
