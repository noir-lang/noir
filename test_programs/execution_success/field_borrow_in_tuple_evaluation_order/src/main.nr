struct Foo {
    x: Field,
}

// A field borrow through a reference is taken when it is evaluated, so a later tuple element
// that repoints the reference must not change which field was borrowed.
unconstrained fn main(x: Field) {
    let mut a = Foo { x };
    let mut b = Foo { x: x + 9 };
    let mut r = &mut a;
    let t = (
        &mut r.x, {
            r = &mut b;
            0
        },
    );
    *t.0 = x + 4;
    assert_eq(a.x, x + 4);
    assert_eq(b.x, x + 9);

    let mut c = Foo { x };
    let mut d = Foo { x: x + 9 };
    let mut inner = &mut c;
    let outer = &mut inner;
    let t = (
        &mut (*outer).x, {
            *outer = &mut d;
            0
        },
    );
    *t.0 = x + 4;
    assert_eq(c.x, x + 4);
    assert_eq(d.x, x + 9);
}
