
        pub struct Foo<T> {
        }

        impl<T> Foo<T> {
            fn new() -> Self {
                Foo {}
            }
        }

        type Bar<T> = Foo<T>;

        fn foo() -> Foo<i32> {
            Bar::<i32>::new()
        }

        fn main() {
            let _ = foo();
        }
    