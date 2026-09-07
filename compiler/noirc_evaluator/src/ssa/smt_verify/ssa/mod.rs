//! Translates SSA into SMT-LIB2 and checks whether an SSA transformation
//! preserves behavior for *all* inputs — either a real `Ssa -> Ssa` pass
//! ([`assert_pass_preserves_behavior`]), or, for `simplify` rules
//! specifically, re-parsing the same source through the parser's own
//! simplifying path ([`assert_simplify_preserves_behavior`]). Both funnel
//! into [`assert_ssa_equivalent`], which checks the two SSAs return the same
//! values for every input.

use std::collections::HashMap;

use crate::ssa::Ssa;
use crate::ssa::ir::basic_block::BasicBlockId;
use crate::ssa::ir::dfg::DataFlowGraph;
use crate::ssa::ir::instruction::{
    Binary, BinaryOp, Instruction, InstructionId, TerminatorInstruction,
};
use crate::ssa::ir::types::Type;
use crate::ssa::ir::value::{Value, ValueId};

use super::{field_modulus_decimal, field_to_decimal, run_cvc5};

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
        match &self.dfg[instruction] {
            Instruction::Binary(Binary { lhs, rhs, operator }) => {
                let (lhs, rhs, operator) = (*lhs, *rhs, *operator);
                if operator == BinaryOp::And {
                    self.assert_boolean(lhs, "And");
                    self.assert_boolean(rhs, "And");
                }
                let lhs = self.encode_value(lhs);
                let rhs = self.encode_value(rhs);
                match operator {
                    BinaryOp::Add { .. } => format!("(ff.add {lhs} {rhs})"),
                    BinaryOp::Sub { .. } => format!("(ff.add {lhs} (ff.neg {rhs}))"),
                    BinaryOp::Mul { .. } => format!("(ff.mul {lhs} {rhs})"),
                    BinaryOp::Eq => format!("(ite (= {lhs} {rhs}) (as ff1 FF) (as ff0 FF))"),
                    // Valid because both operands are asserted boolean above:
                    // AND on {0,1} values is exactly multiplication.
                    BinaryOp::And => format!("(ff.mul {lhs} {rhs})"),
                    other => {
                        panic!(
                            "smt_verify: encoding for binary operator {other:?} is not implemented"
                        )
                    }
                }
            }
            Instruction::Not(value) => {
                let value = *value;
                self.assert_boolean(value, "Not");
                let value = self.encode_value(value);
                // Valid because `value` is asserted boolean above: NOT on a
                // {0,1} value is `1 - x`.
                format!("(ff.add (as ff1 FF) (ff.neg {value}))")
            }
            other => panic!("smt_verify: encoding for instruction {other:?} is not implemented"),
        }
    }

    /// Panics if `value`'s SSA type isn't boolean. `And` and `Not` are only
    /// encoded correctly for boolean operands (AND/NOT on `{0,1}` values are
    /// multiplication/`1-x`; that's not a valid encoding of bitwise
    /// AND/complement on wider integer types), so this guards against
    /// silently mistranslating an out-of-scope case instead of catching it.
    fn assert_boolean(&self, value: ValueId, context: &str) {
        assert!(
            *self.dfg.type_of_value(value) == Type::bool(),
            "smt_verify: {context} is only encoded for boolean operands, got {:?}",
            self.dfg.type_of_value(value)
        );
    }
}

/// Encodes one `Ssa`'s main function into its `Encoder`-produced
/// declarations and return terms, given `params[i]` as the shared
/// free-variable name for parameter `i` (see [`Encoder::new`]). Exposed for
/// reuse by the SSA<->ACIR comparator, which needs exactly these two things
/// to compare against an ACIR encoding sharing the same parameter names.
pub(super) fn encode_function(
    ssa: &Ssa,
    prefix: &str,
    params: &[String],
) -> (Vec<String>, Vec<String>) {
    let function = &ssa.functions[&ssa.main_id];
    let block = function.entry_block();

    let mut encoder = Encoder::new(&function.dfg, prefix, block, params);
    let return_terms: Vec<String> = return_values(&function.dfg, block)
        .to_vec()
        .into_iter()
        .map(|value| encoder.encode_value(value))
        .collect();

    (encoder.declarations, return_terms)
}

/// Checks whether `before` and `after` return the same values for every
/// input, or `None` if `cvc5` isn't installed. Parameters are unified by
/// position (see [`Encoder::new`]) since `before` and `after` are separate
/// `Ssa` graphs with unrelated internal ids.
fn ssa_equivalent(before: &Ssa, after: &Ssa) -> Option<bool> {
    let before_fn = &before.functions[&before.main_id];
    let param_ids = before_fn.dfg[before_fn.entry_block()].parameters();
    let params: Vec<String> = (0..param_ids.len()).map(|i| format!("p{i}")).collect();

    // Plain `(declare-const pN FF)`, plus, for boolean-typed parameters, an
    // idempotence constraint (`pN * pN = pN`) asserting `pN` is `0` or `1` —
    // needed for simplify rules (boolean `Mul`/`Eq`/`And`) that only hold
    // over a value actually constrained to two elements, not a free one.
    let param_decls: Vec<String> = params
        .iter()
        .zip(param_ids)
        .flat_map(|(name, &value_id)| {
            let mut lines = vec![format!("(declare-const {name} FF)")];
            if *before_fn.dfg.type_of_value(value_id) == Type::bool() {
                lines.push(format!("(assert (= (ff.mul {name} {name}) {name}))"));
            }
            lines
        })
        .collect();

    let (before_decls, before_terms) = encode_function(before, "before", &params);
    let (after_decls, after_terms) = encode_function(after, "after", &params);

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
    for line in param_decls.into_iter().chain(before_decls).chain(after_decls) {
        script.push_str(&line);
        script.push('\n');
    }
    script.push_str(&format!("(assert {goal})\n(check-sat)\n"));

    run_cvc5(&script)
}

/// Renders one SSA function's translation to SMT-LIB2 as a single string:
/// parameter and instruction declarations, then its return term(s). Omits
/// the boilerplate (`set-logic`, `define-sort`, `check-sat`) that's identical
/// for every function and not specific to what's being translated.
fn encode_to_smt_text(src: &str) -> String {
    let ssa = Ssa::from_str(src).expect("hand-written SSA text must parse");
    let function = &ssa.functions[&ssa.main_id];
    let num_params = function.dfg[function.entry_block()].parameters().len();
    let params: Vec<String> = (0..num_params).map(|i| format!("p{i}")).collect();

    let mut lines: Vec<String> = params.iter().map(|p| format!("(declare-const {p} FF)")).collect();
    let (declarations, return_terms) = encode_function(&ssa, "t", &params);
    lines.extend(declarations);
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

mod encode_ssa_tests;
mod equivalence_tests;
