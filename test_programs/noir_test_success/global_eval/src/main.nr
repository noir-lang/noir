use std::uint128::U128;

// These definitions require `to_be_bits` and `to_le_bits` to be supported at comptime.
global BITS_BE_13: [u1; 4] = (13 as Field).to_be_bits();
global BITS_LE_13: [u1; 4] = (13 as Field).to_le_bits();

// Examples from #6691 which use the above behind the scenes.
global POW64_A: Field = 2.pow_32(64);
global POW64_B: Field = (U128::one() << 64).to_integer();

#[test]
fn test_be_and_le_bits() {
    assert_eq(BITS_BE_13, [1,1,0,1]);
    assert_eq(BITS_LE_13, [1,0,1,1]);
}

#[test]
fn test_pow64() {
    assert_eq(POW64_A, POW64_B);
}
