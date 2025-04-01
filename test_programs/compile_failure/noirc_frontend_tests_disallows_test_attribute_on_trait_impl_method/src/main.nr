
        pub trait Trait {
            fn foo() { }
        }

        pub struct Foo { }

        impl Trait for Foo {
            #[test]
            fn foo() { }
        }

        fn main() { }
    