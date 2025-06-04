
        pub trait Foo {
            fn foo(self) -> i32 {
                let _ = self;
                1
            }
        }

        fn main() {}
    