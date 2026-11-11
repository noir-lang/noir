struct Foo {
    x: Field,
}

struct Holder {
    foo: &mut Foo,
}

fn main(x: Field) {
    check(x);
    // Safety: runs the same checks in Brillig
    unsafe { check_unconstrained(x) };
}

unconstrained fn check_unconstrained(x: Field) {
    check(x);
}

fn check(x: Field) {
    // A field borrowed through two mutable references
    let mut inner = &mut Foo { x };
    let outer = &mut inner;
    let m: &mut Field = &mut outer.x;
    *m = x + 1;
    let r: &Field = &outer.x;
    assert_eq(*r, x + 1);
    assert_eq(inner.x, x + 1);

    // A field of a reference stored in a struct field
    let holder = &mut Holder { foo: &mut Foo { x } };
    let m: &mut Field = &mut holder.foo.x;
    *m = x + 2;
    assert_eq(holder.foo.x, x + 2);

    // An immutable borrow of a field of a mutable reference sees later writes
    let foo = &mut Foo { x };
    let r: &Field = &foo.x;
    foo.x = x + 3;
    assert_eq(*r, x + 3);
}
