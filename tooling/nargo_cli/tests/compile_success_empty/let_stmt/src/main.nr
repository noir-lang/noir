struct Foo {
    a: u64,
}

fn main() {
    let _ = 42;
    let _ = 42;

    let Foo { a: _ } = Foo { a: 42 };
}
