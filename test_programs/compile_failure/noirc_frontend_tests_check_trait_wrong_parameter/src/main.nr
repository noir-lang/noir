
    trait Default2 {
        fn default(x: Field) -> Self;
    }

    struct Foo {
        bar: u32,
    }

    impl Default2 for Foo {
        fn default(x: u32) -> Self {
            Foo {bar: x}
        }
    }

    fn main() {
    }
    