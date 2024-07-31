struct Foo<T> {
}

impl <T> Foo<T> {
    fn new() -> Foo<T> {
        Foo {}
    }
}

fn main() {
    let foo = Foo::new();
}
