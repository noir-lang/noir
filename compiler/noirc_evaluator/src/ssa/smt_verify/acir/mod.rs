//! Translates ACIR into SMT-LIB2. Unlike SSA, there's no nested expression
//! tree to walk and memoize — an `AssertZero` opcode is already a flat
//! polynomial equation over witnesses (`Expression::mul_terms`/
//! `linear_combinations`/`q_c`), and every witness is already a flat,
//! directly-addressable index (`Witness(u32)`), so each one just gets its
//! natural name `w{index}`; a comparator that needs to relate a witness to
//! something on another side (e.g. an SSA parameter) does so by asserting an
//! explicit equality between the two names, not by renaming either one. The
//! one thing worth tracking as state is whether the `zero`/`one` constants
//! ended up used at all, since every field in ACIR is the same field and
//! those two values show up constantly (`AssertZero` is the whole point of
//! ACIR, and `q_c`/coefficients are `0`/`1`/`-1` far more often than not).

use std::collections::BTreeSet;

use acvm::AcirField;
use acvm::FieldElement;
use acvm::acir::circuit::brillig::{BrilligInputs, BrilligOutputs};
use acvm::acir::circuit::{Circuit, Opcode};
use acvm::acir::native_types::{Expression, Witness};

use crate::acir::GeneratedAcir;

use super::field_to_decimal;

/// Every witness an `Expression`'s polynomial references (`mul_terms`' pairs
/// and `linear_combinations`' witnesses) — the same extraction every opcode
/// case below needs for whichever of its own fields are `Expression`s.
fn expression_witnesses<F>(expression: &Expression<F>) -> impl Iterator<Item = Witness> {
    expression
        .mul_terms
        .iter()
        .flat_map(|(_, a, b)| [*a, *b])
        .chain(expression.linear_combinations.iter().map(|(_, w)| *w))
}

/// Every witness referenced anywhere in `opcodes`, across every opcode kind.
/// A real compiled circuit references many intermediate witnesses that
/// never appear in any parameter list; relying
/// on a witness being listed as a "parameter" (as every hand-written test
/// text so far happens to do) isn't something real compiler output
/// guarantees. Mirrors the ACIR optimizer's own equivalent
/// (`acvm-repo/acvm/src/compiler/optimizers/common_subexpression/merge_expressions.rs`'s
/// `witness_inputs`) — that one isn't reusable here (it's `pub(crate)` to a
/// different crate, and stateful: it resolves `BrilligInputs::MemoryArray`
/// against a memory-block map only the CSE optimizer, not a standalone
/// scan, has built up), but every other case follows the same logic.
fn referenced_witnesses(opcodes: &[Opcode<FieldElement>]) -> BTreeSet<Witness> {
    let mut witnesses = BTreeSet::new();
    for opcode in opcodes {
        match opcode {
            Opcode::AssertZero(expression) => witnesses.extend(expression_witnesses(expression)),
            Opcode::BlackBoxFuncCall(call) => {
                witnesses.extend(call.get_input_witnesses());
                witnesses.extend(call.get_outputs_vec());
                witnesses.extend(call.get_predicate());
            }
            Opcode::MemoryOp { op, .. } => {
                witnesses.insert(op.index);
                witnesses.insert(op.value);
            }
            Opcode::MemoryInit { init, .. } => witnesses.extend(init.iter().copied()),
            Opcode::BrilligCall { inputs, outputs, predicate, .. } => {
                for input in inputs {
                    match input {
                        BrilligInputs::Single(expression) => {
                            witnesses.extend(expression_witnesses(expression));
                        }
                        BrilligInputs::Array(expressions) => {
                            witnesses.extend(expressions.iter().flat_map(expression_witnesses));
                        }
                        BrilligInputs::MemoryArray(_) => panic!(
                            "smt_verify: BrilligInputs::MemoryArray needs the memory-block \
                             contents tracked by MemoryInit/MemoryOp, which this standalone \
                             witness scan doesn't have"
                        ),
                    }
                }
                for output in outputs {
                    match output {
                        BrilligOutputs::Simple(witness) => {
                            witnesses.insert(*witness);
                        }
                        BrilligOutputs::Array(witnesses_) => witnesses.extend(witnesses_),
                    }
                }
                witnesses.extend(expression_witnesses(predicate));
            }
            Opcode::Call { inputs, outputs, predicate, .. } => {
                witnesses.extend(inputs.iter().copied());
                witnesses.extend(outputs.iter().copied());
                witnesses.extend(expression_witnesses(predicate));
            }
        }
    }
    witnesses
}

/// Tracks whether the `zero`/`one` named constants ended up used while
/// encoding, so callers only define the ones actually needed.
#[derive(Default)]
struct AcirEncoder {
    used_zero: bool,
    used_one: bool,
}

impl AcirEncoder {
    /// A witness's name: always its natural `w{index}`.
    fn witness_name(&self, witness: Witness) -> String {
        witness.to_string()
    }

    /// The symbol standing in for `(as ff0 FF)`, marking it as needed.
    fn zero(&mut self) -> &'static str {
        self.used_zero = true;
        "zero"
    }

    /// The symbol standing in for `(as ff1 FF)`, marking it as needed.
    fn one(&mut self) -> &'static str {
        self.used_one = true;
        "one"
    }

    /// SMT-LIB2 term for a coefficient: the `zero`/`one` symbols for `0`/`1`,
    /// `(ff.neg one)` for `-1`, otherwise `(as ffN FF)` or, if shorter,
    /// `(ff.neg (as ffN FF))`. Unlike SSA source literals (almost always
    /// written non-negative), `AssertZero` coefficients routinely are
    /// genuinely negative (e.g. `w0 - w1` needs a `-1` coefficient on `w1`),
    /// and its canonical non-negative residue is a ~77-digit number close to
    /// the field's modulus — `(ff.neg (as ff1 FF))` says the same thing far
    /// more legibly. Mirrors the same "pick whichever of value/-value is
    /// shorter" choice `FieldElement`'s own `Display` impl already makes.
    fn encode_field_coefficient(&mut self, value: FieldElement) -> String {
        if value.is_zero() {
            return self.zero().to_string();
        }
        if value.is_one() {
            return self.one().to_string();
        }
        if (-value).is_one() {
            return format!("(ff.neg {})", self.one());
        }

        let negated = field_to_decimal(-value);
        let direct = field_to_decimal(value);
        if negated.len() < direct.len() {
            format!("(ff.neg (as ff{negated} FF))")
        } else {
            format!("(as ff{direct} FF)")
        }
    }

    /// `coeff * term`, skipping the multiplication entirely when it's a
    /// no-op: `1 * term` is just `term`, `-1 * term` is `(ff.neg term)`. Both
    /// are unconditional identities, true for every field element regardless
    /// of what `term` represents — unlike detecting and rewriting a whole
    /// expression's shape (e.g. recognizing "this is really `a = b`"), which
    /// this deliberately does not attempt.
    fn encode_scaled_term(&mut self, coeff: FieldElement, term: &str) -> String {
        if coeff.is_one() {
            term.to_string()
        } else if (-coeff).is_one() {
            format!("(ff.neg {term})")
        } else {
            format!("(ff.mul {} {term})", self.encode_field_coefficient(coeff))
        }
    }

    /// Translates one `AssertZero` expression's polynomial into an SMT-LIB2
    /// term: the sum of its multiplication terms, linear terms, and
    /// constant. A zero constant contributes no term at all (`x + 0 = x`,
    /// unconditionally), and a single remaining term skips the `ff.add`
    /// wrapper entirely.
    fn encode_expression(&mut self, expression: &Expression<FieldElement>) -> String {
        let mut terms: Vec<String> = expression
            .mul_terms
            .iter()
            .map(|(coeff, a, b)| {
                let a = self.witness_name(*a);
                let b = self.witness_name(*b);
                self.encode_scaled_term(*coeff, &format!("(ff.mul {a} {b})"))
            })
            .collect();
        terms.extend(expression.linear_combinations.iter().map(|(coeff, w)| {
            let w = self.witness_name(*w);
            self.encode_scaled_term(*coeff, &w)
        }));
        if !expression.q_c.is_zero() {
            terms.push(self.encode_field_coefficient(expression.q_c));
        }

        match terms.as_slice() {
            [] => self.zero().to_string(),
            [only] => only.clone(),
            _ => format!("(ff.add {})", terms.join(" ")),
        }
    }

    /// Translates every `AssertZero` opcode into an `(assert ...)` line,
    /// panicking clearly on any other opcode kind (not yet implemented).
    fn encode_opcodes(&mut self, opcodes: &[Opcode<FieldElement>]) -> Vec<String> {
        opcodes
            .iter()
            .map(|opcode| match opcode {
                Opcode::AssertZero(expression) => {
                    let lhs = self.encode_expression(expression);
                    let zero = self.zero();
                    format!("(assert (= {lhs} {zero}))")
                }
                other => {
                    panic!("smt_verify: encoding for ACIR opcode {other:?} is not implemented")
                }
            })
            .collect()
    }

    /// The `(define-fun zero/one ...)` lines for whichever of those
    /// constants ended up used, in a fixed order.
    fn constant_definitions(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if self.used_zero {
            lines.push("(define-fun zero () FF (as ff0 FF))".to_string());
        }
        if self.used_one {
            lines.push("(define-fun one () FF (as ff1 FF))".to_string());
        }
        lines
    }
}

/// Renders one ACIR circuit's translation to SMT-LIB2 as a single string: a
/// `(define-fun zero/one ...)` for whichever of those constants actually got
/// used, a `(declare-const wN FF)` per witness the opcodes actually
/// reference, then one `(assert (= ... zero))` per `AssertZero` opcode.
/// Omits the boilerplate (`set-logic`, `define-sort`) that's identical for
/// every circuit and not specific to what's being translated.
fn encode_to_smt_text(src: &str) -> String {
    let circuit =
        Circuit::<FieldElement>::from_str(src).expect("hand-written ACIR text must parse");

    let mut encoder = AcirEncoder::default();
    let witnesses = referenced_witnesses(&circuit.opcodes);
    let asserts = encoder.encode_opcodes(&circuit.opcodes);

    let mut lines = encoder.constant_definitions();
    lines.extend(
        witnesses.iter().map(|w| format!("(declare-const {} FF)", encoder.witness_name(*w))),
    );
    lines.extend(asserts);

    lines.join("\n")
}

/// Encodes one ACIR function's `AssertZero` opcodes into SMT-LIB2 lines and
/// its return values' terms, for a comparator to check against another
/// encoding (e.g. the SSA side's), linking corresponding free variables
/// (such as a parameter and the input witness it compiled to) with an
/// explicit equality assertion rather than a shared name.
pub(super) fn encode_generated_acir(
    generated_acir: &GeneratedAcir<FieldElement>,
) -> (Vec<String>, Vec<String>) {
    let mut encoder = AcirEncoder::default();
    let witnesses = referenced_witnesses(&generated_acir.opcodes);
    let asserts = encoder.encode_opcodes(&generated_acir.opcodes);

    let mut lines = encoder.constant_definitions();
    lines.extend(
        witnesses.iter().map(|w| format!("(declare-const {} FF)", encoder.witness_name(*w))),
    );
    lines.extend(asserts);

    let return_terms: Vec<String> =
        generated_acir.return_witnesses.iter().map(|w| encoder.witness_name(*w)).collect();

    (lines, return_terms)
}

mod encode_acir_tests;
