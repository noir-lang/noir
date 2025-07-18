
        mod moo {
            struct Foo {}
        }
        use moo::Foo;
        fn main() {
            let _ = Foo {};
        }
    