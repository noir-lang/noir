
        trait Default2 {
            fn default2() -> Self;
        }

        impl Default2 for Field {
            fn default2() -> Self { 0 }
        }

        // Need the above as we don't have access to the stdlib here.
        // We also need to construct a concrete value of `U` without giving away its type
        // as otherwise the unspecified type is ignored.

        struct Foo<T> {
            inner: T
        }

        impl<T> Foo<T> {
            fn generic_method<U>(_self: Self) -> U where U: Default2 {
                U::default2()
            }
        }

        fn main() {
            let foo: Foo<Field> = Foo { inner: 1 };
            let _ = foo.generic_method::<Field>();
        }
    