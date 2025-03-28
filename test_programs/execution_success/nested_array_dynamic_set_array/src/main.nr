// 3 values
struct Bar {
    inner: [Field; 3],
}

// 7 values
struct Foo {
    a: Field,
    b: [Field; 3],
    bar: Bar,
}

fn main(mut x: [Foo; 1], y: pub Field) {
    if y == 3 {
        x[y - 3].b = [5, 10, 15];
    }
    assert(x[y - 3].b == [5, 10, 15]);
}
