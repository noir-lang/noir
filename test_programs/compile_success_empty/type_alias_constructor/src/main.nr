struct Foo {
    inner: Field
}

type Bar = Foo;

fn returns_a_foo() -> Foo {
    Bar { inner: 42 }
}

fn main() {
    let x = Bar { inner: 42 };
    let Bar { inner } = returns_a_foo();
    let y = Foo { inner };
    assert_eq(x.inner, y.inner);
}
