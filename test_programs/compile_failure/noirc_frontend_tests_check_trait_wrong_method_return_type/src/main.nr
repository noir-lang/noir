
    trait Default2 {
        fn default() -> Self;
    }

    struct Foo {
    }

    impl Default2 for Foo {
        fn default() -> Field {
            0
        }
    }

    fn main() {
        let _ = Foo {}; // silence Foo never constructed warning
    }
    