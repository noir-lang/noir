trait MyDefault {
    fn my_default(x: Field, y: Field) -> Self;

    fn method2(x: Field) -> Field {
        x
    }
}

struct Foo {
    bar: Field,
    array: [Field; 2],
}

impl MyDefault for Foo {
    fn my_default(x: Field, y: Field) -> Self {
        Self { bar: x, array: [x, y] }
    }
}

fn main(x: Field) {
    let first = Foo::method2(x);
    assert(first == x);
}
