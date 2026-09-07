//! Checks that an SSA function and its *real, compiler-generated* ACIR (not
//! hand-written ACIR text) compute the same thing, for every input — the
//! SSA<->ACIR counterpart to [`super::ssa`]'s SSA-pass equivalence checking.
//! Reuses the same comparator shape: unify inputs by position, encode both
//! sides, assert their outputs can never disagree, ask cvc5.
//!
//! Scoped, for now, to a single ACIR function (no calls between functions),
//! matching the same "single function, straight-line" scope the rest of
//! `smt_verify` has had from the start.

use acvm::FieldElement;

use crate::acir::GeneratedAcir;
use crate::brillig::BrilligOptions;
use crate::ssa::Ssa;

use super::acir::encode_generated_acir;
use super::ssa::encode_function;
use super::{field_modulus_decimal, run_cvc5};

/// Compiles `ssa` to ACIR via the real pipeline entry point (`Ssa::to_brillig`
/// and `Ssa::into_acir`) and returns its one function's `GeneratedAcir`.
/// Takes `ssa` by value since `Ssa::into_acir` consumes it, so callers must
/// encode anything they still need from the SSA side first. Panics if
/// compilation fails or produces anything other than exactly one ACIR
/// function; calls between multiple ACIR functions are out of scope here.
fn compile_to_generated_acir(ssa: Ssa) -> GeneratedAcir<FieldElement> {
    let brillig_options = BrilligOptions::default();
    let brillig = ssa.to_brillig(&brillig_options);
    let (mut acir_functions, _, _) =
        ssa.into_acir(&brillig, &brillig_options).expect("SSA must compile to ACIR");
    assert_eq!(
        acir_functions.len(),
        1,
        "expected exactly one ACIR function; multiple functions/calls are out of scope for now"
    );
    acir_functions.remove(0)
}

/// Checks whether an already-encoded SSA side (`params`, its declarations
/// and return terms) and `generated_acir` return the same values for every
/// input, or `None` if `cvc5` isn't installed. Each SSA parameter and its
/// corresponding ACIR input witness keep their own real names (`v{id}` and
/// `w{index}` respectively) and are linked by an explicit equality assertion
/// rather than a shared name — semantically the same either way, but every
/// name in the script then traces directly back to something real on its
/// own side. Takes the SSA side pre-encoded (rather than a `&Ssa`) so both
/// the real compiled-ACIR path and a test injecting a deliberately-wrong
/// `GeneratedAcir` can share this one check.
fn ssa_acir_equivalent(
    params: &[String],
    ssa_decls: Vec<String>,
    ssa_terms: &[String],
    generated_acir: &GeneratedAcir<FieldElement>,
) -> Option<bool> {
    assert_eq!(
        generated_acir.input_witnesses.len(),
        params.len(),
        "the ACIR function must have the same number of inputs as the SSA function"
    );
    let (acir_decls, acir_terms) = encode_generated_acir(generated_acir);

    assert_eq!(
        ssa_terms.len(),
        acir_terms.len(),
        "SSA and ACIR must return the same number of values"
    );

    let mismatches: Vec<String> = ssa_terms
        .iter()
        .zip(&acir_terms)
        .map(|(ssa, acir)| format!("(distinct {ssa} {acir})"))
        .collect();
    let goal = match mismatches.as_slice() {
        [only] => only.clone(),
        _ => format!("(or {})", mismatches.join(" ")),
    };

    let param_decls =
        params.iter().map(|p| format!("(declare-const {p} FF)")).collect::<Vec<String>>();
    let links: Vec<String> = params
        .iter()
        .zip(&generated_acir.input_witnesses)
        .map(|(param, witness)| format!("(assert (= {param} {witness}))"))
        .collect();

    let mut script = format!(
        "(set-logic QF_FF)\n(define-sort FF () (_ FiniteField {}))\n",
        field_modulus_decimal()
    );
    for line in param_decls.into_iter().chain(ssa_decls).chain(acir_decls).chain(links) {
        script.push_str(&line);
        script.push('\n');
    }
    script.push_str(&format!("(assert {goal})\n(check-sat)\n"));

    run_cvc5(&script)
}

/// The SSA side's real parameter names (`v{id}`, each parameter's own
/// `ValueId`) and the `Encoder`-produced declarations/return terms for
/// `ssa`'s main function — extracted before `ssa` is consumed by real ACIR
/// compilation.
fn encode_ssa_side(ssa: &Ssa) -> (Vec<String>, Vec<String>, Vec<String>) {
    let function = &ssa.functions[&ssa.main_id];
    let block = function.entry_block();
    let params: Vec<String> =
        function.dfg[block].parameters().iter().map(|value_id| value_id.to_string()).collect();
    let (declarations, return_terms) = encode_function(ssa, "ssa_", &params);
    (params, declarations, return_terms)
}

/// Parses `ssa_src`, compiles it to ACIR via the real pipeline, and asserts
/// the two return the same values for every input. Does nothing if `cvc5`
/// isn't installed.
fn assert_ssa_acir_equivalent(ssa_src: &str) {
    let ssa = Ssa::from_str(ssa_src).expect("hand-written SSA text must parse");
    let (params, ssa_decls, ssa_terms) = encode_ssa_side(&ssa);
    let generated_acir = compile_to_generated_acir(ssa);

    if let Some(equivalent) = ssa_acir_equivalent(&params, ssa_decls, &ssa_terms, &generated_acir) {
        assert!(equivalent, "cvc5 found inputs where the SSA and its compiled ACIR disagree");
    }
}

#[test]
fn add_compiles_to_equivalent_acir() {
    assert_ssa_acir_equivalent(
        "acir(inline) fn main f0 {
           b0(v0: Field, v1: Field):
             v2 = add v0, v1
             return v2
         }",
    );
}

/// Proves the comparator can actually detect a real mismatch, not just
/// always report equivalence: takes the real, correctly-compiled ACIR for
/// `v2 = add v0, v1`, then points its return witness at a free input
/// instead of the real sum — simulating an under-constraining codegen bug,
/// since that witness is never asserted to equal `p0 + p1` and so can
/// disagree with the true output.
#[test]
fn comparator_detects_a_real_mismatch() {
    let ssa_src = "acir(inline) fn main f0 {
           b0(v0: Field, v1: Field):
             v2 = add v0, v1
             return v2
         }";
    let ssa = Ssa::from_str(ssa_src).expect("hand-written SSA text must parse");
    let (params, ssa_decls, ssa_terms) = encode_ssa_side(&ssa);

    let mut generated_acir = compile_to_generated_acir(ssa);
    generated_acir.return_witnesses = vec![generated_acir.input_witnesses[0]];

    if let Some(equivalent) = ssa_acir_equivalent(&params, ssa_decls, &ssa_terms, &generated_acir) {
        assert!(!equivalent, "cvc5 should have found a counterexample");
    }
}
