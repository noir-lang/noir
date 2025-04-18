
    pub struct Foo {
        value: Field,
    }

    type Alias = (fn(Foo) -> Field, Field);

    fn main() {
      let _: Alias = (|foo| foo.value, 1);
    }
    