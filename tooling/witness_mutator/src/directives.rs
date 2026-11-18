//! Candidate outputs derived from what a compiler hint computes.
//!
//! The generic strategy moves a witness and lets a constraint answer for its partner, which only
//! reaches an alias when one of the moved values happens to be right. For the three hints the
//! compiler inserts itself, the alias can be computed directly instead: each one stands for an
//! integer relation (`a = q*b + r`, `a = sum(limb_i * radix^i)`) that the constraints can only
//! check modulo `p`. Recomputing the hint on `a + k*p` gives the outputs a dishonest prover would
//! supply — correct in the field, wrong over the integers.

use acir::{
    AcirField, FieldElement,
    circuit::{Opcode, brillig::BrilligInputs},
};
use num_bigint::BigUint;

use crate::{
    derive::{Known, evaluate},
    hints::HintSite,
};

fn to_big(value: &FieldElement) -> BigUint {
    BigUint::from_bytes_be(&value.to_be_bytes())
}

fn from_big(value: &BigUint) -> FieldElement {
    FieldElement::from_be_bytes_reduce(&value.to_bytes_be())
}

/// Values of a call's inputs during honest execution, or `None` if any of them is an array or a
/// memory block rather than a plain expression.
fn input_values(opcode: &Opcode<FieldElement>, known: &Known) -> Option<Vec<FieldElement>> {
    let Opcode::BrilligCall { inputs, .. } = opcode else {
        return None;
    };
    inputs
        .iter()
        .map(|input| match input {
            BrilligInputs::Single(expression) => evaluate(expression, known),
            BrilligInputs::Array(_) | BrilligInputs::MemoryArray(_) => None,
        })
        .collect()
}

/// Outputs to try for a compiler-inserted hint, given the inputs it was called with.
pub fn candidates(
    site: &HintSite,
    opcode: &Opcode<FieldElement>,
    known: &Known,
) -> Vec<(String, Vec<FieldElement>)> {
    let Some(inputs) = input_values(opcode, known) else {
        return Vec::new();
    };
    let modulus = FieldElement::modulus();

    match site.kind.name() {
        // (a, b) -> (q, r) with a = q*b + r. Over the integers that pair is unique; in the field
        // `a + k*p` divides just as well, and the quotient it yields is the classic alias.
        "directive_integer_quotient" if inputs.len() == 2 && site.outputs.len() == 2 => {
            let (a, b) = (to_big(&inputs[0]), to_big(&inputs[1]));
            if b.bits() == 0 {
                return Vec::new();
            }
            (1..=3u32)
                .map(|k| {
                    let lifted = &a + &modulus * BigUint::from(k);
                    let quotient = &lifted / &b;
                    let remainder = &lifted % &b;
                    (
                        format!("quotient_over_modulus+{k}"),
                        vec![from_big(&quotient), from_big(&remainder)],
                    )
                })
                .collect()
        }

        // (a, limb_count, radix) -> limbs, little endian, with a = sum(limb_i * radix^i).
        "directive_to_radix" if inputs.len() == 3 => {
            let radix = to_big(&inputs[2]);
            if radix < BigUint::from(2u32) {
                return Vec::new();
            }
            (1..=2u32)
                .filter_map(|k| {
                    let mut lifted = to_big(&inputs[0]) + &modulus * BigUint::from(k);
                    let mut limbs = Vec::with_capacity(site.outputs.len());
                    for _ in 0..site.outputs.len() {
                        limbs.push(from_big(&(&lifted % &radix)));
                        lifted /= &radix;
                    }
                    // Anything left over does not fit the limbs this call returns.
                    lifted.bits().eq(&0).then_some((format!("radix_over_modulus+{k}"), limbs))
                })
                .collect()
        }

        // The inverse of a non-zero field element is unique, so there is nothing to alias. The
        // hint is only free when its input is zero, where the circuit expects zero back but the
        // constraint `x * inverse == 1` is satisfied vacuously.
        "directive_invert" if inputs.len() == 1 && site.outputs.len() == 1 => {
            if !inputs[0].is_zero() {
                return Vec::new();
            }
            [FieldElement::one(), FieldElement::from(2u128), -FieldElement::one()]
                .into_iter()
                .map(|value| ("inverse_of_zero".to_string(), vec![value]))
                .collect()
        }

        _ => Vec::new(),
    }
}
