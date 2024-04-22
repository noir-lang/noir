fn main(x: field, y: pub field) {
    let z = foo(x, y);
    let z2 = foo(x, y);
    assert(z == z2);
}

#[fold]
fn foo(x: field, y: field) -> field {
    assert(x != y);
    x
}
