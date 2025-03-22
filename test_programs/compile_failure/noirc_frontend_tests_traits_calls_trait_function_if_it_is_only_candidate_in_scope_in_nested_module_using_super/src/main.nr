
    mod moo {
        use super::public_mod::Foo;

        pub fn method() {
            let _ = super::Bar::foo();
        }
    }

    fn main() {}

    pub struct Bar {}

    pub mod public_mod {
        pub trait Foo {
            fn foo() -> i32;
        }

        impl Foo for super::Bar {
            fn foo() -> i32 {
                42
            }
        }
    }
    