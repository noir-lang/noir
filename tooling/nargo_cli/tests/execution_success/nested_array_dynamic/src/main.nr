struct FooParent {
    parent_arr: [Field; 3],
    foos: [Foo; 4],
}

struct Bar {
    inner: [Field; 3],
}

struct Foo {
    a: Field,
    b: [Field; 3],
    bar: Bar,
}

fn main(mut x : [Foo; 4], y : pub Field) {
    // assert(x[3].a == 10);
    // assert(x[1].b == [5, 6, 21]);

    assert(x[y - 3].a == 1);
    assert(x[y - 3].b == [2, 3, 20]);
    assert(x[y - 2].a == 4);
    assert(x[y - 2].b == [5, 6, 21]);
    assert(x[y - 1].a == 7);
    assert(x[y - 1].b == [8, 9, 22]);
    assert(x[y].a == 10);
    assert(x[y].b == [11, 12, 23]);
    assert(x[y].bar.inner == [109, 110, 111]);

    // Check dynamic array set 
    if y != 2 {
        x[y].a = 50;
    } else {
        x[y].a = 100;
    }
    assert(x[3].a == 50);

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

    let foo_parent_one = FooParent { parent_arr: [0, 1, 2], foos: x };
    let foo_parent_two = FooParent { parent_arr: [3, 4, 5], foos: x };
    let mut foo_parents: [FooParent; 2] = [foo_parent_one, foo_parent_two];

    if y != 2 {
        foo_parents[y - 2].foos[y - 1].b[y - 1] = 5000;
    } else {
        foo_parents[y - 2].foos[y - 1].b[y - 1] = 1000;
    }
    let b_y_minus_one = foo_parents[y - 2].foos[y - 1].b[y - 1];
    assert(b_y_minus_one == 5000);
}

