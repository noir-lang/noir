// Tests arithmetic operations on fields
fn main() {
    let x = 4;
    let y = 2;
    assert((x + y) == unsafe { add(x, y) });
    assert((x - y) == unsafe { sub(x, y) });
    assert((x * y) == unsafe { mul(x, y) });
    assert((x / y) == unsafe { div(x, y) });
}

unconstrained fn add(x: Field, y: Field) -> Field {
    x + y
}

unconstrained fn sub(x: Field, y: Field) -> Field {
    x - y
}

unconstrained fn mul(x: Field, y: Field) -> Field {
    x * y
}

unconstrained fn div(x: Field, y: Field) -> Field {
    x / y
}
