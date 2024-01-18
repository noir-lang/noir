use std::cmp::Ordering;

use acir::{circuit::directives::Directive, native_types::WitnessMap, FieldElement};
use num_bigint::BigUint;

use crate::OpcodeResolutionError;

use super::{get_value, insert_value, ErrorLocation};

mod sorting;

/// Attempts to solve the [`Directive`] opcode `directive`.
/// If successful, `initial_witness` will be mutated to contain the new witness assignment.
///
/// Returns `Ok(OpcodeResolution)` to signal whether the directive was successful solved.
///
/// Returns `Err(OpcodeResolutionError)` if a circuit constraint is unsatisfied.
pub(super) fn solve_directives(
    initial_witness: &mut WitnessMap,
    directive: &Directive,
) -> Result<(), OpcodeResolutionError> {
    match directive {
        Directive::ToLeRadix { a, b, radix } => {
            let value_a = get_value(a, initial_witness)?;
            let big_integer = BigUint::from_bytes_be(&value_a.to_be_bytes());

            // Decompose the integer into its radix digits in little endian form.
            let decomposed_integer = big_integer.to_radix_le(*radix);

            if b.len() < decomposed_integer.len() {
                return Err(OpcodeResolutionError::UnsatisfiedConstrain {
                    opcode_location: ErrorLocation::Unresolved,
                });
            }

            for (i, witness) in b.iter().enumerate() {
                // Fetch the `i'th` digit from the decomposed integer list
                // and convert it to a field element.
                // If it is not available, which can happen when the decomposed integer
                // list is shorter than the witness list, we return 0.
                let value = match decomposed_integer.get(i) {
                    Some(digit) => FieldElement::from_be_bytes_reduce(&[*digit]),
                    None => FieldElement::zero(),
                };

                insert_value(witness, value, initial_witness)?;
            }

            Ok(())
        }
        Directive::PermutationSort { inputs: a, tuple, bits, sort_by } => {
            let mut val_a = Vec::new();
            let mut base = Vec::new();
            for (i, element) in a.iter().enumerate() {
                assert_eq!(element.len(), *tuple as usize);
                let mut element_val = Vec::with_capacity(*tuple as usize + 1);
                for e in element {
                    element_val.push(get_value(e, initial_witness)?);
                }
                let field_i = FieldElement::from(i as i128);
                element_val.push(field_i);
                base.push(field_i);
                val_a.push(element_val);
            }
            val_a.sort_by(|a, b| {
                for i in sort_by {
                    let int_a = BigUint::from_bytes_be(&a[*i as usize].to_be_bytes());
                    let int_b = BigUint::from_bytes_be(&b[*i as usize].to_be_bytes());
                    let cmp = int_a.cmp(&int_b);
                    if cmp != Ordering::Equal {
                        return cmp;
                    }
                }
                Ordering::Equal
            });
            let b = val_a.iter().map(|a| *a.last().unwrap()).collect();
            let control = sorting::route(base, b);
            for (w, value) in bits.iter().zip(control) {
                let value = if value { FieldElement::one() } else { FieldElement::zero() };
                insert_value(w, value, initial_witness)?;
            }
            Ok(())
        }
    }
}
