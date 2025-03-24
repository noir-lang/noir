struct Foo {}

impl Foo {
    fn new() -> Self {
        Self {}
    }
}

type FooAlias1 = Foo;
type FooAlias2 = FooAlias1;

fn main() {
    let _ = FooAlias2::new();
}
