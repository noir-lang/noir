struct Foo {
    x: Field
}

trait Asd {
    fn asd() -> Self;
}

impl Asd for Foo {
    // the Self should typecheck properly
    fn asd() -> Self {
        Foo{x: 100}
    }
}

fn main() {
    assert(Foo::asd().x == 100);
}