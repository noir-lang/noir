
    use private_mod::Foo;

    fn main() {
        let bar = Bar { x: 42 };
        let _ = bar.foo();
    }

    pub struct Bar {
        x: i32,
    }

    mod private_mod {
        pub trait Foo {
            fn foo(self) -> i32;
        }

        impl Foo for super::Bar {
            fn foo(self) -> i32 {
                self.x
            }
        }
    }
    