struct Foo<T> {}

impl<T> Foo<T> {
    fn new() -> Self {
        Self {}
    }
}

type FooAlias1 = Foo<i32>;
type FooAlias2 = FooAlias1;

fn main() {
    let _ = FooAlias2::new();
}
