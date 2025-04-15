
        pub struct Foo {
            wrapped: fn(Field) -> bool,
        }

        impl Foo {
            fn call(self) -> bool {
                self.wrapped(1)
            }
        }

        fn main() {}
    