// These definitions require `to_be_bits` and `to_le_bits` to be supported at comptime.
global BITS_BE_13: [u1; 4] = (13 as Field).to_be_bits();
global BITS_LE_13: [u1; 4] = (13 as Field).to_le_bits();

#[test]
fn test_be_and_le_bits() {
    assert_eq(BITS_BE_13, [1, 1, 0, 1]);
    assert_eq(BITS_LE_13, [1, 0, 1, 1]);
}
