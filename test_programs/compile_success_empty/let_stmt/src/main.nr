struct Foo {
    a: u64,
}

fn main() {
    let _ = 42;
    let _ = 42;

    let Foo { a: _ } = Foo { a: 42 };
    let _regression_2786 = [Foo { a: 1 }; 8];
}
