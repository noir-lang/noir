
    struct Bar {}

    trait Foo {
        fn foo(self, a: u64);
    }

    impl Foo for Bar {
        fn foo(self, _a: u64) {}
    }

    fn main() {
        let _ = Bar {};
    }
    