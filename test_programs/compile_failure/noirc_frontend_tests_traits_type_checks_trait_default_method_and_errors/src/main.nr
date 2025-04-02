
        pub trait Foo {
            fn foo(self) -> i32 {
                let _ = self;
                true
            }
        }

        fn main() {}
    