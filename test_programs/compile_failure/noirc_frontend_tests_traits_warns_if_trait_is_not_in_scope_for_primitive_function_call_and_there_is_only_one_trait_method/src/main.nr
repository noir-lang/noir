
    fn main() {
        let _ = Field::foo();
    }

    mod private_mod {
        pub trait Foo {
            fn foo() -> i32;
        }

        impl Foo for Field {
            fn foo() -> i32 {
                42
            }
        }
    }
    