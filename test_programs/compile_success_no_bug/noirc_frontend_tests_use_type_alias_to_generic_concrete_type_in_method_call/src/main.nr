
        pub struct Foo<T> {
            x: T,
        }

        impl<T> Foo<T> {
            fn new(x: T) -> Self {
                Foo { x }
            }
        }

        type Bar = Foo<i32>;

        fn foo() -> Bar {
            Bar::new(1)
        }

        fn main() {
            let _ = foo();
        }
    