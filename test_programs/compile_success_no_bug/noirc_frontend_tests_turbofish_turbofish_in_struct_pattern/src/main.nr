
    struct Foo<T> {
        x: T
    }

    fn main() {
        let value: Field = 0;
        let Foo::<Field> { x } = Foo { x: value };
        let _ = x;
    }
    