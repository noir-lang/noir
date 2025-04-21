
    trait Default2 {
        fn default(x: Field, y: Field) -> Field;
    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
        // Duplicate trait methods should not compile
        fn default(x: Field, y: Field) -> Field {
            y + 2 * x
        }
        // Duplicate trait methods should not compile
        fn default(x: Field, y: Field) -> Field {
            x + 2 * y
        }
    }

    fn main() {
        let _ = Foo { bar: 1, array: [2, 3] }; // silence Foo never constructed warning
    }