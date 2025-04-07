
    struct Foo {
        value: Field,
    }

    type MyFn = fn(Foo) -> Field;

    fn call(f: MyFn) -> Field {
        f(Foo { value: 1 })
    }

    fn main() {
        let _ = call(|foo| foo.value);
    }
    