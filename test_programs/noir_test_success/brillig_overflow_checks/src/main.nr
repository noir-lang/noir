use std::field::bn254::{assert_gt, TWO_POW_128};

#[test(should_fail_with = "attempt to add with overflow")]
unconstrained fn test_overflow_add() {
    let a: u8 = 255;
    let b: u8 = 1;
    assert_eq(a + b, 0);
}

#[test(should_fail_with = "attempt to subtract with overflow")]
unconstrained fn test_overflow_sub() {
    let a: u8 = 0;
    let b: u8 = 1;
    assert_eq(a - b, 255);
}

#[test(should_fail_with = "attempt to multiply with overflow")]
unconstrained fn test_overflow_mul() {
    let a: u8 = 128;
    let b: u8 = 2;
    assert_eq(a * b, 0);
}

