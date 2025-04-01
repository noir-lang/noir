
    trait Default {
        fn default(x: Field) -> Self;
    }

    struct Foo {
        bar: u32,
    }

    impl Default for Foo {
        fn default(x: u32) -> Self {
            Foo {bar: x}
        }
    }

    fn main() {
    }
    