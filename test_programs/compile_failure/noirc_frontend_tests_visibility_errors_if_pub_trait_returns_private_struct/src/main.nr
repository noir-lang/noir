
    pub mod moo {
        struct Bar {}

        pub trait Foo { 
            fn foo() -> Bar;
        }

        pub fn no_unused_warnings() {
            let _ = Bar {};
        }
    }
    fn main() {}
    