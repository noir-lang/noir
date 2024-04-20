struct FooParent<T> {
    parent_arr: [field; 3],
    foos: [Foo<T>],
}

struct Bar {
    inner: [field; 3],
}

struct Foo<T> {
    a: field,
    b: T,
    bar: Bar,
}

fn main(x: field, y: pub field) {
    assert(x != y);

    let foo = Foo { a: 7, b: [8, 9, 22].as_slice(), bar: Bar { inner: [106, 107, 108] } };
    let mut slice = [foo, foo];
    slice = slice.push_back(foo);
    assert(slice.len() == 3);
}
