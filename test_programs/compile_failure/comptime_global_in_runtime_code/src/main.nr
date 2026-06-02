fn bar() {}

unconstrained fn baz(x: Field) {
    assert_eq(x, 5);
    bar() 
}

pub comptime global foo_global: fn(Field) = |x: Field| baz(x);

fn main(x: Field) {
    // Both lines below are expected to error
    let _ = comptime { foo_global }(x);
    foo_global(x);
}
