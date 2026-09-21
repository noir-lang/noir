//! Soundness of the ACIR gadgets that lower integer arithmetic, checked against
//! the opcodes the gadgets really emit.
//!
//! Each test drives the real [`AcirContext`] method, takes the resulting opcode
//! list, and hands it to [`super::soundness`], which asks whether *any* witness
//! satisfies those opcodes while disagreeing with the specification. Nothing
//! here describes what the gadget is believed to emit; the opcodes are whatever
//! the compiler produced.
//!
//! The `..._is_detected` tests are tests of the checker rather than of the
//! compiler. They delete one opcode the gadget emitted and require the checker
//! to notice. Without them a passing soundness result would be worth nothing: an
//! encoding that is accidentally stronger than the opcodes it models proves
//! everything and catches nothing, which is a failure mode that looks exactly
//! like success.

use acvm::acir::circuit::Opcode;
use acvm::acir::circuit::opcodes::{BlackBoxFuncCall, FunctionInput};
use acvm::acir::native_types::Witness;
use acvm::{AcirField, FieldElement};

use super::soundness::{Encoding, WidthExceedsField};
use super::{AcirContext, BrilligStdLib};

/// The opcodes `euclidean_division_var` emits for `a / b` on `bit_size`-bit
/// unsigned operands, with the predicate active, plus the witnesses for
/// `a`, `b`, the quotient and the remainder.
fn euclidean_division(bit_size: u32) -> (Vec<Opcode<FieldElement>>, [Witness; 4]) {
    let mut context = AcirContext::<FieldElement>::new(BrilligStdLib::default());
    let one = context.add_constant(FieldElement::one());

    // Both operands carry the range constraint their `uN` type implies; in a
    // real compilation that comes from the caller, so the harness adds it here.
    let a = context.add_variable();
    let b = context.add_variable();
    let a = context.range_constrain_var(a, bit_size, None, one).unwrap();
    let b = context.range_constrain_var(b, bit_size, None, one).unwrap();

    let (quotient, remainder) = context.euclidean_division_var(a, b, bit_size, one).unwrap();

    let witnesses = [a, b, quotient, remainder].map(|var| context.var_to_witness(var).unwrap());
    (context.finish(Vec::new(), Vec::new()).opcodes, witnesses)
}

/// `quotient` and `remainder` are anything other than the true ones.
fn not_the_true_quotient_and_remainder(witnesses: [Witness; 4]) -> String {
    let [a, b, quotient, remainder] = witnesses.map(|witness| format!("w{}", witness.0));
    format!("(or (not (= {quotient} (bvudiv {a} {b}))) (not (= {remainder} (bvurem {a} {b}))))")
}

fn encode(opcodes: &[Opcode<FieldElement>], witnesses: [Witness; 4]) -> Encoding {
    let mut encoding =
        Encoding::new(opcodes).expect("the gadget should be encodable at this width");
    for witness in witnesses {
        encoding.declare(witness);
    }
    encoding
}

/// Removes the range constraint on `target`, as if the gadget had forgotten to
/// emit it.
fn without_range_check_on(
    opcodes: &[Opcode<FieldElement>],
    target: Witness,
) -> Vec<Opcode<FieldElement>> {
    let removed = opcodes
        .iter()
        .filter(|opcode| {
            !matches!(
                opcode,
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                    input: FunctionInput::Witness(witness),
                    ..
                }) if *witness == target
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    assert!(removed.len() < opcodes.len(), "no range check on {target:?} to remove");
    removed
}

#[test]
fn euclidean_division_is_sound_for_u8() {
    let (opcodes, witnesses) = euclidean_division(8);
    let encoding = encode(&opcodes, witnesses);
    encoding.check(&not_the_true_quotient_and_remainder(witnesses)).assert_sound();
}

/// The same property at 16 bits, which the solver takes about 16 minutes to
/// settle (992 s measured), so it belongs in a nightly run rather than in the
/// per-PR suite. Needs `NOIR_SOUNDNESS_TIMEOUT_MS=2400000`, above the ten
/// minutes allowed by default.
#[test]
#[ignore = "~16 min; run with --ignored and NOIR_SOUNDNESS_TIMEOUT_MS=2400000"]
fn euclidean_division_is_sound_for_u16() {
    let (opcodes, witnesses) = euclidean_division(16);
    let encoding = encode(&opcodes, witnesses);
    encoding.check(&not_the_true_quotient_and_remainder(witnesses)).assert_sound();
}

#[test]
fn dropping_the_remainder_range_check_is_detected() {
    let (opcodes, witnesses) = euclidean_division(8);
    let opcodes = without_range_check_on(&opcodes, witnesses[3]);
    let encoding = encode(&opcodes, witnesses);
    encoding.check(&not_the_true_quotient_and_remainder(witnesses)).assert_unsound();
}

#[test]
fn dropping_the_quotient_range_check_is_detected() {
    let (opcodes, witnesses) = euclidean_division(8);
    let opcodes = without_range_check_on(&opcodes, witnesses[2]);
    let encoding = encode(&opcodes, witnesses);
    encoding.check(&not_the_true_quotient_and_remainder(witnesses)).assert_unsound();
}

#[test]
fn truncate_is_sound() {
    for (bit_size, max_bit_size) in [(3, 6), (8, 16), (32, 64)] {
        let mut context = AcirContext::<FieldElement>::new(BrilligStdLib::default());
        let one = context.add_constant(FieldElement::one());
        let value = context.add_variable();
        let value = context.range_constrain_var(value, max_bit_size, None, one).unwrap();
        let truncated = context.truncate_var(value, bit_size, max_bit_size).unwrap();

        let value_witness = context.var_to_witness(value).unwrap();
        let truncated_witness = context.var_to_witness(truncated).unwrap();
        let opcodes = context.finish(Vec::new(), Vec::new()).opcodes;

        let mut encoding = Encoding::new(&opcodes).expect("truncate should be encodable");
        encoding.declare(value_witness);
        encoding.declare(truncated_witness);

        let mask = (1_u128 << bit_size) - 1;
        let goal = format!(
            "(not (= w{} (bvand w{} (_ bv{mask} {}))))",
            truncated_witness.0,
            value_witness.0,
            encoding.width()
        );
        encoding.check(&goal).assert_sound();
    }
}

/// At 128 bits the quotient-times-divisor product needs more room than the
/// field has, so no bitvector width can stand in for it and the checker says so
/// instead of verifying something weaker. `euclidean_division_var` reaches the
/// same conclusion by a different route: it special-cases `bit_size == 128` and
/// calls `unreachable!("overflow in unbounded division")` otherwise.
#[test]
fn u128_division_is_out_of_range_of_this_encoding() {
    let (opcodes, _) = euclidean_division(128);
    assert!(matches!(Encoding::new(&opcodes), Err(WidthExceedsField { .. })));
}

/// The inverse constraint `b · b⁻¹ = 1` that `inv_var` emits has no faithful
/// bitvector form — modulo `2^W` only odd values are invertible — so it is
/// dropped rather than approximated. Everything else in the gadget is encoded.
#[test]
fn only_the_inverse_constraint_is_dropped() {
    let (opcodes, _) = euclidean_division(8);
    let encoding = Encoding::new(&opcodes).expect("the gadget should be encodable at this width");
    assert_eq!(encoding.dropped(), 1);
}

/// `bound_constraint_with_offset` claims that `lhs < rhs`, and gets there by
/// range-constraining `rhs - (lhs + 1)`: when `lhs >= rhs` that difference
/// underflows to a value far too large to pass. The argument only holds while
/// `bits + 1 < log2 p`, which the function asserts at runtime rather than
/// proves.
#[test]
fn bound_constraint_with_offset_really_bounds() {
    let bit_size = 8;
    let mut context = AcirContext::<FieldElement>::new(BrilligStdLib::default());
    let one = context.add_constant(FieldElement::one());

    let lhs = context.add_variable();
    let rhs = context.add_variable();
    let lhs = context.range_constrain_var(lhs, bit_size, None, one).unwrap();
    let rhs = context.range_constrain_var(rhs, bit_size, None, one).unwrap();
    context.bound_constraint_with_offset(lhs, rhs, one, bit_size, one).unwrap();

    let lhs_witness = context.var_to_witness(lhs).unwrap();
    let rhs_witness = context.var_to_witness(rhs).unwrap();
    let opcodes = context.finish(Vec::new(), Vec::new()).opcodes;

    let mut encoding = Encoding::new(&opcodes).expect("the gadget should be encodable");
    encoding.declare(lhs_witness);
    encoding.declare(rhs_witness);

    let goal = format!("(bvuge w{} w{})", lhs_witness.0, rhs_witness.0);
    encoding.check(&goal).assert_sound();
}
