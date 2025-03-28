fn main() {
    comptime {
        let foo = quote { Foo }.as_type();
        quote {
            $foo::static()
        }
    }

    // Make sure this call works fine: in the past we used to not distinguish
    // whether a TypePath had generics or not, always resolved them, filling them
    // up with Type::Error, and eventually leading to an ICE.
    let _ = Field::from_be_bytes([1]);
}

pub struct Foo {}

impl Foo {
    fn static() {}
}
