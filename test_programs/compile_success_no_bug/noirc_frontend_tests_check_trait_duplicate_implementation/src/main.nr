
    trait Default {
    }
    struct Foo {
        bar: Field,
    }

    impl Default for Foo {
    }
    impl Default for Foo {
    }
    fn main() {
        let _ = Foo { bar: 1 }; // silence Foo never constructed warning
    }
    