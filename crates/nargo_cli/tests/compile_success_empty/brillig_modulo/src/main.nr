// Tests a very simple program.
// 
// The features being tested is modulo operations on brillig


fn main() {
    assert_eq(modulo(47, 3), 2);
    assert_eq(modulo(2, 3), 2);
    assert_eq(signed_modulo(5, 3), 2);
    assert_eq(signed_modulo(2, 3), 2);

    let minus_two: i4 = 14;
    let minus_three: i4 = 13;
    let minus_five: i4 = 11;

    // (5 / -3) * -3 + 2 = -1 * -3 + 2 = 3 + 2 = 5
    assert_eq(signed_modulo(5, minus_three), 2);
    // (-5 / 3) * 3 - 2 = -1 * 3 - 2 = -3 - 2 = -5
    assert_eq(signed_modulo(minus_five, 3), minus_two);
    // (-5 / -3) * -3 - 2 = 1 * -3 - 2 = -3 - 2 = -5
    assert_eq(signed_modulo(minus_five, minus_three), minus_two);
}

unconstrained fn modulo(x: u32, y: u32) -> u32 {
    x % y
}

unconstrained fn signed_modulo(x: i4, y: i4) -> i4 {
    x % y
}
