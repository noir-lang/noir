
    fn main() {
        let _ = Bar::foo();
    }

    pub struct Bar {
    }

    mod private_mod {
        pub trait Foo {
            fn foo() -> i32;
        }

        impl Foo for super::Bar {
            fn foo() -> i32 {
                42
            }
        }
    }
    