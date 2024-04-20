struct FooParent {
    parent_arr: [field; 3],
    foos: [Foo],
}

struct Bar {
    inner: [field; 3],
}

struct Foo {
    a: field,
    b: [field],
    bar: Bar,
}

fn main(x: field, y: pub field) {
    assert(x != y);
}
