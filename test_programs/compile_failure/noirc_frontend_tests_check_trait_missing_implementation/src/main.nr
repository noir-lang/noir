
    trait Default2 {
        fn default(x: Field, y: Field) -> Self;

        fn method2(x: Field) -> Field;

    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
        fn default(x: Field, y: Field) -> Self {
            Self { bar: x, array: [x,y] }
        }
    }

    fn main() {
    }
    