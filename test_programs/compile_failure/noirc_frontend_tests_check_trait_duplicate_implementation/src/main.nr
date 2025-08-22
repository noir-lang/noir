
    trait Default2 {
    }
    struct Foo {
        bar: Field,
    }

    impl Default2 for Foo {
    }
    impl Default2 for Foo {
    }
    fn main() {
        let _ = Foo { bar: 1 }; // silence Foo never constructed warning
    }
    