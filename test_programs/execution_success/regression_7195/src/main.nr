fn bar(y: Field) {
    assert(y != 0);
}

fn foo(x: Field) {
    /// Safety: test
    let y = unsafe { baz(x) };
    bar(y);
}

unconstrained fn baz(x: Field) -> Field {
    x
}

fn main(x: Field, y: pub Field) {
    /// Safety: test
    let x = unsafe { baz(x) };
    foo(x);
    foo(y);
    assert(x != y);
}
