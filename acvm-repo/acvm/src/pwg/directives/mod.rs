use acir::{circuit::directives::Directive, native_types::WitnessMap, AcirField};
use num_bigint::BigUint;

use crate::OpcodeResolutionError;

use super::{get_value, insert_value, ErrorLocation};

/// Attempts to solve the [`Directive`] opcode `directive`.
/// If successful, `initial_witness` will be mutated to contain the new witness assignment.
///
/// Returns `Ok(OpcodeResolution)` to signal whether the directive was successful solved.
///
/// Returns `Err(OpcodeResolutionError)` if a circuit constraint is unsatisfied.
pub(crate) fn solve_directives<F: AcirField>(
    initial_witness: &mut WitnessMap<F>,
    directive: &Directive<F>,
) -> Result<(), OpcodeResolutionError<F>> {
    match directive {
        Directive::ToLeRadix { a, b, radix } => {
            let value_a = get_value(a, initial_witness)?;
            let big_integer = BigUint::from_bytes_be(&value_a.to_be_bytes());

            // Decompose the integer into its radix digits in little endian form.
            let decomposed_integer = big_integer.to_radix_le(*radix);

            if b.len() < decomposed_integer.len() {
                return Err(OpcodeResolutionError::UnsatisfiedConstrain {
                    opcode_location: ErrorLocation::Unresolved,
                    payload: None,
                });
            }

            for (i, witness) in b.iter().enumerate() {
                // Fetch the `i'th` digit from the decomposed integer list
                // and convert it to a field element.
                // If it is not available, which can happen when the decomposed integer
                // list is shorter than the witness list, we return 0.
                let value = match decomposed_integer.get(i) {
                    Some(digit) => F::from_be_bytes_reduce(&[*digit]),
                    None => F::zero(),
                };

                insert_value(witness, value, initial_witness)?;
            }

            Ok(())
        }
    }
}
