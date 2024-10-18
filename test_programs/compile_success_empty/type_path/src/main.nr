fn main() {
    comptime
    {
        let foo = quote { Foo }.as_type();
        quote {
            $foo::static()
        }
    }
}

pub struct Foo {}

impl Foo {
    fn static() {}
}
