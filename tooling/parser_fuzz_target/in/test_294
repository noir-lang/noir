fn main(x: Field, y: pub Field) {
    let z = func_with_nested_foo_call(x, y);
    let z2 = func_with_nested_foo_call(x, y);
    assert(z == z2);
}

#[fold]
fn func_with_nested_foo_call(x: Field, y: Field) -> Field {
    foo(x + 2, y)
}

#[fold]
fn foo(x: Field, y: Field) -> Field {
    assert(x != y);
    x
}
