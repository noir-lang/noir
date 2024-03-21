struct Bar {
    inner: [Field; 3],
}

struct Foo {
    a: Field,
    b: [Field; 3],
    bar: Bar,
}

fn main(y: Field) {
    let foo_one = Foo { a: 1, b: [2, 3, 20], bar: Bar { inner: [100, 101, 102] } };
    let foo_two = Foo { a: 4, b: [5, 6, 21], bar: Bar { inner: [103, 104, 105] } };
    let foo_three = Foo { a: 7, b: [8, 9, 22], bar: Bar { inner: [106, 107, 108] } };
    let foo_four = Foo { a: 10, b: [11, 12, 23], bar: Bar { inner: [109, 110, 111] } };
    let mut x = [foo_one];
    x = x.push_back(foo_two);
    x = x.push_back(foo_three);
    x = x.push_back(foo_four);

    assert(x[y - 3].a == 1);
    assert(x[y - 3].b == [2, 3, 20]);
    assert(x[y - 2].a == 4);
    assert(x[y - 2].b == [5, 6, 21]);
    assert(x[y - 1].a == 7);
    assert(x[y - 1].b == [8, 9, 22]);
    assert(x[y].a == 10);
    assert(x[y].b == [11, 12, 23]);
    assert(x[y].bar.inner == [109, 110, 111]);

    if y != 2 {
        x[y - 2].a = 50;
    } else {
        x[y - 2].a = 100;
    }
    assert(x[y - 2].a == 50);

    if y == 2 {
        x[y - 1].b = [50, 51, 52];
    } else {
        x[y - 1].b = [100, 101, 102];
    }
    assert(x[2].b == [100, 101, 102]);

    assert(x[y - 3].bar.inner == [100, 101, 102]);
    assert(x[y - 2].bar.inner == [103, 104, 105]);
    assert(x[y - 1].bar.inner == [106, 107, 108]);
    assert(x[y].bar.inner == [109, 110, 111]);
}
