#[fail_assert]
struct Foo { x: Field }

comptime fn fail_assert(_typ: StructDefinition) {
    assert(false);
}

fn main() {}
