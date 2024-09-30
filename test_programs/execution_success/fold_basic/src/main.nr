fn main(x: Field, y: pub Field) {
    let z = foo(x, y);
    let z2 = foo(x, y);
    let z3 = foo(x, y);
    assert(z == z2);
    assert(z2 == z3);
}

#[fold]
fn foo(x: Field, y: Field) -> Field {
    assert(x != y);
    x
}
