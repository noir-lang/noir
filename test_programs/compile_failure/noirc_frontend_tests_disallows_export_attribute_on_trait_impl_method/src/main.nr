
        pub trait Trait {
            fn foo() { }
        }

        pub struct Foo { }

        impl Trait for Foo {
            #[export]
            fn foo() { }
        }

        fn main() { }
    