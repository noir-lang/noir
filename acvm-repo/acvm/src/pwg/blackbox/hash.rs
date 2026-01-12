use acir::{
    AcirField,
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
};
use acvm_blackbox_solver::{BlackBoxFunctionSolver, BlackBoxResolutionError, sha256_compression};

use crate::OpcodeResolutionError;
use crate::pwg::{input_to_value, insert_value};

/// Attempts to solve a 256 bit hash function opcode.
/// If successful, `initial_witness` will be mutated to contain the new witness assignment.
pub(super) fn solve_generic_256_hash_opcode<F: AcirField>(
    initial_witness: &mut WitnessMap<F>,
    inputs: &[FunctionInput<F>],
    var_message_size: Option<&FunctionInput<F>>,
    outputs: &[Witness; 32],
    hash_function: fn(data: &[u8]) -> Result<[u8; 32], BlackBoxResolutionError>,
) -> Result<(), OpcodeResolutionError<F>> {
    let message_input = get_hash_input(initial_witness, inputs, var_message_size, 8)?;
    let digest: [u8; 32] = hash_function(&message_input)?;

    write_digest_to_outputs(initial_witness, outputs, digest)
}

/// Reads the hash function input from a [`WitnessMap`].
pub(crate) fn get_hash_input<F: AcirField>(
    initial_witness: &WitnessMap<F>,
    inputs: &[FunctionInput<F>],
    message_size: Option<&FunctionInput<F>>,
    num_bits: usize,
) -> Result<Vec<u8>, OpcodeResolutionError<F>> {
    // Read witness assignments.
    let mut message_input = Vec::new();
    for input in inputs.iter() {
        let witness_assignment = input_to_value(initial_witness, *input)?;
        let bytes = witness_assignment.fetch_nearest_bytes(num_bits);
        message_input.extend(bytes);
    }

    // Truncate the message if there is a `message_size` parameter given
    match message_size {
        Some(input) => {
            let num_bytes_to_take = input_to_value(initial_witness, *input)?
                .try_into_u128()
                .map(|num_bytes_to_take| num_bytes_to_take as usize)
                .expect("expected a 'num_bytes_to_take' that fit into a u128");

            // If the number of bytes to take is more than the amount of bytes available
            // in the message, then we error.
            if num_bytes_to_take > message_input.len() {
                return Err(OpcodeResolutionError::BlackBoxFunctionFailed(
                    acir::BlackBoxFunc::Blake2s,
                    format!(
                        "the number of bytes to take from the message is more than the number of bytes in the message. {} > {}",
                        num_bytes_to_take,
                        message_input.len()
                    ),
                ));
            }
            let truncated_message = message_input[0..num_bytes_to_take].to_vec();
            Ok(truncated_message)
        }
        None => Ok(message_input),
    }
}

/// Writes a `digest` to the [`WitnessMap`] at witness indices `outputs`.
fn write_digest_to_outputs<F: AcirField>(
    initial_witness: &mut WitnessMap<F>,
    outputs: &[Witness; 32],
    digest: [u8; 32],
) -> Result<(), OpcodeResolutionError<F>> {
    for (output_witness, value) in outputs.iter().zip(digest.into_iter()) {
        insert_value(output_witness, F::from_be_bytes_reduce(&[value]), initial_witness)?;
    }

    Ok(())
}

fn to_u32_array<const N: usize, F: AcirField>(
    initial_witness: &WitnessMap<F>,
    inputs: &[FunctionInput<F>; N],
) -> Result<[u32; N], OpcodeResolutionError<F>> {
    let mut result = [0; N];
    for (it, input) in result.iter_mut().zip(inputs) {
        let witness_value = input_to_value(initial_witness, *input)?;
        *it = witness_value
            .try_into_u128()
            .expect("expected the 'witness_value' to fit into a u128") as u32;
    }
    Ok(result)
}

pub(crate) fn solve_sha_256_permutation_opcode<F: AcirField>(
    initial_witness: &mut WitnessMap<F>,
    inputs: &[FunctionInput<F>; 16],
    hash_values: &[FunctionInput<F>; 8],
    outputs: &[Witness; 8],
) -> Result<(), OpcodeResolutionError<F>> {
    let state = execute_sha_256_permutation_opcode(initial_witness, inputs, hash_values)?;

    for (output_witness, value) in outputs.iter().zip(state.into_iter()) {
        insert_value(output_witness, F::from(u128::from(value)), initial_witness)?;
    }

    Ok(())
}

pub(crate) fn execute_sha_256_permutation_opcode<F: AcirField>(
    initial_witness: &WitnessMap<F>,
    inputs: &[FunctionInput<F>; 16],
    hash_values: &[FunctionInput<F>; 8],
) -> Result<[u32; 8], OpcodeResolutionError<F>> {
    let message = to_u32_array(initial_witness, inputs)?;
    let mut state = to_u32_array(initial_witness, hash_values)?;

    sha256_compression(&mut state, &message);

    Ok(state)
}

pub(crate) fn solve_poseidon2_permutation_opcode<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &mut WitnessMap<F>,
    inputs: &[FunctionInput<F>],
    outputs: &[Witness],
) -> Result<(), OpcodeResolutionError<F>> {
    if inputs.len() != outputs.len() {
        return Err(OpcodeResolutionError::BlackBoxFunctionFailed(
            acir::BlackBoxFunc::Poseidon2Permutation,
            format!(
                "the input and output sizes are not consistent. {} != {}",
                inputs.len(),
                outputs.len()
            ),
        ));
    }

    let state = execute_poseidon2_permutation_opcode(backend, initial_witness, inputs)?;

    // Write witness assignments
    for (output_witness, value) in outputs.iter().zip(state.into_iter()) {
        insert_value(output_witness, value, initial_witness)?;
    }
    Ok(())
}

pub(crate) fn execute_poseidon2_permutation_opcode<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &WitnessMap<F>,
    inputs: &[FunctionInput<F>],
) -> Result<Vec<F>, OpcodeResolutionError<F>> {
    // Read witness assignments
    let state: Vec<F> = inputs
        .iter()
        .map(|input| input_to_value(initial_witness, *input))
        .collect::<Result<_, _>>()?;

    let state = backend.poseidon2_permutation(&state)?;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use crate::pwg::blackbox::solve_generic_256_hash_opcode;
    use acir::{
        FieldElement,
        circuit::opcodes::FunctionInput,
        native_types::{Witness, WitnessMap},
    };
    use acvm_blackbox_solver::{blake2s, blake3};
    use std::collections::BTreeMap;

    #[test]
    fn test_blake2s() {
        // Test vector is coming from Barretenberg (cf. blake2s.test.cpp)
        let mut inputs = Vec::new();
        for i in 0..3 {
            inputs.push(FunctionInput::Witness(Witness(1 + i)));
        }
        let mut outputs = [Witness(0); 32];
        #[allow(clippy::needless_range_loop)]
        for i in 0..32 {
            outputs[i] = Witness(4 + i as u32);
        }

        let mut initial_witness = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from('a' as u128)),
            (Witness(2), FieldElement::from('b' as u128)),
            (Witness(3), FieldElement::from('c' as u128)),
        ]));

        solve_generic_256_hash_opcode(&mut initial_witness, &inputs, None, &outputs, blake2s)
            .unwrap();

        let expected_output: [u128; 32] = [
            0x50, 0x8C, 0x5E, 0x8C, 0x32, 0x7C, 0x14, 0xE2, 0xE1, 0xA7, 0x2B, 0xA3, 0x4E, 0xEB,
            0x45, 0x2F, 0x37, 0x45, 0x8B, 0x20, 0x9E, 0xD6, 0x3A, 0x29, 0x4D, 0x99, 0x9B, 0x4C,
            0x86, 0x67, 0x59, 0x82,
        ];
        let expected_output = expected_output.map(FieldElement::from);
        let expected_output: Vec<&FieldElement> = expected_output.iter().collect();
        for i in 0..32 {
            assert_eq!(initial_witness[&Witness(4 + i as u32)], *expected_output[i]);
        }
    }

    #[test]
    fn test_blake3s() {
        // Test vector is coming from Barretenberg (cf. blake3s.test.cpp)
        let mut inputs = Vec::new();
        for i in 0..3 {
            inputs.push(FunctionInput::Witness(Witness(1 + i)));
        }
        let mut outputs = [Witness(0); 32];
        #[allow(clippy::needless_range_loop)]
        for i in 0..32 {
            outputs[i] = Witness(4 + i as u32);
        }

        let mut initial_witness = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from('a' as u128)),
            (Witness(2), FieldElement::from('b' as u128)),
            (Witness(3), FieldElement::from('c' as u128)),
        ]));

        solve_generic_256_hash_opcode(&mut initial_witness, &inputs, None, &outputs, blake3)
            .unwrap();

        let expected_output: [u128; 32] = [
            0x64, 0x37, 0xB3, 0xAC, 0x38, 0x46, 0x51, 0x33, 0xFF, 0xB6, 0x3B, 0x75, 0x27, 0x3A,
            0x8D, 0xB5, 0x48, 0xC5, 0x58, 0x46, 0x5D, 0x79, 0xDB, 0x03, 0xFD, 0x35, 0x9C, 0x6C,
            0xD5, 0xBD, 0x9D, 0x85,
        ];
        let expected_output = expected_output.map(FieldElement::from);
        let expected_output: Vec<&FieldElement> = expected_output.iter().collect();
        for i in 0..32 {
            assert_eq!(initial_witness[&Witness(4 + i as u32)], *expected_output[i]);
        }
    }
}
