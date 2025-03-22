
    pub struct Foo {
        value: Field,
    }

    type FooFn = fn(Foo) -> Field;
    type FooFn2 = FooFn;

    fn main() {
      let _: FooFn2 = |foo| foo.value;
    }
    