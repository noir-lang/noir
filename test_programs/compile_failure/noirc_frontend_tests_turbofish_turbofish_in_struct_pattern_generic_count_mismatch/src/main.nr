
    struct Foo<T> {
        x: T
    }

    fn main() {
        let value = 0;
        let Foo::<i32, i64> { x } = Foo { x: value };
        let _ = x;
    }
    