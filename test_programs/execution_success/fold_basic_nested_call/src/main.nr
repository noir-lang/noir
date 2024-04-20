fn main(x: field, y: pub field) {
    let z = func_with_nested_foo_call(x, y);
    let z2 = func_with_nested_foo_call(x, y);
    assert(z == z2);
}

#[fold]
fn func_with_nested_foo_call(x: field, y: field) -> field {
    foo(x + 2, y)
}

#[fold]
fn foo(x: field, y: field) -> field {
    assert(x != y);
    x
}
