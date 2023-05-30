// This program tests:
// - the allocation of virtual arrays for array params to main
// - load instructions for such arrays

struct Foo {
    bar: Field,
    baz: bool,
}

fn main(xs : [Field; 2], foos: [Foo; 3]) -> pub Field {
    xs[0] + foos[1].bar + foos[2].baz
}
