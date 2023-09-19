struct Bar {
    inner: [Field; 3],
}

struct Foo {
    a: Field,
    b: [Field; 3],
    bar: Bar,
}

fn main(mut x : [Foo; 4], y : pub Field) {
    assert(x[y - 3].a == 1);
    assert(x[y - 3].b == [2, 3, 20]);
    assert(x[y - 2].a == 4);
    assert(x[y - 2].b == [5, 6, 21]);
    assert(x[y - 1].a == 7);
    assert(x[y - 1].b == [8, 9, 22]);
    assert(x[y].a == 10);
    assert(x[y].b == [11, 12, 23]);
    assert(x[y].bar.inner == [109, 110, 111]);
    dep::std::println("show list");
    for i in 0..4 {
        dep::std::println(x[i]);
    }
    if y != 2 {
        x[y].a = 50;
    } else {
        x[y].a = 100;
    }
    assert(x[y].a == 50);

    if y == 2 {
        x[y - 1].b = [50, 51, 52];
    } else {
        x[y - 1].b = [100, 101, 102];
    }
    assert(x[2].b == [100, 101, 102]);
}

