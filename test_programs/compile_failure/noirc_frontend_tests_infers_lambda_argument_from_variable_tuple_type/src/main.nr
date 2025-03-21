
    pub struct Foo {
        value: Field,
    }

    fn main() {
      let _: (fn(Foo) -> Field, _) = (|foo| foo.value, 1);
    }
    