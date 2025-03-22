
    pub mod moo {
        struct Bar {}

        pub trait Foo { 
            fn foo() -> Self;
        }

        impl Foo for Bar {
            fn foo() -> Self { 
                Bar {}
            }
        }

        pub fn no_unused_warnings() {
            let _ = Bar {};
        }
    }
    fn main() {}
    