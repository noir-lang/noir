
        pub struct Foo<T> {
            x: T,
        }

        impl<T> Foo<T> {
            fn new(x: T) -> Self {
                Foo { x }
            }
        }

        type Bar<T> = Foo<T>;

        fn main() {
            let _ = Bar::<i32>::new(true);
        }
    