fn main(x: Field, y: pub Field) {
    assert(x != y);
}

#[requires(result > 5)]  // result can be only used in the ensures attribute
fn foo(x: Field) -> Field {
  x + x
}
