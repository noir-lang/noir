
    pub mod moo {
        struct Bar {}

        impl Bar { 
            pub fn bar() -> Bar { 
                Bar {}
            }
        }

        pub fn no_unused_warnings() {
            let _ = Bar {};
        }
    }
    fn main() {}
    