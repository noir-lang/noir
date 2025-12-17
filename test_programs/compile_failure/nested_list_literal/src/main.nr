struct FooParent<T> {
    parent_arr: [Field; 3],
    foos: [Foo<T>],
}

struct Bar {
    inner: [Field; 3],
}

struct Foo<T> {
    a: Field,
    b: T,
    bar: Bar,
}

fn main(x: Field, y: pub Field) {
    assert(x != y);

    let foo = Foo { a: 7, b: [8, 9, 22].as_list(), bar: Bar { inner: [106, 107, 108] } };
    let mut list = [foo, foo];
    list = list.push_back(foo);
    assert(list.len() == 3);
}
