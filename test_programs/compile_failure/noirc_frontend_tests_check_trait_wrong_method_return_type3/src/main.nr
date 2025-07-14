
    trait Default2 {
        fn default(x: Field, y: Field) -> Self;
    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
        fn default(_x: Field, _y: Field) {
        }
    }

    fn main() {
        let _ = Foo { bar: 1, array: [2, 3] }; // silence Foo never constructed warning
    }
    