
        pub struct Foo<T, U> {
            x: T,
            y: U,
        }

        impl<T, U> Foo<T, U> {
            fn new(x: T, y: U) -> Self {
                Foo { x, y }
            }
        }

        type Bar<T> = Foo<T, i32>;

        fn main() {
            let _ = Bar::<bool>::new(1, 1);
        }
    