//! End-to-end Brillig tests for truncating signed values, including negatives.
//!
//! These compile SSA to Brillig bytecode and run it on the Brillig VM, so they validate the
//! real generated code rather than the SSA interpreter's model of it. Signed inputs are passed
//! as their two's-complement field representation (e.g. `-1_i32` is `0xFFFF_FFFF = 4294967295`),
//! and signed return values come back as that same representation.

use acvm::{AcirField, FieldElement, acir::brillig::Opcode as BrilligOpcode};

use crate::brillig::{
    BrilligOptions,
    brillig_gen::tests::{
        execute_brillig_from_ssa, execute_brillig_from_ssa_with_options, linked_bytecode_from_ssa,
    },
    brillig_ir::{LayoutConfig, registers::MAX_SCRATCH_SPACE},
};

/// The number of cast opcodes `src` emits, not counting those the entry point uses for a `main`
/// with the same parameters: `params` is the SSA parameter list of that `main`.
fn cast_opcodes_in_body(src: &str, params: &str) -> usize {
    let count = |src: &str| {
        linked_bytecode_from_ssa(src, &BrilligOptions::default())
            .iter()
            .filter(|opcode| matches!(opcode, BrilligOpcode::Cast { .. }))
            .count()
    };
    let empty_body = format!("brillig(inline) fn main f0 {{ b0({params}): return }}");
    count(src) - count(&empty_body)
}

/// `2^128 + 5`: a field whose low 128 bits differ from the value itself.
fn field_wider_than_u128() -> FieldElement {
    FieldElement::from(2u128).pow(&FieldElement::from(128u128)) + FieldElement::from(5u128)
}

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

/// `x as u128` for a field `x` needs a single opcode, as a Brillig cast already truncates.
#[test]
fn truncate_then_same_width_cast_of_field_is_one_cast() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: Field):
        v1 = truncate v0 to 128 bits, max_bit_size: 254
        v2 = cast v1 as u128
        return v2
    }
    ";
    assert_eq!(cast_opcodes_in_body(src, "v0: Field"), 1);
    let result = execute_brillig_from_ssa(src, vec![field_wider_than_u128()]);
    assert_eq!(result, vec![FieldElement::from(5u128)]);
}

/// `x as u64` for a `u128` `x` needs a single opcode as well.
#[test]
fn truncate_then_same_width_cast_of_integer_is_one_cast() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u128):
        v1 = truncate v0 to 64 bits, max_bit_size: 128
        v2 = cast v1 as u64
        return v2
    }
    ";
    assert_eq!(cast_opcodes_in_body(src, "v0: u128"), 1);
    let input = (1u128 << 64) + 7;
    let result = execute_brillig_from_ssa(src, vec![FieldElement::from(input)]);
    assert_eq!(result, vec![FieldElement::from(7u128)]);
}

/// When the truncated value is also used on its own, both results are produced with two opcodes,
/// and each has the right value.
#[test]
fn truncate_result_with_other_uses_is_two_casts() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: Field):
        v1 = truncate v0 to 128 bits, max_bit_size: 254
        v2 = cast v1 as u128
        v3 = add v1, Field 1
        return v2, v3
    }
    ";
    assert_eq!(cast_opcodes_in_body(src, "v0: Field"), 2);
    let result = execute_brillig_from_ssa(src, vec![field_wider_than_u128()]);
    assert_eq!(result, vec![FieldElement::from(5u128), FieldElement::from(6u128)]);
}

/// A truncate result used in another block has to outlive the cast.
#[test]
fn truncate_result_used_in_later_block() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: Field):
        v1 = truncate v0 to 128 bits, max_bit_size: 254
        v2 = cast v1 as u128
        jmp b1()
      b1():
        return v1, v2
    }
    ";
    let result = execute_brillig_from_ssa(src, vec![field_wider_than_u128()]);
    assert_eq!(result, vec![FieldElement::from(5u128), FieldElement::from(5u128)]);
}

/// A cast that does not directly follow the truncate gets the regular lowering.
#[test]
fn truncate_and_cast_with_instruction_in_between() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: Field):
        v1 = truncate v0 to 128 bits, max_bit_size: 254
        v2 = add v0, Field 1
        v3 = cast v1 as u128
        return v2, v3
    }
    ";
    assert_eq!(cast_opcodes_in_body(src, "v0: Field"), 3);
    let input = field_wider_than_u128();
    let result = execute_brillig_from_ssa(src, vec![input]);
    assert_eq!(result, vec![input + FieldElement::one(), FieldElement::from(5u128)]);
}

/// Both results of a truncate and cast stay correct when registers are scarce enough for values
/// to get spilled.
#[test]
fn truncate_then_cast_under_register_pressure() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: Field, v1: u32):
        v2 = truncate v0 to 128 bits, max_bit_size: 254
        v3 = cast v2 as u128
        v4 = unchecked_add v1, u32 1
        v5 = unchecked_add v1, u32 2
        v6 = unchecked_add v4, v5
        v7 = cast v6 as Field
        v8 = add v2, v7
        v9 = cast v3 as Field
        v10 = add v8, v9
        return v10
    }
    ";
    let options = BrilligOptions {
        layout: LayoutConfig::new(6, 16, MAX_SCRATCH_SPACE),
        ..BrilligOptions::default()
    };
    let spill_stores = linked_bytecode_from_ssa(src, &options)
        .iter()
        .filter(|opcode| matches!(opcode, BrilligOpcode::Store { .. }))
        .count();
    assert!(spill_stores > 0, "expected the register pressure to cause spills");

    let calldata = vec![field_wider_than_u128(), FieldElement::from(1u128)];
    let result = execute_brillig_from_ssa_with_options(src, calldata, &options);
    // 5 (truncated) + 5 (sum of the u32s) + 5 (cast)
    assert_eq!(result, vec![FieldElement::from(15u128)]);
}

/// A cast result passed to a block parameter can share its register, while the parameter is the
/// operand of the truncate right before the cast.
#[test]
fn adjacent_truncate_then_cast_of_block_parameter() {
    let src = "
    brillig(inline) fn main f0 {
      b0(v0: u128):
        jmp b1(v0, u32 0)
      b1(v1: u128, v2: u32):
        v3 = eq v2, u32 2
        jmpif v3 then: b3(), else: b2()
      b2():
        v4 = truncate v1 to 64 bits, max_bit_size: 128
        v5 = cast v4 as u64
        v6 = cast v5 as u128
        v7 = unchecked_add v2, u32 1
        jmp b1(v6, v7)
      b3():
        return v1
    }
    ";
    let input = (1u128 << 64) + 7;
    let result = execute_brillig_from_ssa(src, vec![FieldElement::from(input)]);
    assert_eq!(result, vec![FieldElement::from(7u128)]);
}
