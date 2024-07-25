use std::uint128::{U128, pow64, pow63};

fn main() { }

#[test]
fn test_not(lo: u64, hi: u64) {
    let num = U128::from_u64s_le(lo, hi);
    let not_num = num.not();

    assert_eq(not_num.hi, (hi.not() as Field));
    assert_eq(not_num.lo, (lo.not() as Field));

    let not_not_num = not_num.not();
    assert_eq(num, not_not_num);
}
#[test]
fn test_construction() {
    // Check little-endian u64 is inversed with big-endian u64 construction
    let a = U128::from_u64s_le(2, 1);
    let b = U128::from_u64s_be(1, 2);
    assert_eq(a, b);
    // Check byte construction is equivalent
    let c = U128::from_le_bytes([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
    let d = U128::from_u64s_le(0x0706050403020100, 0x0f0e0d0c0b0a0908);
    assert_eq(c, d);
}
#[test]
fn test_byte_decomposition() {
    let a = U128::from_u64s_le(0x0706050403020100, 0x0f0e0d0c0b0a0908);
    // Get big-endian and little-endian byte decompostions
    let le_bytes_a= a.to_le_bytes();
    let be_bytes_a= a.to_be_bytes();

    // Check equivalence
    for i in 0..16 {
        assert_eq(le_bytes_a[i], be_bytes_a[15 - i]);
    }
    // Reconstruct U128 from byte decomposition
    let b= U128::from_le_bytes(le_bytes_a);
    // Check that it's the same element
    assert_eq(a, b);
}
#[test]
fn test_hex_constuction() {
    let a = U128::from_u64s_le(0x1, 0x2);
    let b = U128::from_hex("0x20000000000000001");
    assert_eq(a, b);

    let c= U128::from_hex("0xffffffffffffffffffffffffffffffff");
    let d= U128::from_u64s_le(0xffffffffffffffff, 0xffffffffffffffff);
    assert_eq(c, d);

    let e= U128::from_hex("0x00000000000000000000000000000000");
    let f= U128::from_u64s_le(0, 0);
    assert_eq(e, f);
}

// Ascii decode tests

#[test]
fn test_ascii_decode_correct_range() {
    // '0'..'9' range
    for i in 0..10 {
        let decoded= U128::decode_ascii(48 + i);
        assert_eq(decoded, i as Field);
    }
    // 'A'..'F' range
    for i in 0..6 {
        let decoded = U128::decode_ascii(65 + i);
        assert_eq(decoded, (i + 10) as Field);
    }
    // 'a'..'f' range
    for i in 0..6 {
        let decoded = U128::decode_ascii(97 + i);
        assert_eq(decoded, (i + 10) as Field);
    }
}

#[test(should_fail)]
fn test_ascii_decode_range_less_than_48_fails_0() {
    crate::println(U128::decode_ascii(0));
}
#[test(should_fail)]
fn test_ascii_decode_range_less_than_48_fails_1() {
    crate::println(U128::decode_ascii(47));
}

#[test(should_fail)]
fn test_ascii_decode_range_58_64_fails_0() {
    let _ = U128::decode_ascii(58);
}
#[test(should_fail)]
fn test_ascii_decode_range_58_64_fails_1() {
    let _ = U128::decode_ascii(64);
}
#[test(should_fail)]
fn test_ascii_decode_range_71_96_fails_0() {
    let _ = U128::decode_ascii(71);
}
#[test(should_fail)]
fn test_ascii_decode_range_71_96_fails_1() {
    let _ = U128::decode_ascii(96);
}
#[test(should_fail)]
fn test_ascii_decode_range_greater_than_102_fails() {
    let _ = U128::decode_ascii(103);
}

#[test(should_fail)]
fn test_ascii_decode_regression() {
    // This code will actually fail because of ascii_decode,
    // but in the past it was possible to create a value > (1<<128)
    let a = U128::from_hex("0x~fffffffffffffffffffffffffffffff");
    let b:Field= a.to_integer();
    let c= b.to_le_bytes(17);
    assert(c[16] != 0);
}

#[test]
fn test_unconstrained_div() {
    // Test the potential overflow case
    let a= U128::from_u64s_le(0x0, 0xffffffffffffffff);
    let b= U128::from_u64s_le(0x0, 0xfffffffffffffffe);
    let c= U128::one();
    let d= U128::from_u64s_le(0x0, 0x1);
    let (q,r) = a.unconstrained_div(b);
    assert_eq(q, c);
    assert_eq(r, d);

    let a = U128::from_u64s_le(2, 0);
    let b = U128::one();
    // Check the case where a is a multiple of b
    let (c,d ) = a.unconstrained_div(b);
    assert_eq((c, d), (a, U128::zero()));

    // Check where b is a multiple of a
    let (c,d) = b.unconstrained_div(a);
    assert_eq((c, d), (U128::zero(), b));

    // Dividing by zero returns 0,0
    let a = U128::from_u64s_le(0x1, 0x0);
    let b = U128::zero();
    let (c,d)= a.unconstrained_div(b);
    assert_eq((c, d), (U128::zero(), U128::zero()));

    // Dividing 1<<127 by 1<<127 (special case)
    let a = U128::from_u64s_le(0x0, pow63 as u64);
    let b = U128::from_u64s_le(0x0, pow63 as u64);
    let (c,d )= a.unconstrained_div(b);
    assert_eq((c, d), (U128::one(), U128::zero()));
}

#[test]
fn integer_conversions() {
    // Maximum
    let start:Field = 0xffffffffffffffffffffffffffffffff;
    let a = U128::from_integer(start);
    let end = a.to_integer();
    assert_eq(start, end);

    // Minimum
    let start:Field = 0x0;
    let a = U128::from_integer(start);
    let end = a.to_integer();
    assert_eq(start, end);

    // Low limb
    let start:Field = 0xffffffffffffffff;
    let a = U128::from_integer(start);
    let end = a.to_integer();
    assert_eq(start, end);

    // High limb
    let start:Field = 0xffffffffffffffff0000000000000000;
    let a = U128::from_integer(start);
    let end = a.to_integer();
    assert_eq(start, end);
}

#[test]
fn integer_conversions_fuzz(lo: u64, hi: u64) {
    let start: Field = (lo as Field) + pow64 * (hi as Field);
    let a = U128::from_integer(start);
    let end = a.to_integer();
    assert_eq(start, end);
}

#[test]
fn test_wrapping_mul() {
    // 1*0==0
    assert_eq(U128::zero(), U128::zero().wrapping_mul(U128::one()));

    // 0*1==0
    assert_eq(U128::zero(), U128::one().wrapping_mul(U128::zero()));

    // 1*1==1
    assert_eq(U128::one(), U128::one().wrapping_mul(U128::one()));

    // 0 * ( 1 << 64 ) ==  0
    assert_eq(U128::zero(), U128::zero().wrapping_mul(U128::from_u64s_le(0, 1)));

    // ( 1 << 64 ) * 0 == 0
    assert_eq(U128::zero(), U128::from_u64s_le(0, 1).wrapping_mul(U128::zero()));

    // 1 * ( 1 << 64 ) == 1 << 64
    assert_eq(U128::from_u64s_le(0, 1), U128::from_u64s_le(0, 1).wrapping_mul(U128::one()));

    // ( 1 << 64 ) * 1 == 1 << 64
    assert_eq(U128::from_u64s_le(0, 1), U128::one().wrapping_mul(U128::from_u64s_le(0, 1)));

    // ( 1 << 64 ) * ( 1 << 64 ) == 1 << 64
    assert_eq(U128::zero(), U128::from_u64s_le(0, 1).wrapping_mul(U128::from_u64s_le(0, 1)));
    // -1 * -1 == 1
    assert_eq(
        U128::one(), U128::from_u64s_le(0xffffffffffffffff, 0xffffffffffffffff).wrapping_mul(U128::from_u64s_le(0xffffffffffffffff, 0xffffffffffffffff))
    );
}
