//! Turning a hint site into concrete candidate outputs.
//!
//! Strategy 1 knows nothing about what a hint computes. It moves one output, optionally lets the
//! constraints solve for a second one, and keeps the rest honest.

use acir::{AcirField, FieldElement, circuit::Circuit, native_types::Witness};
use num_bigint::BigUint;
use std::collections::{BTreeMap, HashSet};

use crate::{
    derive::{Known, derive_from_constraints, linear_coefficients},
    hints::HintSite,
};

/// One override to try: a full set of outputs for a single hint site.
#[derive(Clone, Debug)]
pub struct Candidate {
    pub site_index: usize,
    pub values: Vec<FieldElement>,
    /// How the moved output was chosen, for the report.
    pub strategy: String,
    /// Output that was moved.
    pub moved: usize,
    /// Output the constraints solved for, if any.
    pub derived: Option<usize>,
}

fn to_big(value: &FieldElement) -> BigUint {
    BigUint::from_bytes_be(&value.to_be_bytes())
}

fn from_big(value: &BigUint) -> FieldElement {
    FieldElement::from_be_bytes_reduce(&value.to_bytes_be())
}

/// Values to try for a single output.
///
/// Two families. The near and edge values catch aliases that are a small step away, such as a
/// remainder that may exceed its divisor by one. The wraparound values catch aliases that only
/// exist in the field: a witness multiplied by `c` shifted by `ceil(p / c)` moves the product by
/// just over the modulus, so the constraint still holds while the integer relation it stands for
/// does not.
fn candidate_values(
    circuit: &Circuit<FieldElement>,
    witness: Witness,
    honest: FieldElement,
) -> Vec<(String, FieldElement)> {
    let mut values = vec![
        ("honest+1".to_string(), honest + FieldElement::one()),
        ("honest-1".to_string(), honest - FieldElement::one()),
        ("zero".to_string(), FieldElement::zero()),
        ("one".to_string(), FieldElement::one()),
        ("p-1".to_string(), -FieldElement::one()),
    ];

    let modulus = FieldElement::modulus();
    for coefficient in linear_coefficients(circuit, witness) {
        let positive = to_big(&coefficient);
        let negative = &modulus - &positive;
        let magnitude = positive.min(negative);
        if magnitude <= BigUint::from(1u32) {
            continue;
        }
        // `p / magnitude` is not an integer, and both neighbours are meaningful: shifting a
        // witness by either one moves its product just past the modulus, and which of the two
        // keeps the other witnesses of the constraint in range depends on their honest values.
        let floor = &modulus / &magnitude;
        let ceil = &floor + BigUint::from(1u32);
        for (name, step) in [("floor", floor), ("ceil", ceil)] {
            let step = from_big(&step);
            for multiple in 1..=2u32 {
                let shift = step * FieldElement::from(u128::from(multiple));
                values.push((format!("wraparound_{name}+{multiple}"), honest + shift));
                values.push((format!("wraparound_{name}-{multiple}"), honest - shift));
            }
        }
    }

    values.retain(|(_, value)| *value != honest);
    values
}

/// Every candidate for every site, in the order they will be tried.
pub fn candidates(
    circuit: &Circuit<FieldElement>,
    sites: &[HintSite],
    honest_witness: &Known,
    limit: usize,
) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    let mut seen: HashSet<(usize, Vec<FieldElement>)> = HashSet::new();

    for (site_index, site) in sites.iter().enumerate() {
        for moved in 0..site.outputs.len() {
            let values = candidate_values(circuit, site.outputs[moved], site.honest[moved]);

            for (strategy, value) in values {
                let mut pinned = site.honest.clone();
                pinned[moved] = value;

                let mut push = |values: Vec<FieldElement>, strategy: String, derived| {
                    if candidates.len() < limit && seen.insert((site_index, values.clone())) {
                        candidates.push(Candidate { site_index, values, strategy, moved, derived });
                    }
                };

                // Move one output on its own.
                push(pinned.clone(), strategy.clone(), None);

                // Move one output and let the constraints supply another one.
                for freed in 0..site.outputs.len() {
                    if freed == moved {
                        continue;
                    }
                    let mut known: BTreeMap<_, _> = honest_witness.clone();
                    known.remove(&site.outputs[freed]);
                    known.insert(site.outputs[moved], value);

                    let Some(derived) =
                        derive_from_constraints(circuit, &known, site.outputs[freed])
                    else {
                        continue;
                    };
                    if derived == site.honest[freed] {
                        continue;
                    }

                    let mut values = pinned.clone();
                    values[freed] = derived;
                    push(values, format!("{strategy}+derived"), Some(freed));
                }
            }
        }
    }
    candidates
}
