
    struct Foo<T> {}

    impl<T> Foo<T> {
        fn static_method<U>() {}
    }

    fn main() {
        Foo::<u8>::static_method::<Field>();
    }
    