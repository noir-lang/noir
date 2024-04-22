struct Foo {
    a: field,
    b: field,
}

fn main(mut x: [Foo; 3], y: pub field) {
    assert(x[y + 2].a == 5);
}
