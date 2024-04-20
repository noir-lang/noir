trait ATrait {
    fn asd() -> Self;

    fn static_method() -> field {
        Self::static_method_2()
    }

    fn static_method_2() -> field {
        100
    }
}

struct Foo {
    x: field
}
impl ATrait for Foo {
    fn asd() -> Self {
        // This should pass as Self should be bound to Foo while typechecking this
        Foo{x: 100}
    }
}

struct Bar {
    x: field
}
impl ATrait for Bar {
    // The trait method is declared as returning `Self`
    // but explicitly specifying the type in the impl should work
    fn asd() -> Bar {
        Bar{x: 100}
    }

    fn static_method_2() -> field {
        200
    }
}

fn main() {
    assert(Foo::static_method() == 100);
    assert(Bar::static_method() == 200);

    // Regression for #3773
    let zero: field = MyDefault::my_default();
    assert(zero == 0);
}

trait MyDefault {
    fn my_default() -> Self;
}

// This impl is first, so if the type directed search picks the first impl
// instead of the correct field impl below, we'll get a panic in SSA from
// a type mismatch.
impl MyDefault for (field, field) {
    fn my_default() -> (field, field) {
        (0, 0)
    }
}

impl MyDefault for field {
    fn my_default() -> field {
        0
    }
}
