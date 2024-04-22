fn main(x: field) {
    assert(1 == conditional(x));
}

unconstrained fn conditional(x: field) -> field {
    let z = x as u8 + 20;
    assert_eq(z, 25, f"Expected 25 but got {z}");
    assert(x == 10, f"Expected x to equal 10, but got {x}");
    1
}
