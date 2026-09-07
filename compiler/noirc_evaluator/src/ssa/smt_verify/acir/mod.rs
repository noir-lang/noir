//! Translates ACIR into SMT-LIB2. Unlike SSA, there's no nested expression
//! tree to walk and memoize — an `AssertZero` opcode is already a flat
//! polynomial equation over witnesses (`Expression::mul_terms`/
//! `linear_combinations`/`q_c`), and every witness is already a flat,
//! directly-addressable index (`Witness(u32)`), so each one just gets the
//! name `w{index}` with no value-to-name memo table needed. The one thing
//! still worth tracking as state is whether the `zero`/`one` constants ended
//! up used at all, since every field in ACIR is the same field and those two
//! values show up constantly (`AssertZero` is the whole point of ACIR, and
//! `q_c`/coefficients are `0`/`1`/`-1` far more often than not).

use std::collections::BTreeSet;

use acvm::AcirField;
use acvm::FieldElement;
use acvm::acir::circuit::{Circuit, Opcode};
use acvm::acir::native_types::{Expression, Witness};

use super::field_to_decimal;

/// Tracks whether the `zero`/`one` named constants ended up used while
/// encoding, so `encode_to_smt_text` only defines the ones actually needed.
#[derive(Default)]
struct AcirEncoder {
    used_zero: bool,
    used_one: bool,
}

impl AcirEncoder {
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
                self.encode_scaled_term(*coeff, &format!("(ff.mul w{} w{})", a.0, b.0))
            })
            .collect();
        terms.extend(
            expression
                .linear_combinations
                .iter()
                .map(|(coeff, w)| self.encode_scaled_term(*coeff, &format!("w{}", w.0))),
        );
        if !expression.q_c.is_zero() {
            terms.push(self.encode_field_coefficient(expression.q_c));
        }

        match terms.as_slice() {
            [] => self.zero().to_string(),
            [only] => only.clone(),
            _ => format!("(ff.add {})", terms.join(" ")),
        }
    }
}

/// Renders one ACIR circuit's translation to SMT-LIB2 as a single string: a
/// `(define-fun zero/one ...)` for whichever of those constants actually got
/// used, a `(declare-const wN FF)` per private/public parameter witness,
/// then one `(assert (= ... zero))` per `AssertZero` opcode. Omits the
/// boilerplate (`set-logic`, `define-sort`) that's identical for every
/// circuit and not specific to what's being translated.
fn encode_to_smt_text(src: &str) -> String {
    let circuit =
        Circuit::<FieldElement>::from_str(src).expect("hand-written ACIR text must parse");

    let mut witnesses: BTreeSet<Witness> = circuit.private_parameters.clone();
    witnesses.extend(circuit.public_parameters.0.iter().copied());

    let mut encoder = AcirEncoder::default();
    let asserts: Vec<String> = circuit
        .opcodes
        .iter()
        .map(|opcode| match opcode {
            Opcode::AssertZero(expression) => {
                let lhs = encoder.encode_expression(expression);
                let zero = encoder.zero();
                format!("(assert (= {lhs} {zero}))")
            }
            other => panic!("smt_verify: encoding for ACIR opcode {other:?} is not implemented"),
        })
        .collect();

    let mut lines: Vec<String> = Vec::new();
    if encoder.used_zero {
        lines.push("(define-fun zero () FF (as ff0 FF))".to_string());
    }
    if encoder.used_one {
        lines.push("(define-fun one () FF (as ff1 FF))".to_string());
    }
    lines.extend(witnesses.iter().map(|w| format!("(declare-const w{} FF)", w.0)));
    lines.extend(asserts);

    lines.join("\n")
}

mod encode_acir_tests;
