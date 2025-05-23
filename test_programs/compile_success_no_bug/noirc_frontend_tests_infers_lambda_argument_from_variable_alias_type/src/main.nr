
    pub struct Foo {
        value: Field,
    }

    type FooFn = fn(Foo) -> Field;

    fn main() {
      let _: FooFn = |foo| foo.value;
    }
    