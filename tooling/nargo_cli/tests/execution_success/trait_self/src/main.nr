trait ATrait {
    fn asd() -> Self;

    fn static_method() -> Field {
        Self::static_method_2()
    }

    fn static_method_2() -> Field {
        100
    }
}

struct Foo {
    x: Field
}
impl ATrait for Foo {
    fn asd() -> Self {
        // This should pass as Self should be bound to Foo while typechecking this
        Foo{x: 100}
    }
}

struct Bar {
    x: Field
}
impl ATrait for Bar {
    // The trait method is declared as returning `Self`
    // but explicitly specifying the type in the impl should work
    fn asd() -> Bar {
        Bar{x: 100}
    }

    fn static_method_2() -> Field {
        200
    }
}

fn main() {
    assert(Foo::static_method() == 100);
    assert(Bar::static_method() == 200);
}