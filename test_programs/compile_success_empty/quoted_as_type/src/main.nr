fn main() {
    macro!().do_nothing();
}

comptime fn macro() -> Quoted {
    let typ = quote { Foo<Field> }.as_type();
    quote { let foo: $typ = Foo {}; foo }
}

struct Foo<T> {}

// Ensure we call the Foo<Field> impl
impl Foo<u32> {
    fn do_nothing(self) {
        assert(false);
    }
}

impl Foo<Field> {
    fn do_nothing(self) {}
}
