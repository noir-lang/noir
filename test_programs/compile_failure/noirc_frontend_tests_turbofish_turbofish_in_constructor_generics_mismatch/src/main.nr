
    struct Foo<T> {
        x: T
    }

    fn main() {
        let _ = Foo::<i32, i64> { x: 1 };
    }
    