fn main(a: Field, b: Field) -> pub u8 {
    let lhs = sub_and_truncate(a, b);
    // Safety: Test program
    let rhs = unsafe { __sub_and_truncate(a, b) };
    assert_eq(lhs, rhs);
    lhs
}

fn sub_and_truncate(a: Field, b: Field) -> u8 {
    ((a - b) as u8)
}

unconstrained fn __sub_and_truncate(a: Field, b: Field) -> u8 {
    ((a - b) as u8)
}
