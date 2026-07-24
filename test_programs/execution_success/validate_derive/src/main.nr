// `#[derive(Validate)]` recurses through a type's fields. With only the no-op leaf impls for
// primitive types in place there are no invariants to assert here, so this exercises that the
// derive resolves and recurses through nested structs, arrays, tuples, and `Option`, and that
// `validate` is callable.

#[derive(Validate)]
struct Point {
    x: Field,
    y: Field,
}

#[derive(Validate)]
struct Shape {
    corners: [Point; 3],
    closed: bool,
    labelled: (Field, bool),
    origin: Option<Point>,
}

fn main(x: Field) {
    let p = Point { x, y: x };
    let shape =
        Shape { corners: [p, p, p], closed: true, labelled: (x, false), origin: Option::some(p) };
    shape.validate();
}
