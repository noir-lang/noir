
    pub struct Foo {
        value: Field,
    }

    pub fn func() -> fn(Foo) -> Field {
        |foo| foo.value
    }

    fn main() {
    }
    