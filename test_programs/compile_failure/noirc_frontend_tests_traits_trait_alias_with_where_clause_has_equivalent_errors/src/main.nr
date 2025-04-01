
        trait Bar {
            fn bar(self) -> Self;
        }

        trait Baz {
            fn baz(self) -> bool;
        }

        trait Qux<T>: Bar where T: Baz {}

        impl<T, U> Qux<T> for U where
            U: Bar,
            T: Baz,
        {}

        pub fn qux<T, U>(x: T, _: U) -> bool where U: Qux<T> {
            x.baz()
        }

        fn main() {}
    