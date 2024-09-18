fn main(x: Field, y: pub Field) {
    assert(x != y);
}

#[requires(y > 5)]
fn foo(x: Field) -> Field {
  x + x
}
