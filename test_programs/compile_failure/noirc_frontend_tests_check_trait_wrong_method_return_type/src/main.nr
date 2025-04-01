
    trait Default {
        fn default() -> Self;
    }

    struct Foo {
    }

    impl Default for Foo {
        fn default() -> Field {
            0
        }
    }

    fn main() {
        let _ = Foo {}; // silence Foo never constructed warning
    }
    