use acir::{
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
    FieldElement,
};
use acvm_blackbox_solver::{sha256compression, BlackBoxFunctionSolver, BlackBoxResolutionError};

use crate::pwg::{insert_value, witness_to_value};
use crate::OpcodeResolutionError;

/// Attempts to solve a 256 bit hash function opcode.
/// If successful, `initial_witness` will be mutated to contain the new witness assignment.
pub(super) fn solve_generic_256_hash_opcode(
    initial_witness: &mut WitnessMap,
    inputs: &[FunctionInput],
    var_message_size: Option<&FunctionInput>,
    outputs: &[Witness; 32],
    hash_function: fn(data: &[u8]) -> Result<[u8; 32], BlackBoxResolutionError>,
) -> Result<(), OpcodeResolutionError> {
    let message_input = get_hash_input(initial_witness, inputs, var_message_size)?;
    let digest: [u8; 32] = hash_function(&message_input)?;

    write_digest_to_outputs(initial_witness, outputs, digest)
}

/// Reads the hash function input from a [`WitnessMap`].
fn get_hash_input(
    initial_witness: &WitnessMap,
    inputs: &[FunctionInput],
    message_size: Option<&FunctionInput>,
) -> Result<Vec<u8>, OpcodeResolutionError> {
    // Read witness assignments.
    let mut message_input = Vec::new();
    for input in inputs.iter() {
        let witness = input.witness;
        let num_bits = input.num_bits as usize;

        let witness_assignment = witness_to_value(initial_witness, witness)?;
        let bytes = witness_assignment.fetch_nearest_bytes(num_bits);
        message_input.extend(bytes);
    }

    // Truncate the message if there is a `message_size` parameter given
    match message_size {
        Some(input) => {
            let num_bytes_to_take =
                witness_to_value(initial_witness, input.witness)?.to_u128() as usize;

            // If the number of bytes to take is more than the amount of bytes available
            // in the message, then we error.
            if num_bytes_to_take > message_input.len() {
                return Err(OpcodeResolutionError::BlackBoxFunctionFailed(
                        acir::BlackBoxFunc::Keccak256,
                        format!("the number of bytes to take from the message is more than the number of bytes in the message. {} > {}", num_bytes_to_take, message_input.len()),
                    ));
            }
            let truncated_message = message_input[0..num_bytes_to_take].to_vec();
            Ok(truncated_message)
        }
        None => Ok(message_input),
    }
}

/// Writes a `digest` to the [`WitnessMap`] at witness indices `outputs`.
fn write_digest_to_outputs(
    initial_witness: &mut WitnessMap,
    outputs: &[Witness; 32],
    digest: [u8; 32],
) -> Result<(), OpcodeResolutionError> {
    for (output_witness, value) in outputs.iter().zip(digest.into_iter()) {
        insert_value(
            output_witness,
            FieldElement::from_be_bytes_reduce(&[value]),
            initial_witness,
        )?;
    }

    Ok(())
}

fn to_u32_array<const N: usize>(
    initial_witness: &WitnessMap,
    inputs: &[FunctionInput; N],
) -> Result<[u32; N], OpcodeResolutionError> {
    let mut result = [0; N];
    for (it, input) in result.iter_mut().zip(inputs) {
        let witness_value = witness_to_value(initial_witness, input.witness)?;
        *it = witness_value.to_u128() as u32;
    }
    Ok(result)
}

pub(crate) fn solve_sha_256_permutation_opcode(
    initial_witness: &mut WitnessMap,
    inputs: &[FunctionInput; 16],
    hash_values: &[FunctionInput; 8],
    outputs: &[Witness; 8],
) -> Result<(), OpcodeResolutionError> {
    let message = to_u32_array(initial_witness, inputs)?;
    let mut state = to_u32_array(initial_witness, hash_values)?;

    sha256compression(&mut state, &message);

    for (output_witness, value) in outputs.iter().zip(state.into_iter()) {
        insert_value(output_witness, FieldElement::from(value as u128), initial_witness)?;
    }

    Ok(())
}

pub(crate) fn solve_poseidon2_permutation_opcode(
    backend: &impl BlackBoxFunctionSolver,
    initial_witness: &mut WitnessMap,
    inputs: &[FunctionInput],
    outputs: &[Witness],
    len: u32,
) -> Result<(), OpcodeResolutionError> {
    if len as usize != inputs.len() {
        return Err(OpcodeResolutionError::BlackBoxFunctionFailed(
            acir::BlackBoxFunc::Poseidon2Permutation,
            format!(
                "the number of inputs does not match specified length. {} != {}",
                inputs.len(),
                len
            ),
        ));
    }
    if len as usize != outputs.len() {
        return Err(OpcodeResolutionError::BlackBoxFunctionFailed(
            acir::BlackBoxFunc::Poseidon2Permutation,
            format!(
                "the number of outputs does not match specified length. {} != {}",
                outputs.len(),
                len
            ),
        ));
    }

    // Read witness assignments
    let mut state = Vec::new();
    for input in inputs.iter() {
        let witness_assignment = witness_to_value(initial_witness, input.witness)?;
        state.push(*witness_assignment);
    }

    let state = backend.poseidon2_permutation(&state, len)?;

    // Write witness assignments
    for (output_witness, value) in outputs.iter().zip(state.into_iter()) {
        insert_value(output_witness, value, initial_witness)?;
    }
    Ok(())
}
