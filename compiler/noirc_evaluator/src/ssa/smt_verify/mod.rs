//! Check that an SSA transformation preserves behavior for *all* inputs,
//! using cvc5's finite-field SMT theory — not just the one concrete input a
//! `Prover.toml`-style execution test would exercise.
//!
//! Every test here has exactly two parts: an initial SSA (parsed from text)
//! and a transformation applied to it — either a real `Ssa -> Ssa` pass
//! ([`assert_pass_preserves_behavior`]), or, for `simplify` rules
//! specifically, re-parsing the same source through the parser's own
//! simplifying path ([`assert_simplify_preserves_behavior`]). Both funnel
//! into [`assert_ssa_equivalent`], which checks the two SSAs return the same
//! values for every input.
//!
//! This shells out to the `cvc5` binary with a hand-written SMT-LIB2 script
//! rather than linking an SMT solver into the compiler: cvc5's finite-field
//! theory (needed to model Noir's prime-field `Field` type) isn't exposed by
//! the safe Rust bindings yet, only by the low-level FFI crate. Tests here are
//! skipped if `cvc5` isn't on `PATH`, so they never affect the default
//! `cargo test` run (except when running on CI).

use std::collections::HashMap;
use std::io::Write as _;
use std::process::{Command, Stdio};

use acvm::AcirField;
use acvm::FieldElement;
use num_bigint::BigUint;

use crate::assert_ssa_snapshot;

use super::Ssa;
use super::ir::basic_block::BasicBlockId;
use super::ir::dfg::DataFlowGraph;
use super::ir::instruction::{Binary, BinaryOp, Instruction, InstructionId, TerminatorInstruction};
use super::ir::value::{Value, ValueId};

/// The values a block's `Return` terminator returns. Panics on any other
/// terminator kind — control flow is out of scope for this prototype.
fn return_values(dfg: &DataFlowGraph, block: BasicBlockId) -> &[ValueId] {
    match dfg[block].unwrap_terminator() {
        TerminatorInstruction::Return { return_values, .. } => return_values,
        other => panic!("smt_verify: only Return terminators are supported, got {other:?}"),
    }
}

/// Translates SSA values reachable from one function's return values into
/// SMT-LIB2, memoizing each value to a name the first time it's seen so a
/// value referenced from multiple places is declared once and referenced by
/// name everywhere else, matching the DAG structure of real SSA. Re-expanding
/// a shared value's definition at every use site instead would blow up
/// exponentially in the number of instructions that reuse a value.
struct Encoder<'a> {
    dfg: &'a DataFlowGraph,
    /// Distinguishes this side's instruction names from the other side's
    /// when both are concatenated into one script (e.g. "before"/"after").
    prefix: &'a str,
    /// Every value already translated, mapped to the name standing in for
    /// it. Seeded up front with this function's parameters.
    names: HashMap<ValueId, String>,
    /// One `(declare-const ...)` + `(assert (= ...))` pair per instruction
    /// translated so far, in the order they were first encoded.
    declarations: Vec<String>,
}

impl<'a> Encoder<'a> {
    /// `params[i]` is the shared free-variable name for parameter `i` — the
    /// same names are used on both sides being compared, which is what lets
    /// `assert_ssa_equivalent` treat them as the same inputs.
    fn new(
        dfg: &'a DataFlowGraph,
        prefix: &'a str,
        block: BasicBlockId,
        params: &[String],
    ) -> Self {
        let names = dfg[block].parameters().iter().copied().zip(params.iter().cloned()).collect();
        Self { dfg, prefix, names, declarations: Vec::new() }
    }

    /// Returns the name standing in for `value`. The first time a given
    /// value is seen, this computes its term, records a declaration for it,
    /// and caches the name; every later call for the same value is an O(1)
    /// lookup instead of re-expanding its definition.
    fn encode_value(&mut self, value: ValueId) -> String {
        if let Some(name) = self.names.get(&value) {
            return name.clone();
        }

        let instruction = match &self.dfg[value] {
            Value::NumericConstant { constant, .. } => {
                return format!("(as ff{} FF)", field_to_decimal(*constant));
            }
            Value::Instruction { instruction, .. } => *instruction,
            other => panic!("smt_verify: don't know how to encode value {other:?} as an SMT term"),
        };

        let term = self.encode_instruction(instruction);
        let name = format!("{}_{}", self.prefix, self.names.len());
        self.declarations.push(format!("(declare-const {name} FF)"));
        self.declarations.push(format!("(assert (= {name} {term}))"));
        self.names.insert(value, name.clone());
        name
    }

    /// Translates a single instruction into an SMT-LIB2 term, from its
    /// actual `lhs`/`rhs`/`operator` fields.
    fn encode_instruction(&mut self, instruction: InstructionId) -> String {
        let (lhs, rhs, operator) = match &self.dfg[instruction] {
            Instruction::Binary(Binary { lhs, rhs, operator }) => (*lhs, *rhs, *operator),
            other => panic!("smt_verify: encoding for instruction {other:?} is not implemented"),
        };

        let lhs = self.encode_value(lhs);
        let rhs = self.encode_value(rhs);
        match operator {
            BinaryOp::Add { .. } => format!("(ff.add {lhs} {rhs})"),
            BinaryOp::Sub { .. } => format!("(ff.add {lhs} (ff.neg {rhs}))"),
            BinaryOp::Mul { .. } => format!("(ff.mul {lhs} {rhs})"),
            other => {
                panic!("smt_verify: encoding for binary operator {other:?} is not implemented")
            }
        }
    }
}

/// SMT-LIB2 decimal representation of a field element (non-negative, unlike
/// `FieldElement`'s own `Display`, which prints small negative representations).
fn field_to_decimal(value: FieldElement) -> String {
    BigUint::from_bytes_be(&value.to_be_bytes()).to_string()
}

fn field_modulus_decimal() -> String {
    FieldElement::modulus().to_string()
}

/// Checks whether `before` and `after` return the same values for every
/// input, or `None` if `cvc5` isn't installed. Parameters are unified by
/// position (see [`Encoder::new`]) since `before` and `after` are separate
/// `Ssa` graphs with unrelated internal ids.
fn ssa_equivalent(before: &Ssa, after: &Ssa) -> Option<bool> {
    let before_fn = &before.functions[&before.main_id];
    let after_fn = &after.functions[&after.main_id];
    let before_block = before_fn.entry_block();
    let after_block = after_fn.entry_block();

    let params: Vec<String> =
        (0..before_fn.dfg[before_block].parameters().len()).map(|i| format!("p{i}")).collect();
    let param_decls = params.iter().map(|p| format!("(declare-const {p} FF)"));

    let mut before_enc = Encoder::new(&before_fn.dfg, "before", before_block, &params);
    let before_terms: Vec<String> = return_values(&before_fn.dfg, before_block)
        .to_vec()
        .into_iter()
        .map(|value| before_enc.encode_value(value))
        .collect();

    let mut after_enc = Encoder::new(&after_fn.dfg, "after", after_block, &params);
    let after_terms: Vec<String> = return_values(&after_fn.dfg, after_block)
        .to_vec()
        .into_iter()
        .map(|value| after_enc.encode_value(value))
        .collect();

    assert_eq!(
        before_terms.len(),
        after_terms.len(),
        "`before` and `after` must return the same number of values"
    );

    let mismatches: Vec<String> = before_terms
        .iter()
        .zip(&after_terms)
        .map(|(before, after)| format!("(distinct {before} {after})"))
        .collect();
    let goal = match mismatches.as_slice() {
        [only] => only.clone(),
        _ => format!("(or {})", mismatches.join(" ")),
    };

    let mut script = format!(
        "(set-logic QF_FF)\n(define-sort FF () (_ FiniteField {}))\n",
        field_modulus_decimal()
    );
    for line in param_decls.chain(before_enc.declarations).chain(after_enc.declarations) {
        script.push_str(&line);
        script.push('\n');
    }
    script.push_str(&format!("(assert {goal})\n(check-sat)\n"));

    run_cvc5(&script)
}

/// Whether tests are running in CI, per this repo's own convention
/// (`justfile`'s `ci :=` line checks the same variable the same way).
fn is_ci() -> bool {
    matches!(std::env::var("CI").as_deref(), Ok("true") | Ok("1"))
}

/// Runs `cvc5` on a full SMT-LIB2 script and returns whether it reported
/// `unsat` (`Some(true)`), `sat` (`Some(false)`), or `None` if `cvc5` isn't
/// installed. A missing `cvc5` is only a silent skip locally — in CI it's a
/// hard failure, since a misconfigured install step would otherwise leave CI
/// green having never actually checked anything.
fn run_cvc5(script: &str) -> Option<bool> {
    let Ok(mut child) = Command::new("cvc5")
        .arg("--lang=smt2")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        assert!(
            !is_ci(),
            "smt_verify: `cvc5` not found on PATH in CI. It should have been \
             installed by the CI workflow (see .github/workflows/test-rust-workspace.yml) \
             — this must not silently skip in CI."
        );
        eprintln!("skipping smt_verify test: `cvc5` not found on PATH");
        return None;
    };

    child
        .stdin
        .take()
        .expect("child was spawned with a piped stdin")
        .write_all(script.as_bytes())
        .expect("failed to write SMT-LIB2 script to cvc5's stdin");

    let output = child.wait_with_output().expect("failed to read cvc5's output");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let verdict = stdout.lines().next().unwrap_or("").trim();

    match verdict {
        "unsat" => Some(true),
        "sat" => Some(false),
        other => panic!("unexpected cvc5 output: {other:?}\nfull output:\n{stdout}"),
    }
}

/// Renders one SSA function's translation to SMT-LIB2 as a single string:
/// parameter and instruction declarations, then its return term(s). Omits
/// the boilerplate (`set-logic`, `define-sort`, `check-sat`) that's identical
/// for every function and not specific to what's being translated.
fn encode_to_smt_text(src: &str) -> String {
    let ssa = Ssa::from_str(src).expect("hand-written SSA text must parse");
    let function = &ssa.functions[&ssa.main_id];
    let block = function.entry_block();

    let params: Vec<String> =
        (0..function.dfg[block].parameters().len()).map(|i| format!("p{i}")).collect();
    let mut lines: Vec<String> = params.iter().map(|p| format!("(declare-const {p} FF)")).collect();

    let mut encoder = Encoder::new(&function.dfg, "t", block, &params);
    let return_terms: Vec<String> = return_values(&function.dfg, block)
        .to_vec()
        .into_iter()
        .map(|value| encoder.encode_value(value))
        .collect();

    lines.extend(encoder.declarations);
    lines.push(format!("return: {}", return_terms.join(", ")));
    lines.join("\n")
}

/// Asserts `before` and `after` return the same values for every input.
/// Does nothing if `cvc5` isn't installed.
fn assert_ssa_equivalent(before: &Ssa, after: &Ssa) {
    if let Some(equivalent) = ssa_equivalent(before, after) {
        assert!(equivalent, "cvc5 found inputs where `before` and `after` disagree");
    }
}

/// For real SSA passes: parses `src` once for `before`, once fresh for
/// `after`, applies `transform`, checks the two are equivalent, and returns
/// `after` so the caller can additionally snapshot what the transformation
/// produced.
fn assert_pass_preserves_behavior(src: &str, transform: impl FnOnce(Ssa) -> Ssa) -> Ssa {
    let before = Ssa::from_str(src).expect("hand-written SSA text must parse");
    let after = transform(Ssa::from_str(src).expect("hand-written SSA text must parse"));
    assert_ssa_equivalent(&before, &after);
    after
}

/// For `simplify` rules specifically: the "transformation" is the parser's
/// own simplifying path over the same source, not a `Ssa -> Ssa` function —
/// `Ssa::from_str` leaves instructions unsimplified, `Ssa::from_str_simplifying`
/// runs the real `simplify` entry point (`ir/dfg/simplify.rs`, the same one
/// every SSA pass goes through) as each instruction is inserted. Returns
/// `after` so the caller can additionally snapshot what it simplified to.
fn assert_simplify_preserves_behavior(src: &str) -> Ssa {
    let before = Ssa::from_str(src).expect("hand-written SSA text must parse");
    let after = Ssa::from_str_simplifying(src).expect("hand-written SSA text must parse");
    assert_ssa_equivalent(&before, &after);
    after
}

mod encode_tests;

#[test]
fn sub_self_is_zero_holds_for_all_field_elements() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = sub v0, v0
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: Field):
        return Field 0
    }
    ");
}

#[test]
fn add_zero_lhs_holds_for_all_field_elements() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = add Field 0, v0
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: Field):
        return v0
    }
    ");
}

#[test]
fn add_zero_rhs_holds_for_all_field_elements() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = add v0, Field 0
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: Field):
        return v0
    }
    ");
}

#[test]
fn sub_zero_rhs_holds_for_all_field_elements() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = sub v0, Field 0
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: Field):
        return v0
    }
    ");
}

#[test]
fn mul_one_lhs_holds_for_all_field_elements() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = mul Field 1, v0
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: Field):
        return v0
    }
    ");
}

#[test]
fn mul_one_rhs_holds_for_all_field_elements() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = mul v0, Field 1
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: Field):
        return v0
    }
    ");
}

#[test]
fn mul_zero_holds_for_all_field_elements() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = mul v0, Field 0
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: Field):
        return Field 0
    }
    ");
}

#[test]
fn dead_instruction_elimination_preserves_behavior() {
    let after = assert_pass_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = add v0, v0
             v2 = add v0, v0
             return v2
         }",
        Ssa::dead_instruction_elimination,
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: Field):
        v1 = add v0, v0
        return v1
    }
    ");
}

/// Regression test for the encoder re-expanding a shared value's whole
/// definition at every use site instead of naming it once: `v1` and `v2`
/// here are each referenced twice by the instruction after them, so a
/// non-memoizing encoder would double the term size at each step (8x
/// overall) instead of growing linearly.
#[test]
fn shared_values_are_encoded_once_not_reexpanded() {
    let after = assert_pass_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = add v0, v0
             v2 = add v1, v1
             v3 = add v2, v2
             return v3
         }",
        Ssa::dead_instruction_elimination,
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: Field):
        v1 = add v0, v0
        v2 = add v1, v1
        v3 = add v2, v2
        return v3
    }
    ");
}

/// Proves the harness can actually detect a real mismatch, not just always
/// report equivalence: two SSA snippets that plainly compute different
/// things.
#[test]
fn harness_detects_a_real_mismatch() {
    let before = Ssa::from_str(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = sub v0, v0
             return v1
         }",
    )
    .unwrap();
    let after = Ssa::from_str(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = add v0, Field 1
             return v1
         }",
    )
    .unwrap();

    if let Some(equivalent) = ssa_equivalent(&before, &after) {
        assert!(!equivalent, "cvc5 should have found a counterexample");
    }
}
