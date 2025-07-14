
    pub mod moo {
        struct Bar {}
        pub struct Foo {}

        impl Foo { 
            pub fn bar() -> Bar { 
                Bar {}
            }
        }

        pub fn no_unused_warnings() {
            let _ = Foo {};            
        }
    }
    fn main() {}
    