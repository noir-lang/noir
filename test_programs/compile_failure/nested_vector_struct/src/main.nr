struct FooParent {
    parent_arr: [Field; 3],
    foos: [Foo],
}

struct Bar {
    inner: [Field; 3],
}

struct Foo {
    a: Field,
    b: [Field],
    bar: Bar,
}

fn main(x: Field, y: pub Field) {
    assert(x != y);
}
