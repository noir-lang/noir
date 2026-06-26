// Phase 1 of the `Validate` work (https://github.com/noir-lang/noir/issues/13188):
// `#[derive(Validate)]` recurses through a type's fields. With only the no-op leaf impls in
// place there are no invariants to assert yet, so this just demonstrates that the derive
// resolves and recurses (through nested structs and arrays) and that `validate` is callable.
use std::validate::Validate;

#[derive(Validate)]
struct Point {
    x: Field,
    y: Field,
}

#[derive(Validate)]
struct Shape {
    corners: [Point; 3],
    closed: bool,
}

fn main(x: Field) {
    let p = Point { x, y: x };
    let shape = Shape { corners: [p, p, p], closed: true };
    shape.validate();
}
