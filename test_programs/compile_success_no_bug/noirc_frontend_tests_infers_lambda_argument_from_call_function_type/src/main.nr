
    struct Foo {
        value: Field,
    }

    fn call(f: fn(Foo) -> Field) -> Field {
        f(Foo { value: 1 })
    }

    fn main() {
        let _ = call(|foo| foo.value);
    }
    