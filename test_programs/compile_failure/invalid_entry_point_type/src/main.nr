struct Foo {
    bar: Bar,
}

struct Bar {
    baz: [Field],
}

type SomeAlias = Foo;

fn main(_: SomeAlias) {}
