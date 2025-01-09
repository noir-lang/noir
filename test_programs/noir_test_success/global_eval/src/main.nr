use std::uint128::U128;

// These definitions require `to_be_bits` and `to_le_bits` to be supported at comptime.
global POW64_A: Field = 2.pow_32(64);
global POW64_B: Field = (U128::one() << 64).to_integer();

#[test]
fn test_pow64_equal() {
    assert_eq(POW64_A, POW64_B);
}
