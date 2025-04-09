
    trait Foo {
        type Assoc;
    }

    struct Bar {}

    impl Foo for Bar {
        type Assoc = i32;
    }

    fn main() {
        // AsTraitPath syntax is a bit silly when associated types
        // are explicitly specified
        let _: i64 = 1 as <Bar as Foo<Assoc = i32>>::Assoc;

        let _ = Bar {}; // silence Bar never constructed warning
    }
    