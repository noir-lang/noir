struct Foo {
    bar: Field,
    baz: Field,
}

fn main(x: u64, foos: [Foo; 3]) -> pub Field {
    x as Field + foos[2].bar + foos[2].baz
}
