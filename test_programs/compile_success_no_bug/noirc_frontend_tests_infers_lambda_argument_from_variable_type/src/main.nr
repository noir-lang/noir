
    pub struct Foo {
        value: Field,
    }

    fn main() {
      let _: fn(Foo) -> Field = |foo| foo.value;
    }
    