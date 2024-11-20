//! Tests to show that the comptime interpreter implement blackbox functions.
use std::bigint;

/// Test that all bigint operations work in comptime.
#[test]
fn test_bigint() {
    let result: [u8] = comptime {
        let a = bigint::Secpk1Fq::from_le_bytes(&[0, 1, 2, 3, 4]);
        let b = bigint::Secpk1Fq::from_le_bytes(&[5, 6, 7, 8, 9]);
        let c = (a + b) * b / a - a;
        c.to_le_bytes()
    };
    // Do the same calculation outside comptime.
    let a = bigint::Secpk1Fq::from_le_bytes(&[0, 1, 2, 3, 4]);
    let b = bigint::Secpk1Fq::from_le_bytes(&[5, 6, 7, 8, 9]);
    let c = bigint::Secpk1Fq::from_le_bytes(result);
    assert_eq(c, (a + b) * b / a - a);
}

/// Test that to_le_radix returns an array.
#[test]
fn test_to_le_radix() {
    comptime {
        let field = 2;
        let bytes: [u8; 8] = field.to_le_radix(256);
        let _num = bigint::BigInt::from_le_bytes(bytes, bigint::bn254_fq);
    };
}
