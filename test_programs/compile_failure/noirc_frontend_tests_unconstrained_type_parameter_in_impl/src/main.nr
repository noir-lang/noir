
        pub struct Foo<T> {}

        impl<T, U> Foo<T> {}

        fn main() {
            let _ = Foo {};
        }
        