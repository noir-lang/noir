// Tests arithmetic operations on fields
fn main() {
    let x = 4;
    let y = 2;
    assert((x + y) == add(x, y));
    assert((x - y) == sub(x, y));
    assert((x * y) == mul(x, y));
    assert((x / y) == div(x, y));
}

unconstrained fn add(x: field, y: field) -> field {
    x + y
}

unconstrained fn sub(x: field, y: field) -> field {
    x - y
}

unconstrained fn mul(x: field, y: field) -> field {
    x * y
}

unconstrained fn div(x: field, y: field) -> field {
    x / y
}
