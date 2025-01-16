#[derive(Eq)]
struct Foo<let T: u32> {
    a: [Field; T],
    b: u32,
}

fn main() {
    let foo = Foo { a: [0; 10], b: 27 };
    let bar = Foo { a: [0; 10], b: 28 };
    assert(foo != bar);
}

