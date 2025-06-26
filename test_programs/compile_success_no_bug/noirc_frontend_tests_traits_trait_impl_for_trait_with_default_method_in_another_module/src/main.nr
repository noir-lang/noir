
    mod moo {
        fn foo() {}

        pub trait Trait {
            fn bar() {
                foo()
            }
        }
    }

    impl moo::Trait for i32 {}

    fn main() {}
    