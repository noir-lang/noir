//! End-to-end Brillig tests for truncating signed values, including negatives.
//!
//! These compile SSA to Brillig bytecode and run it on the Brillig VM, so they validate the
//! real generated code rather than the SSA interpreter's model of it. Signed inputs are passed
//! as their two's-complement field representation (e.g. `-1_i32` is `0xFFFF_FFFF = 4294967295`),
//! and signed return values come back as that same representation.

use acvm::FieldElement;

use crate::brillig::brillig_gen::tests::execute_brillig_from_ssa;

/// A bare `truncate` keeps the low `bit_size` bits of the value's representation and does no
/// sign-specific work. `-1_i32` (`0xFFFF_FFFF`) truncated to 16 bits is `0xFFFF = 65535`, still
/// typed `i32`, so it comes back as the non-negative `65535`.
#[test]
fn truncate_negative_signed_keeps_low_bits() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: i32):
        v1 = truncate v0 to 16 bits, max_bit_size: 32
        return v1
    }
    ";
    let result = execute_brillig_from_ssa(src, vec![FieldElement::from(4294967295u64)]);
    assert_eq!(result, vec![FieldElement::from(65535u64)]);
}

/// `x as i16` for `x = -1_i32` lowers to `truncate` then `cast`. The truncate yields `65535`,
/// the cast reinterprets those 16 bits as `i16`, recovering `-1_i16` (representation `65535`).
#[test]
fn truncate_then_cast_recovers_negative() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: i32):
        v1 = truncate v0 to 16 bits, max_bit_size: 32
        v2 = cast v1 as i16
        return v2
    }
    ";
    let result = execute_brillig_from_ssa(src, vec![FieldElement::from(4294967295u64)]);
    assert_eq!(result, vec![FieldElement::from(65535u64)]);
}

/// A negative whose low 16 bits are zero wraps to `0`: `-65536_i32 as i16 == 0`.
/// `-65536_i32` is `0xFFFF_0000 = 4294901760`.
#[test]
fn truncate_then_cast_negative_wraps_to_zero() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: i32):
        v1 = truncate v0 to 16 bits, max_bit_size: 32
        v2 = cast v1 as i16
        return v2
    }
    ";
    let result = execute_brillig_from_ssa(src, vec![FieldElement::from(4294901760u64)]);
    assert_eq!(result, vec![FieldElement::from(0u64)]);
}

/// An in-range negative round-trips exactly: `-100_i32 as i16 == -100`.
/// `-100_i32` is `0xFFFF_FF9C = 4294967196`; `-100_i16` is `65536 - 100 = 65436`.
#[test]
fn truncate_then_cast_preserves_in_range_negative() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: i32):
        v1 = truncate v0 to 16 bits, max_bit_size: 32
        v2 = cast v1 as i16
        return v2
    }
    ";
    let result = execute_brillig_from_ssa(src, vec![FieldElement::from(4294967196u64)]);
    assert_eq!(result, vec![FieldElement::from(65436u64)]);
}
