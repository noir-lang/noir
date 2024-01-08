use acir::{
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
    BlackBoxFunc, FieldElement,
};
use acvm_blackbox_solver::BlackBoxResolutionError;

use crate::pwg::{insert_value, witness_to_value};
use crate::OpcodeResolutionError;

/// Attempts to solve a 256 bit hash function opcode.
/// If successful, `initial_witness` will be mutated to contain the new witness assignment.
pub(super) fn solve_generic_256_hash_opcode(
    initial_witness: &mut WitnessMap,
    inputs: &[FunctionInput],
    var_message_size: Option<&FunctionInput>,
    outputs: &[Witness],
    hash_function: fn(data: &[u8]) -> Result<[u8; 32], BlackBoxResolutionError>,
    black_box_func: BlackBoxFunc,
) -> Result<(), OpcodeResolutionError> {
    let message_input = get_hash_input(initial_witness, inputs, var_message_size)?;
    let digest: [u8; 32] = hash_function(&message_input)?;

    let outputs: [Witness; 32] = outputs.try_into().map_err(|_| {
        OpcodeResolutionError::BlackBoxFunctionFailed(
            black_box_func,
            format!("Expected 32 outputs but encountered {}", outputs.len()),
        )
    })?;
    write_digest_to_outputs(initial_witness, outputs, digest)?;

    Ok(())
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
    outputs: [Witness; 32],
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

const ROUNDS: usize = 24;

const RC: [u64; ROUNDS] = [
    1u64,
    0x8082u64,
    0x800000000000808au64,
    0x8000000080008000u64,
    0x808bu64,
    0x80000001u64,
    0x8000000080008081u64,
    0x8000000000008009u64,
    0x8au64,
    0x88u64,
    0x80008009u64,
    0x8000000au64,
    0x8000808bu64,
    0x800000000000008bu64,
    0x8000000000008089u64,
    0x8000000000008003u64,
    0x8000000000008002u64,
    0x8000000000000080u64,
    0x800au64,
    0x800000008000000au64,
    0x8000000080008081u64,
    0x8000000000008080u64,
    0x80000001u64,
    0x8000000080008008u64,
];

const RHO: [u32; 24] =
    [1, 3, 6, 10, 15, 21, 28, 36, 45, 55, 2, 14, 27, 41, 56, 8, 25, 43, 62, 18, 39, 61, 20, 44];

const PI: [usize; 24] =
    [10, 7, 11, 17, 18, 3, 5, 16, 8, 21, 24, 4, 15, 23, 19, 13, 12, 2, 20, 14, 22, 9, 6, 1];

const KECCAK_LANES: usize = 25;

pub(crate) fn keccakf1600(state: &mut [u64; KECCAK_LANES]) {
    for rc in RC {
        let mut array: [u64; 5] = [0; 5];

        // Theta
        for x in 0..5 {
            for y_count in 0..5 {
                let y = y_count * 5;
                array[x] ^= state[x + y];
            }
        }

        for x in 0..5 {
            for y_count in 0..5 {
                let y = y_count * 5;
                state[y + x] ^= array[(x + 4) % 5] ^ array[(x + 1) % 5].rotate_left(1);
            }
        }

        // Rho and pi
        let mut last = state[1];
        for x in 0..24 {
            array[0] = state[PI[x]];
            state[PI[x]] = last.rotate_left(RHO[x]);
            last = array[0];
        }

        // Chi
        for y_step in 0..5 {
            let y = y_step * 5;
            array[..5].copy_from_slice(&state[y..(5 + y)]);

            for x in 0..5 {
                state[y + x] = array[x] ^ ((!array[(x + 1) % 5]) & (array[(x + 2) % 5]));
            }
        }

        // Iota
        state[0] ^= rc;
    }
}
