
    pub trait Foo {
        unconstrained fn foo() -> Field;
    }

    impl Foo for Field {
        fn foo() -> Field {
            42
        }
    }
    