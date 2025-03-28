
    pub struct Foo {
        value: Field,
    }

    pub fn func() -> fn(Foo) -> Field {
        let _ = 1;
        |foo| foo.value
    }

    fn main() {
    }
    