
struct Bar {
    inner: [Field; 3],
}

struct Foo {
    a: Field,
    b: [Field; 3],
    bar: Bar,
}

struct FooParent {
    array: [Field; 3],
    foos: [Foo; 4],
}

fn main(mut x: [Foo; 4], y: pub Field) {
    let foo_parent_one = FooParent { array: [0, 1, 2], foos: x };
    let foo_parent_two = FooParent { array: [3, 4, 5], foos: x };
    let mut foo_parents = [foo_parent_one, foo_parent_two];

    if y == 3 {
        foo_parents[y - 2].foos[y - 2].b[y - 1] = 5000;
    } else {
        foo_parents[y - 2].foos[y - 2].b[y - 1] = 1000;
    }
    assert(foo_parents[1].foos[1].b[2] == 5000);
}