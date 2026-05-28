// Regression test: a parenthesized negative integer literal cast to a signed type
// must produce the same value as the corresponding typed literal. Previously the
// polymorphic literal defaulted to `Field` and the field-to-int cast went through
// a bit-truncation that yielded the wrong answer for primes (such as BN254) whose
// residue mod 2^bit_size is non-zero - for example, `-128 as i8` produced `-127`
// instead of `-128`.
fn main() {
    assert_eq(-128 as i8, -128_i8);
    assert_eq(-1 as i8, -1_i8);
    assert_eq(-127 as i8, -127_i8);

    assert_eq(-32768 as i16, -32768_i16);
    assert_eq(-1 as i16, -1_i16);

    assert_eq(-2147483648 as i32, -2147483648_i32);
    assert_eq(-1 as i32, -1_i32);

    assert_eq(-9223372036854775808 as i64, -9223372036854775808_i64);
    assert_eq(-1 as i64, -1_i64);
}
