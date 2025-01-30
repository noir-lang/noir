// Tests arithmetic operations on fields
fn main() {
    /// Safety: testing context
    unsafe {
        //@safety: testing context
        let x = 4;
        let y = 2;
        assert((x + y) == add(x, y));
        assert((x - y) == sub(x, y));
        assert((x * y) == mul(x, y));
        assert((x / y) == div(x, y));
    }
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
