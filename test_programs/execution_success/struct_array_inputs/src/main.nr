struct Foo {
    bar: field,
    baz: field,
}

fn main(foos: [Foo; 3]) -> pub field {
    foos[2].bar + foos[2].baz
}
