
        pub struct Foo {
        }

        impl Foo {
            fn new() -> Self {
                Foo {}
            }
        }

        type Bar = Foo;

        fn foo() -> Foo {
            Bar::new()
        }

        fn main() {
            let _ = foo();
        }
    