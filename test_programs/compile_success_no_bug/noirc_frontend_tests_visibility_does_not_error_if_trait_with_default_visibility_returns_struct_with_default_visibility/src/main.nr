
    struct Foo {}

    trait Bar {
        fn bar(self) -> Foo;
    }

    impl Bar for Foo {
        fn bar(self) -> Foo {
            self
        }
    }

    fn main() {
        let foo = Foo {};
        let _ = foo.bar();
    }
    