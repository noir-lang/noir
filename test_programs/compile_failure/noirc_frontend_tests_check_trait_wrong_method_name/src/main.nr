
    trait Default2 {
    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
        fn does_not_exist(x: Field, y: Field) -> Self {
            Self { bar: x, array: [x,y] }
        }
    }

    fn main() {
        let _ = Foo { bar: 1, array: [2, 3] }; // silence Foo never constructed warning
    }