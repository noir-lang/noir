//! Machine-checked soundness for the ACIR gadgets that lower integer arithmetic.
//!
//! # The property
//!
//! ACIR is a *relation*, not a function. A gadget is sound when every witness
//! satisfying the constraints it emits agrees with the gadget's specification —
//! not merely the one witness the ACVM happens to construct. Written out, for a
//! gadget with inputs `x` and outputs `y`:
//!
//! ```text
//! ∀ w. satisfies(emitted_opcodes, w) → y(w) = spec(x(w))
//! ```
//!
//! Every existing test in this crate checks the other thing: it runs the ACVM,
//! which solves for `y` from the Brillig hints, and compares the result. That
//! samples exactly one witness — the honest one — so it cannot observe an
//! under-constrained gadget, where the constraints admit a *second* witness a
//! malicious prover could supply instead. Under-constrainedness is a statement
//! about the witnesses the solver would never produce.
//!
//! [`check`] asks an SMT solver for that second witness. `Verdict::Sound` means
//! none exists; `Verdict::Unsound` carries the counterexample.
//!
//! Brillig hint outputs are deliberately left unconstrained: a `BrilligCall`
//! contributes nothing to the encoding, so its outputs are free variables. That
//! is exactly the threat model — a prover chooses them.
//!
//! # Why bitvectors and not a finite field
//!
//! The obvious encoding gives every witness cvc5's finite-field sort. It does
//! not scale: a range constraint has no native form there, so `x < 2^k` must be
//! expanded to `x = Σ 2^i b_i ∧ b_i² = b_i`, and cvc5's Gröbner-basis field
//! solver then has to reconcile two independently-introduced decompositions of
//! the same value. Measured on the `truncate` gadget, that encoding proves
//! 1 bit in 20 ms, 2 bits in 544 ms, and does not finish 3 bits in 30 s.
//!
//! So witnesses are bitvectors of a common width `W` instead, where a range
//! constraint is a `bvult` and costs nothing. The same 3-bit query then takes
//! 11 ms, and 64-bit truncation of a 128-bit value takes 14 ms.
//!
//! # Why that is faithful, and how it is enforced
//!
//! Field arithmetic reduces modulo `p`; bitvector arithmetic reduces modulo
//! `2^W`. They are interchangeable only where neither reduction is observable,
//! so [`Encoding::new`] *proves* that before emitting anything:
//!
//! - it bounds every `AssertZero` expression from the range constraints the
//!   opcodes themselves carry, and picks `W` so that every value stays inside
//!   `(-2^(W-1), 2^(W-1))`;
//! - it requires `W + 1 < log₂ p`, so a difference that underflows is a large
//!   value under *both* moduli and fails a subsequent range check under both.
//!
//! That second condition is not incidental: underflow is the mechanism
//! [`super::AcirContext::bound_constraint_with_offset`] relies on to encode
//! `a < b`, and it is the same condition that function asserts at runtime.
//!
//! # Anything that cannot be encoded faithfully is dropped, never approximated
//!
//! Some opcodes have no faithful bitvector form. The inverse constraint
//! `b · b⁻¹ = 1` that `inv_var` emits is the clearest case: over a field it
//! says `b ≠ 0`, but modulo `2^W` only odd values are invertible, so encoding
//! it directly would quietly restrict `b` and make the query pass for reasons
//! that have nothing to do with the gadget.
//!
//! Such constraints are therefore *dropped*, which is conservative in exactly
//! the direction that matters. Dropping a constraint can only admit more
//! witnesses, so a `Sound` verdict over the smaller set holds a fortiori over
//! the real one. The cost is the other direction: a counterexample found
//! against a weakened system may be spurious, so [`Verdict::Unsound`] reports
//! how many constraints were dropped. The one thing never done is encoding a
//! constraint as something stronger than it is, because that direction turns an
//! unsound gadget into a passing test.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;
use std::process::{Command, Stdio};

use acvm::AcirField;
use acvm::FieldElement;
use acvm::acir::circuit::Opcode;
use acvm::acir::circuit::opcodes::{BlackBoxFuncCall, FunctionInput};
use acvm::acir::native_types::{Expression, Witness};
use num_bigint::BigUint;

/// The query needs a width at which bitvector and field arithmetic can visibly
/// disagree, so no faithful encoding exists at all.
/// `bound_constraint_with_offset` asserts the same condition at runtime.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct WidthExceedsField {
    pub(super) needed: u32,
    pub(super) available: u32,
}

/// What the solver had to say about a gadget.
#[derive(Debug)]
pub(super) enum Verdict {
    /// No witness satisfies the constraints and violates the specification.
    Sound,
    /// One does; the string is cvc5's model for it. `dropped` counts the
    /// constraints that had no faithful encoding and were left out, so a
    /// nonzero count means the counterexample needs checking against the real
    /// opcode list before it is believed.
    Unsound { model: String, dropped: usize },
    /// cvc5 is not installed, so nothing was checked.
    Skipped,
}

impl Verdict {
    pub(super) fn assert_sound(self) {
        match self {
            Verdict::Sound | Verdict::Skipped => {}
            Verdict::Unsound { model, dropped } => panic!(
                "gadget is not sound; counterexample ({dropped} constraint(s) dropped as \
                 unencodable):\n{model}"
            ),
        }
    }

    /// Whether a counterexample was found, or `None` when nothing was checked.
    pub(super) fn found_counterexample(&self) -> Option<bool> {
        match self {
            Verdict::Sound => Some(false),
            Verdict::Unsound { .. } => Some(true),
            Verdict::Skipped => None,
        }
    }

    pub(super) fn assert_unsound(self) {
        match self {
            Verdict::Unsound { .. } | Verdict::Skipped => {}
            Verdict::Sound => panic!(
                "expected a counterexample but the query was proved sound — the encoding is \
                 stronger than the opcodes it claims to model, so a passing soundness result \
                 from it would mean nothing"
            ),
        }
    }
}

/// An SMT-LIB2 rendering of an opcode list, over bitvectors of width [`Self::width`].
pub(super) struct Encoding {
    width: u32,
    lines: Vec<String>,
    witnesses: BTreeSet<Witness>,
    dropped: usize,
}

impl Encoding {
    pub(super) fn new(opcodes: &[Opcode<FieldElement>]) -> Result<Self, WidthExceedsField> {
        let ranges = collect_ranges(opcodes);
        let analysis = analyze(opcodes, &ranges);
        let bounds = &analysis.bounds;
        let width = required_width(opcodes, &analysis)?;

        let mut witnesses = BTreeSet::new();
        let mut lines = Vec::new();
        let mut dropped = 0;
        let zero = literal(&BigUint::ZERO, width);
        let one = literal(&BigUint::from(1_u32), width);

        for (index, opcode) in opcodes.iter().enumerate() {
            match analysis.abstractions.get(&index) {
                Some(Abstraction::Absorbed) => continue,
                Some(Abstraction::NonZero(subject)) => {
                    collect_expression_witnesses(subject, &mut witnesses);
                    let subject = encode_expression(subject, width);
                    lines.push(format!("(assert (not (= {subject} {zero})))"));
                    continue;
                }
                Some(Abstraction::IsZero { indicator, subject }) => {
                    witnesses.insert(*indicator);
                    collect_expression_witnesses(subject, &mut witnesses);
                    let subject = encode_expression(subject, width);
                    lines.push(format!(
                        "(assert (= {} (ite (= {subject} {zero}) {one} {zero})))",
                        name(*indicator)
                    ));
                    continue;
                }
                None => {}
            }
            match opcode {
                Opcode::AssertZero(expression) => {
                    // Only an expression whose every witness is bounded has a
                    // faithful bitvector form; the rest are dropped, which can
                    // only admit more witnesses.
                    if expression_bound(expression, bounds).is_err() {
                        dropped += 1;
                        continue;
                    }
                    collect_witnesses(opcode, &mut witnesses);
                    let terms = encode_expression(expression, width);
                    lines.push(format!("(assert (= {terms} {zero}))"));
                }
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                    input: FunctionInput::Witness(witness),
                    num_bits,
                }) => {
                    witnesses.insert(*witness);
                    let limit = BigUint::from(1_u32) << num_bits;
                    lines.push(format!(
                        "(assert (bvult {} {}))",
                        name(*witness),
                        literal(&limit, width)
                    ));
                }
                // A hint constrains nothing: its outputs stay free variables.
                Opcode::BrilligCall { .. } => {}
                _ => dropped += 1,
            }
        }

        let declarations = witnesses
            .iter()
            .map(|witness| format!("(declare-const {} (_ BitVec {width}))", name(*witness)))
            .collect::<Vec<_>>();
        lines.splice(0..0, declarations);

        Ok(Encoding { width, lines, witnesses, dropped })
    }

    /// How many constraints had no faithful encoding and were left out.
    pub(super) fn dropped(&self) -> usize {
        self.dropped
    }

    pub(super) fn width(&self) -> u32 {
        self.width
    }

    /// Makes sure a witness the specification talks about is declared, even if
    /// every constraint mentioning it was dropped.
    pub(super) fn declare(&mut self, witness: Witness) {
        if self.witnesses.insert(witness) {
            self.lines
                .insert(0, format!("(declare-const {} (_ BitVec {}))", name(witness), self.width));
        }
    }

    /// Asks whether any witness satisfies the constraints while `goal` — the
    /// negation of the gadget's specification — also holds.
    pub(super) fn check(&self, goal: &str) -> Verdict {
        // A constraint set with no solutions at all proves every goal, so a
        // `Sound` verdict over one would be vacuous. That is not a theoretical
        // worry: an earlier version of this encoding picked a width too narrow
        // to hold its own range limits, every `bvult` became false, and the
        // whole query silently passed.
        if let Verdict::Sound = self.solve("true") {
            panic!(
                "the encoded constraints are unsatisfiable, so any soundness result from them \
                 would be vacuous"
            );
        }
        self.solve(goal)
    }

    fn solve(&self, goal: &str) -> Verdict {
        let mut script = String::from("(set-logic QF_BV)\n(set-option :produce-models true)\n");
        for line in &self.lines {
            script.push_str(line);
            script.push('\n');
        }
        script.push_str(&format!("(assert {goal})\n(check-sat)\n"));
        for witness in &self.witnesses {
            script.push_str(&format!("(get-value ({}))\n", name(*witness)));
        }
        run_cvc5(&script, self.dropped)
    }
}

/// The bit width each witness is range-constrained to. A witness constrained
/// more than once keeps the tightest bound.
fn collect_ranges(opcodes: &[Opcode<FieldElement>]) -> BTreeMap<Witness, u32> {
    let mut ranges: BTreeMap<Witness, u32> = BTreeMap::new();
    for opcode in opcodes {
        if let Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
            input: FunctionInput::Witness(witness),
            num_bits,
        }) = opcode
        {
            ranges
                .entry(*witness)
                .and_modify(|bits| *bits = (*bits).min(*num_bits))
                .or_insert(*num_bits);
        }
    }
    ranges
}

/// One of the two field-inverse idioms, rewritten into a form a bitvector
/// solver can read. Both rewrites are equivalences, proved separately in
/// `QF_FF`, so neither loses a witness nor invents one.
#[derive(Clone)]
enum Abstraction {
    /// `E * inv = 1` with `inv` free and unused elsewhere, hence `E != 0`.
    NonZero(Expression<FieldElement>),
    /// `z = 1 - E * inv` together with `E * z = 0`, hence `z = ite(E = 0, 1, 0)`.
    IsZero { indicator: Witness, subject: Expression<FieldElement> },
    /// The partner opcode of an [`Abstraction::IsZero`], already accounted for.
    Absorbed,
}

/// Splits `expression` into `(coefficient, remainder)` with
/// `expression = witness * coefficient + remainder`, both linear. `None` when
/// `witness` appears squared, which no idiom here produces.
fn split_on(
    expression: &Expression<FieldElement>,
    witness: Witness,
) -> Option<(Expression<FieldElement>, Expression<FieldElement>)> {
    let mut coefficient = Expression::<FieldElement>::default();
    let mut remainder = Expression::<FieldElement>::default();

    for (factor, lhs, rhs) in &expression.mul_terms {
        let other = match (*lhs == witness, *rhs == witness) {
            (true, true) => return None,
            (true, false) => *rhs,
            (false, true) => *lhs,
            (false, false) => {
                remainder.mul_terms.push((*factor, *lhs, *rhs));
                continue;
            }
        };
        coefficient.linear_combinations.push((*factor, other));
    }
    for (factor, other) in &expression.linear_combinations {
        if *other == witness {
            coefficient.q_c += *factor;
        } else {
            remainder.linear_combinations.push((*factor, *other));
        }
    }
    remainder.q_c = expression.q_c;
    Some((coefficient, remainder))
}

fn is_constant(expression: &Expression<FieldElement>, value: FieldElement) -> bool {
    expression.mul_terms.is_empty()
        && expression.linear_combinations.is_empty()
        && expression.q_c == value
}

/// An `AssertZero` says its expression is zero, so it carries no preferred
/// sign: these shapes are matched up to negation throughout.
fn is_unit_constant(expression: &Expression<FieldElement>) -> bool {
    is_constant(expression, FieldElement::one()) || is_constant(expression, -FieldElement::one())
}

/// `±(1 - z)`, for the `z` the caller is looking for.
fn one_minus_witness(expression: &Expression<FieldElement>) -> Option<Witness> {
    if !expression.mul_terms.is_empty() {
        return None;
    }
    let [(factor, witness)] = expression.linear_combinations[..] else { return None };
    (expression.q_c == -factor && is_unit_constant(&Expression::from_field(factor)))
        .then_some(witness)
}

/// Whether two linear polynomials are equal up to sign, which is all `E * z = 0`
/// needs in order to be the partner of an `IsZero`.
fn same_up_to_sign(left: &Expression<FieldElement>, right: &Expression<FieldElement>) -> bool {
    let negated = |e: &Expression<FieldElement>| {
        let mut e = e.clone();
        e.linear_combinations.iter_mut().for_each(|(factor, _)| *factor = -*factor);
        e.q_c = -e.q_c;
        e.sort();
        e
    };
    let mut left_sorted = left.clone();
    left_sorted.sort();
    let mut right_sorted = right.clone();
    right_sorted.sort();
    left_sorted == right_sorted || left_sorted == negated(right)
}

/// How many opcodes a witness appears in, so that a hint used in exactly one
/// place can be told apart from one that is load-bearing elsewhere.
fn appearances(opcodes: &[Opcode<FieldElement>]) -> BTreeMap<Witness, usize> {
    let mut counts: BTreeMap<Witness, usize> = BTreeMap::new();
    for opcode in opcodes {
        let mut here = BTreeSet::new();
        collect_witnesses(opcode, &mut here);
        for witness in here {
            *counts.entry(witness).or_default() += 1;
        }
    }
    counts
}

/// Bounds and inverse-idiom rewrites, computed together: a rewrite bounds its
/// indicator witness, which can bound more expressions, which can expose
/// another rewrite.
struct Analysis {
    bounds: BTreeMap<Witness, BigUint>,
    abstractions: BTreeMap<usize, Abstraction>,
}

fn analyze(opcodes: &[Opcode<FieldElement>], ranges: &BTreeMap<Witness, u32>) -> Analysis {
    let appearances = appearances(opcodes);
    let mut analysis = Analysis {
        bounds: ranges
            .iter()
            .map(|(witness, bits)| (*witness, (BigUint::from(1_u32) << bits) - 1_u32))
            .collect(),
        abstractions: BTreeMap::new(),
    };

    loop {
        let grew = propagate_bounds(opcodes, &mut analysis);
        let rewritten = find_inverse_idioms(opcodes, &appearances, &mut analysis);
        if !grew && !rewritten {
            return analysis;
        }
    }
}

/// Recognises the two inverse idioms and records their proved conclusions.
fn find_inverse_idioms(
    opcodes: &[Opcode<FieldElement>],
    appearances: &BTreeMap<Witness, usize>,
    analysis: &mut Analysis,
) -> bool {
    let mut progress = false;
    for (index, opcode) in opcodes.iter().enumerate() {
        if analysis.abstractions.contains_key(&index) {
            continue;
        }
        let Opcode::AssertZero(expression) = opcode else { continue };

        // The hint is the unbounded witness inside a product — in both idioms
        // the indicator is unbounded too until the rewrite bounds it, so the
        // product is what tells them apart. It must be the only such witness,
        // and must appear in no other opcode, or removing it would lose a
        // constraint that is doing work elsewhere.
        let mut in_products = BTreeSet::new();
        for (_, lhs, rhs) in &expression.mul_terms {
            in_products.insert(*lhs);
            in_products.insert(*rhs);
        }
        in_products.retain(|witness| !analysis.bounds.contains_key(witness));
        let [inverse] = in_products.iter().copied().collect::<Vec<_>>()[..] else { continue };
        if appearances.get(&inverse) != Some(&1) {
            continue;
        }
        let Some((subject, remainder)) = split_on(expression, inverse) else { continue };
        if subject.linear_combinations.is_empty() && subject.q_c.is_zero() {
            continue;
        }

        if is_unit_constant(&remainder) {
            analysis.abstractions.insert(index, Abstraction::NonZero(subject));
            progress = true;
        } else if let Some(indicator) = one_minus_witness(&remainder) {
            let Some(partner) = find_partner(opcodes, index, indicator, &subject) else { continue };
            analysis.abstractions.insert(index, Abstraction::IsZero { indicator, subject });
            analysis.abstractions.insert(partner, Abstraction::Absorbed);
            analysis.bounds.insert(indicator, BigUint::from(1_u32));
            progress = true;
        }
    }
    progress
}

/// The `E * z = 0` half of an `IsZero`.
fn find_partner(
    opcodes: &[Opcode<FieldElement>],
    skip: usize,
    indicator: Witness,
    subject: &Expression<FieldElement>,
) -> Option<usize> {
    opcodes.iter().enumerate().position(|(index, opcode)| {
        if index == skip {
            return false;
        }
        let Opcode::AssertZero(expression) = opcode else { return false };
        let Some((coefficient, remainder)) = split_on(expression, indicator) else { return false };
        is_constant(&remainder, FieldElement::zero()) && same_up_to_sign(&coefficient, subject)
    })
}

/// The largest absolute value each witness can hold. Range constraints seed
/// this, and it then propagates through defining equations: when an
/// `AssertZero` pins one otherwise-unknown witness to a combination of known
/// ones, that witness inherits their bound. Without the propagation step the
/// return witness of a compiled circuit is unbounded — it is only ever tied to
/// a range-constrained value by an equation — so the equation defining it would
/// be dropped and the query would admit witnesses the real circuit rejects.
fn propagate_bounds(opcodes: &[Opcode<FieldElement>], analysis: &mut Analysis) -> bool {
    let bounds = &mut analysis.bounds;
    let mut grew = false;
    loop {
        let mut progress = false;
        for (index, opcode) in opcodes.iter().enumerate() {
            if analysis.abstractions.contains_key(&index) {
                continue;
            }
            let Opcode::AssertZero(expression) = opcode else { continue };
            let Some(unknown) = sole_unknown(expression, bounds) else { continue };

            // `unknown` is `-(everything else)`, so it cannot exceed the sum of
            // the magnitudes of the rest.
            let mut rest = magnitude(&expression.q_c);
            let mut derivable = true;
            for (coefficient, lhs, rhs) in &expression.mul_terms {
                match (bounds.get(lhs), bounds.get(rhs)) {
                    (Some(lhs), Some(rhs)) => rest += magnitude(coefficient) * lhs * rhs,
                    _ => derivable = false,
                }
            }
            for (coefficient, witness) in &expression.linear_combinations {
                if *witness == unknown {
                    continue;
                }
                match bounds.get(witness) {
                    Some(bound) => rest += magnitude(coefficient) * bound,
                    None => derivable = false,
                }
            }
            if derivable && bounds.insert(unknown, rest).is_none() {
                progress = true;
                grew = true;
            }
        }
        if !progress {
            return grew;
        }
    }
}

/// The one unbounded witness an expression pins down, if there is exactly one
/// and it appears as a lone `±1`-weighted linear term. Any other shape leaves
/// its value underdetermined by this equation.
fn sole_unknown(
    expression: &Expression<FieldElement>,
    bounds: &BTreeMap<Witness, BigUint>,
) -> Option<Witness> {
    let mut candidate = None;
    for (_, witness) in &expression.linear_combinations {
        if bounds.contains_key(witness) {
            continue;
        }
        if candidate.is_some_and(|existing| existing != *witness) {
            return None;
        }
        candidate = Some(*witness);
    }
    let candidate = candidate?;

    let appearances = expression
        .linear_combinations
        .iter()
        .filter(|(_, witness)| *witness == candidate)
        .collect::<Vec<_>>();
    let [(coefficient, _)] = appearances[..] else { return None };
    if magnitude(coefficient) != BigUint::from(1_u32) {
        return None;
    }
    // A witness inside a product is not pinned down by this equation: the
    // factor it multiplies may be zero.
    let in_a_product =
        expression.mul_terms.iter().any(|(_, lhs, rhs)| *lhs == candidate || *rhs == candidate);
    (!in_a_product).then_some(candidate)
}

/// A width at which no `AssertZero` expression can reach the modulus, so that
/// reducing modulo `2^W` and reducing modulo `p` agree on every value the
/// opcodes can produce.
fn required_width(
    opcodes: &[Opcode<FieldElement>],
    analysis: &Analysis,
) -> Result<u32, WidthExceedsField> {
    let bounds = &analysis.bounds;
    let mut largest = BigUint::from(1_u32);
    for abstraction in analysis.abstractions.values() {
        let subject = match abstraction {
            Abstraction::NonZero(subject) => subject,
            Abstraction::IsZero { subject, .. } => subject,
            Abstraction::Absorbed => continue,
        };
        if let Ok(bound) = expression_bound(subject, bounds) {
            largest = largest.max(bound);
        }
    }
    for (index, opcode) in opcodes.iter().enumerate() {
        if analysis.abstractions.contains_key(&index) {
            continue;
        }
        match opcode {
            Opcode::AssertZero(expression) => {
                // An expression that cannot be bounded gets dropped rather than
                // encoded, so it does not constrain the width either.
                if let Ok(bound) = expression_bound(expression, bounds) {
                    largest = largest.max(bound);
                }
            }
            // The limit `2^num_bits` is itself a literal in the query, so the
            // width has to hold it. Without this a circuit whose expressions
            // were all dropped would get a width too narrow for its own range
            // limits, they would wrap to zero, and every `bvult` against them
            // would be false -- an unsatisfiable system, which reads as "sound"
            // while having checked nothing.
            Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE { num_bits, .. }) => {
                largest = largest.max(BigUint::from(1_u32) << num_bits);
            }
            _ => {}
        }
    }
    // One bit for the sign, one so that an underflowed value cannot be mistaken
    // for a small one.
    let needed = largest.bits() as u32 + 2;
    let available = FieldElement::max_num_bits();
    if needed + 1 >= available {
        return Err(WidthExceedsField { needed, available });
    }
    Ok(needed)
}

/// The largest absolute value `expression` can take, given the range
/// constraints, treating each coefficient as its signed magnitude.
fn expression_bound(
    expression: &Expression<FieldElement>,
    bounds: &BTreeMap<Witness, BigUint>,
) -> Result<BigUint, Unbounded> {
    let bound = |witness: &Witness| -> Result<BigUint, Unbounded> {
        bounds.get(witness).cloned().ok_or(Unbounded)
    };

    let mut total = magnitude(&expression.q_c);
    for (coefficient, lhs, rhs) in &expression.mul_terms {
        total += magnitude(coefficient) * bound(lhs)? * bound(rhs)?;
    }
    for (coefficient, witness) in &expression.linear_combinations {
        total += magnitude(coefficient) * bound(witness)?;
    }
    Ok(total)
}

/// A witness with no range constraint has no bound on its value, so no
/// justification for reading it as a bitvector.
struct Unbounded;

/// `|x|` for a field element read as a signed representative: coefficients in
/// ACIR are routinely `p - 1` meaning `-1`, and bounding those as ~`p` would
/// make every expression unencodable.
fn magnitude(value: &FieldElement) -> BigUint {
    let value = to_biguint(value);
    let negated = FieldElement::modulus() - &value;
    value.min(negated)
}

fn to_biguint(value: &FieldElement) -> BigUint {
    BigUint::from_bytes_be(&value.to_be_bytes())
}

/// Renders an expression as a bitvector term. Signed coefficients are emitted
/// as their two's-complement representative at this width, which is the same
/// value the field holds for them once reduced.
fn encode_expression(expression: &Expression<FieldElement>, width: u32) -> String {
    let mut terms = Vec::new();
    for (coefficient, lhs, rhs) in &expression.mul_terms {
        terms.push(format!(
            "(bvmul {} {} {})",
            coefficient_literal(coefficient, width),
            name(*lhs),
            name(*rhs)
        ));
    }
    for (coefficient, witness) in &expression.linear_combinations {
        terms.push(format!(
            "(bvmul {} {})",
            coefficient_literal(coefficient, width),
            name(*witness)
        ));
    }
    if !expression.q_c.is_zero() {
        terms.push(coefficient_literal(&expression.q_c, width));
    }
    match terms.len() {
        0 => literal(&BigUint::ZERO, width),
        1 => terms.pop().expect("just checked there is one"),
        _ => format!("(bvadd {})", terms.join(" ")),
    }
}

fn coefficient_literal(value: &FieldElement, width: u32) -> String {
    let value = to_biguint(value);
    let modulus = FieldElement::modulus();
    let negated = &modulus - &value;
    if negated < value {
        // A negative coefficient: its two's-complement representative.
        literal(&((BigUint::from(1_u32) << width) - negated), width)
    } else {
        literal(&value, width)
    }
}

/// Renders a bitvector constant, refusing to truncate: a literal that does not
/// fit is a bug in the width computation, and silently wrapping it would change
/// what the query asks.
fn literal(value: &BigUint, width: u32) -> String {
    assert!(
        *value < (BigUint::from(1_u32) << width),
        "constant {value} does not fit in {width} bits"
    );
    format!("(_ bv{value} {width})")
}

fn name(witness: Witness) -> String {
    format!("w{}", witness.0)
}

fn collect_expression_witnesses(
    expression: &Expression<FieldElement>,
    into: &mut BTreeSet<Witness>,
) {
    for (_, lhs, rhs) in &expression.mul_terms {
        into.insert(*lhs);
        into.insert(*rhs);
    }
    for (_, witness) in &expression.linear_combinations {
        into.insert(*witness);
    }
}

fn collect_witnesses(opcode: &Opcode<FieldElement>, into: &mut BTreeSet<Witness>) {
    match opcode {
        Opcode::AssertZero(expression) => collect_expression_witnesses(expression, into),
        Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
            input: FunctionInput::Witness(witness),
            ..
        }) => {
            into.insert(*witness);
        }
        _ => {}
    }
}

/// Whether tests are running in CI, by this repo\'s own convention (the
/// `justfile`\'s `ci :=` line checks the same variable the same way).
fn is_ci() -> bool {
    matches!(std::env::var("CI").as_deref(), Ok("true") | Ok("1"))
}

/// Runs a complete SMT-LIB2 script. Used for the field-algebra lemmas the
/// inverse-idiom rewrites rest on, which are written out in full rather than
/// built by [`Encoding`]: they are statements about `ZZ_p`, not about any
/// particular circuit.
pub(super) fn prove(script: &str) -> Verdict {
    run_cvc5(script, 0)
}

/// The prime the circuits live over, for those lemmas to quantify over.
pub(super) fn modulus() -> BigUint {
    FieldElement::modulus()
}

/// Shells out rather than linking a solver into the compiler: cvc5 is a test
/// dependency of one module, not of the build.
fn run_cvc5(script: &str, dropped: usize) -> Verdict {
    let budget = std::env::var("NOIR_SOUNDNESS_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(600_000);
    let Ok(mut child) = Command::new("cvc5")
        .args(["--lang=smt2", &format!("--tlimit={budget}"), "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        // Locally a missing solver is a skip, so `cargo test` still works
        // without it. On CI it is a failure: a misconfigured install step would
        // otherwise leave every one of these tests green having checked nothing.
        assert!(
            !is_ci(),
            "cvc5 is not on PATH in CI. The `Install cvc5` step (`just install-cvc5`) should have \
             put it there — see .github/workflows/test-rust-workspace.yml"
        );
        eprintln!("skipping soundness check: cvc5 not found on PATH");
        return Verdict::Skipped;
    };

    child
        .stdin
        .take()
        .expect("spawned with a piped stdin")
        .write_all(script.as_bytes())
        .expect("failed to write the query to cvc5");

    let output = child.wait_with_output().expect("failed to read cvc5's output");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();
    match lines.next().unwrap_or("").trim() {
        "unsat" => Verdict::Sound,
        "sat" => Verdict::Unsound { model: lines.collect::<Vec<_>>().join("\n"), dropped },
        other => panic!(
            "cvc5 did not decide the query (output {other:?}); raise NOIR_SOUNDNESS_TIMEOUT_MS \
             if it ran out of budget\n{stdout}"
        ),
    }
}
