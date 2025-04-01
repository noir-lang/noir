
    pub struct Foo {
        value: Field,
    }

    pub fn func() -> fn(Foo) -> Field {
        if true {
            |foo| foo.value
        } else {
            |foo| foo.value
        }
    }

    fn main() {
    }
    