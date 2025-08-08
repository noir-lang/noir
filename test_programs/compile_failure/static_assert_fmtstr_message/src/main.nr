use std::static_assert;

struct Foo {
    x: Field,
    y: Field,
    z: Field,
}

fn main() {
    let foo = Foo { x: 1, y: 2, z: 3 };
    static_assert(1 == 2, f"foo: {foo}");
}
