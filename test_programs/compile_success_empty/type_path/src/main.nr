fn main() {
    comptime
    {
        let foo = quote { Foo }.as_type();
        quote {
            $foo::static()
        }
    }
}

struct Foo {}

impl Foo {
    fn static() {
    }
}
