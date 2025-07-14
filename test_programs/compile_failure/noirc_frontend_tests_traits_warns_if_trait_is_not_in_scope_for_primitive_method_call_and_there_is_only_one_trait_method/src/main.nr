
    fn main() {
        let x: Field = 1;
        let _ = x.foo();
    }

    mod private_mod {
        pub trait Foo {
            fn foo(self) -> i32;
        }

        impl Foo for Field {
            fn foo(self) -> i32 {
                self as i32
            }
        }
    }
    