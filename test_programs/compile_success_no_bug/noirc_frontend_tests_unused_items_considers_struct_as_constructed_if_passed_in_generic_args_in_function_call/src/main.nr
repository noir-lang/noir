
    struct Bar {}

    fn foo<T>() {}

    fn main() {
        let _ = foo::<Bar>();
    }
    