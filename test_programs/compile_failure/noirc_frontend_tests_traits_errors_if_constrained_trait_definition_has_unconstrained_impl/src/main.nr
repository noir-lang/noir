
    pub trait Foo {
        fn foo() -> Field;
    }

    impl Foo for Field {
        unconstrained fn foo() -> Field {
            42
        }
    }
    