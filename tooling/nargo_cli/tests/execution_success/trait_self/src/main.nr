trait ATrait {
    fn asd() -> Self;
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
}

fn main() {
}