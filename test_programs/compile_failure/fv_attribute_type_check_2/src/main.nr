fn main(x: Field, y: pub Field) {
    assert(x != y);
}

#[ensures(result)]
fn foo(x: Field) -> Field {
  x + x
}

