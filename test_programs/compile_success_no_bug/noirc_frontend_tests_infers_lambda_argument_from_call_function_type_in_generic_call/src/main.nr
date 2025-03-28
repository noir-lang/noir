
    struct Foo {
        value: Field,
    }

    fn call<T>(t: T, f: fn(T) -> Field) -> Field {
        f(t)
    }

    fn main() {
        let _ = call(Foo { value: 1 }, |foo| foo.value);
    }
    