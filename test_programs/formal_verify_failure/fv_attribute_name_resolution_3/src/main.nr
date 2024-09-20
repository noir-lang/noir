fn main(x: Field, y: pub Field) {
    assert(x != y);
}

#[ensures(result > y)]
fn foo(x: Field) -> Field {
  x + x
}
