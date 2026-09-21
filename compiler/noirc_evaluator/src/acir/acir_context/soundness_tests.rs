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

use super::generated_acir::GeneratedAcir;
use super::soundness::{self, Encoding, WidthExceedsField};
use super::{AcirContext, BrilligStdLib};
use crate::brillig::BrilligOptions;
use crate::ssa::ssa_gen::Ssa;

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

/// Nothing in this gadget is beyond the encoder: the one constraint with no
/// bitvector reading, the inverse `b · b⁻¹ = 1` that `inv_var` emits, is
/// rewritten to `b ≠ 0` rather than dropped.
#[test]
fn the_whole_division_gadget_is_encoded() {
    let (opcodes, _) = euclidean_division(8);
    let encoding = Encoding::new(&opcodes).expect("the gadget should be encodable at this width");
    assert_eq!(encoding.dropped(), 0);
}

/// The field algebra behind [`super::soundness`]\'s first rewrite: an
/// unconstrained `inv` can satisfy `E · inv = 1` exactly when `E` is nonzero.
/// Checked in `QF_FF`, which is quick here because the statement carries no
/// range constraints — the thing that logic handles badly.
#[test]
fn inverse_idiom_means_nonzero() {
    prove_over_the_field(
        "
        (declare-const e FF)
        (declare-const inv FF)
        (assert (= (ff.mul e inv) (as ff1 FF)))
        (assert (= e (as ff0 FF)))
        ",
    );
}

/// The second rewrite: given `z = 1 - E · inv` and `E · z = 0`, the value of
/// `z` is pinned to the indicator of `E = 0`, however `inv` is chosen.
#[test]
fn is_zero_idiom_pins_the_indicator() {
    prove_over_the_field(
        "
        (declare-const e FF)
        (declare-const inv FF)
        (declare-const z FF)
        (assert (= z (ff.add (as ff1 FF) (ff.mul (as ff-1 FF) e inv))))
        (assert (= (ff.mul e z) (as ff0 FF)))
        (assert (or (and (= e (as ff0 FF)) (not (= z (as ff1 FF))))
                    (and (not (= e (as ff0 FF))) (not (= z (as ff0 FF))))))
        ",
    );
}

/// Asserts that `body` — the negation of a lemma — has no solution over the
/// circuits\' prime field.
fn prove_over_the_field(body: &str) {
    let script = format!(
        "(set-logic QF_FF)\n(define-sort FF () (_ FiniteField {}))\n{body}\n(check-sat)\n",
        soundness::modulus()
    );
    soundness::prove(&script).assert_sound();
}

/// Signed division, the case the inverse rewrites exist for. `expand_signed_math`
/// lowers it into `Field` arithmetic that wraps modulo `p` and leans on four
/// inverse idioms; the check is that the circuit\'s return witness holds the
/// two\'s-complement result of `bvsdiv` on its inputs, for every witness the
/// constraints admit.
/// The ACIR that `expand_signed_math` plus the real lowering produce for `i8`
/// division.
fn signed_division() -> GeneratedAcir<FieldElement> {
    let ssa = Ssa::from_str(
        "
        acir(inline) fn main f0 {
          b0(v0: i8, v1: i8):
            v2 = div v0, v1
            return v2
        }
        ",
    )
    .unwrap()
    .expand_signed_math();

    let options = BrilligOptions::default();
    let brillig = ssa.to_brillig(&options);
    let (mut functions, ..) = ssa.into_acir(&brillig, &options).unwrap();
    functions.remove(0)
}

/// Each witness holds an 8-bit two\'s-complement pattern, zero-extended to the
/// encoding width, so the specification reads those low bits as signed.
fn signed_division_goal(
    encoding: &Encoding,
    lhs: Witness,
    rhs: Witness,
    result: Witness,
) -> String {
    let byte = |witness: Witness| format!("((_ extract 7 0) w{})", witness.0);
    format!(
        "(not (= w{} ((_ zero_extend {}) (bvsdiv {} {}))))",
        result.0,
        encoding.width() - 8,
        byte(lhs),
        byte(rhs)
    )
}

#[test]
fn signed_division_from_ssa_is_sound() {
    let acir = signed_division();
    let [lhs, rhs] = acir.input_witnesses[..] else { panic!("expected two inputs") };
    let [result] = acir.return_witnesses[..] else { panic!("expected one return value") };

    let mut encoding = Encoding::new(&acir.opcodes).expect("the circuit should be encodable");
    assert_eq!(encoding.dropped(), 0, "every constraint should be encoded or rewritten");
    for witness in [lhs, rhs, result] {
        encoding.declare(witness);
    }
    encoding.check(&signed_division_goal(&encoding, lhs, rhs, result)).assert_sound();
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

/// Compiles SSA source through the real lowering (`Ssa::to_brillig` +
/// `Ssa::into_acir`) and returns the ACIR of its single function. Nothing in
/// the harness decides what gets emitted; the opcodes are whatever the compiler
/// produced for that program.
fn compile(src: &str) -> GeneratedAcir<FieldElement> {
    let ssa = Ssa::from_str(src).unwrap();
    let options = BrilligOptions::default();
    let brillig = ssa.to_brillig(&options);
    let (mut functions, ..) = ssa.into_acir(&brillig, &options).unwrap();
    assert_eq!(functions.len(), 1, "multiple ACIR functions are out of scope");
    functions.remove(0)
}

/// The whole path, for a whole program: `u8` division lowered by the real
/// pipeline, then checked against `bvudiv`/`bvurem` over the circuit's declared
/// inputs and return value.
#[test]
fn unsigned_division_from_ssa_is_sound() {
    let acir = compile(
        "
        acir(inline) fn main f0 {
          b0(v0: u8, v1: u8):
            v2 = div v0, v1
            return v2
        }
        ",
    );

    let [lhs, rhs] = acir.input_witnesses[..] else { panic!("expected two inputs") };
    let [result] = acir.return_witnesses[..] else { panic!("expected one return value") };

    let mut encoding = Encoding::new(&acir.opcodes).expect("the circuit should be encodable");
    for witness in [lhs, rhs, result] {
        encoding.declare(witness);
    }
    let goal = format!("(not (= w{} (bvudiv w{} w{})))", result.0, lhs.0, rhs.0);
    encoding.check(&goal).assert_sound();
}

#[test]
fn unsigned_remainder_from_ssa_is_sound() {
    let acir = compile(
        "
        acir(inline) fn main f0 {
          b0(v0: u8, v1: u8):
            v2 = mod v0, v1
            return v2
        }
        ",
    );

    let [lhs, rhs] = acir.input_witnesses[..] else { panic!("expected two inputs") };
    let [result] = acir.return_witnesses[..] else { panic!("expected one return value") };

    let mut encoding = Encoding::new(&acir.opcodes).expect("the circuit should be encodable");
    for witness in [lhs, rhs, result] {
        encoding.declare(witness);
    }
    let goal = format!("(not (= w{} (bvurem w{} w{})))", result.0, lhs.0, rhs.0);
    encoding.check(&goal).assert_sound();
}

/// Sensitivity check for the signed query: at least one of the range
/// constraints that circuit emits has to be load-bearing, or the proof above is
/// passing for reasons unconnected to what the compiler emitted.
#[test]
fn signed_division_range_checks_are_load_bearing() {
    let acir = signed_division();
    let [lhs, rhs] = acir.input_witnesses[..] else { panic!("expected two inputs") };
    let [result] = acir.return_witnesses[..] else { panic!("expected one return value") };

    let ranged = acir
        .opcodes
        .iter()
        .filter_map(|opcode| match opcode {
            Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                input: FunctionInput::Witness(witness),
                ..
            }) => Some(*witness),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();

    let mut checked = 0;
    let mut detected = 0;
    for witness in ranged {
        let opcodes = without_range_check_on(&acir.opcodes, witness);
        let Ok(mut encoding) = Encoding::new(&opcodes) else { continue };
        for declared in [lhs, rhs, result] {
            encoding.declare(declared);
        }
        if let Some(found) = encoding
            .check(&signed_division_goal(&encoding, lhs, rhs, result))
            .found_counterexample()
        {
            checked += 1;
            detected += usize::from(found);
        }
    }
    assert!(checked == 0 || detected > 0, "no range constraint turned out to matter");
}
