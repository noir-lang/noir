struct Foo {
    a: Field,
    b: Field,
}

fn main(mut x: [Foo; 3], y: pub u32) {
    assert(x[y + 2].a == 5);
}
