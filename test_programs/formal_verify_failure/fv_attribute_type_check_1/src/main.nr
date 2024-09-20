fn main(x: Field, y: pub Field) {
    assert(x != y);
}

#[requires(x)]
fn foo(x: Field) -> Field {
  x + x
}

