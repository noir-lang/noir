#[fail_assert]
struct Foo { x: Field }

comptime fn fail_assert(_typ: TypeDefinition) {
    assert(false);
}

fn main() {}
