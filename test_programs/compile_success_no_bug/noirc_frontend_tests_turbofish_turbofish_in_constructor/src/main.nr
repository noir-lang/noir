
    struct Foo<T> {
        x: T
    }

    fn main() {
        let x: Field = 0;
        let _ = Foo::<i32> { x: x };
    }
    