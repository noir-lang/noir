
    struct Foo<T> {
        x: T
    }

    impl <T> Foo<T> {
        fn new(x: T) -> Self {
            Foo { x }
        }
    }

    fn main() {
        let _ = Foo::<i32>::new(1);
    }
    