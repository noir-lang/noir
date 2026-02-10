use acir::{
    AcirField,
    circuit::opcodes::{BlackBoxFuncCall, FunctionInput},
    native_types::{Witness, WitnessMap},
};
use acvm_blackbox_solver::{blake2s, blake3, keccakf1600};

use self::{aes128::solve_aes128_encryption_opcode, hash::solve_poseidon2_permutation_opcode};

use super::{OpcodeNotSolvable, OpcodeResolutionError, insert_value};
use crate::{BlackBoxFunctionSolver, pwg::input_to_value};

pub(crate) mod aes128;
pub(crate) mod embedded_curve_ops;
pub(crate) mod hash;
mod logic;
mod range;
pub(crate) mod signature;
pub(crate) mod utils;

use embedded_curve_ops::{embedded_curve_add, multi_scalar_mul};
// Hash functions should eventually be exposed for external consumers.
use hash::{solve_generic_256_hash_opcode, solve_sha_256_permutation_opcode};
use logic::{and, xor};
pub(crate) use range::solve_range_opcode;
use signature::ecdsa::{secp256k1_prehashed, secp256r1_prehashed};

/// Check if all of the inputs to the function have assignments
///
/// Returns the first missing assignment if any are missing
fn first_missing_assignment<F>(
    witness_assignments: &WitnessMap<F>,
    inputs: &[FunctionInput<F>],
) -> Option<Witness> {
    inputs.iter().find_map(|input| {
        if let FunctionInput::Witness(witness) = input {
            if witness_assignments.contains_key(witness) { None } else { Some(*witness) }
        } else {
            None
        }
    })
}

/// Check if all of the inputs to the function have assignments
fn contains_all_inputs<F>(
    witness_assignments: &WitnessMap<F>,
    inputs: &[FunctionInput<F>],
) -> bool {
    first_missing_assignment(witness_assignments, inputs).is_none()
}

/// Solve a black box function call
/// 1. Returns an error if not all the inputs are already resolved to a value
/// 2. Compute the output from the inputs, using the dedicated solvers
///
/// A blackbox is a fully specified function (e.g sha256, ecdsa signature,...)
/// which the backend can prove execution in a more efficient way than using a generic
/// arithmetic circuit.
/// Solving a black box function simply means to compute the output from the inputs for
/// the specific function.
/// Our black box solver uses the standard rust implementation for the function if it is available.
/// However, some functions depend on the backend, such as embedded curve operations, which depend on the
/// elliptic curve used by the proving system. This is why the 'solve' functions takes a blackbox solver trait.
/// The 'AcvmBigIntSolver' is also a blackbox solver, but dedicated to the BigInteger blackbox functions.
pub(crate) fn solve<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &mut WitnessMap<F>,
    bb_func: &BlackBoxFuncCall<F>,
) -> Result<(), OpcodeResolutionError<F>> {
    let inputs = bb_func.get_inputs_vec();
    if !contains_all_inputs(initial_witness, &inputs) {
        let unassigned_witness = first_missing_assignment(initial_witness, &inputs)
            .expect("Some assignments must be missing because it does not contains all inputs");
        return Err(OpcodeResolutionError::OpcodeNotSolvable(
            OpcodeNotSolvable::MissingAssignment(unassigned_witness.0),
        ));
    }

    match bb_func {
        BlackBoxFuncCall::AES128Encrypt { inputs, iv, key, outputs } => {
            solve_aes128_encryption_opcode(initial_witness, inputs, iv, key, outputs)
        }
        BlackBoxFuncCall::AND { lhs, rhs, num_bits, output } => {
            and(initial_witness, lhs, rhs, *num_bits, output)
        }
        BlackBoxFuncCall::XOR { lhs, rhs, num_bits, output } => {
            xor(initial_witness, lhs, rhs, *num_bits, output)
        }
        BlackBoxFuncCall::RANGE { input, num_bits } => {
            solve_range_opcode(initial_witness, input, *num_bits)
        }
        BlackBoxFuncCall::Blake2s { outputs, .. } => {
            let inputs = bb_func.get_inputs_vec();
            solve_generic_256_hash_opcode(initial_witness, &inputs, None, outputs, blake2s)
        }
        BlackBoxFuncCall::Blake3 { outputs, .. } => {
            let inputs = bb_func.get_inputs_vec();
            solve_generic_256_hash_opcode(initial_witness, &inputs, None, outputs, blake3)
        }
        BlackBoxFuncCall::Keccakf1600 { inputs, outputs } => {
            let mut state = [0; 25];
            for (it, input) in state.iter_mut().zip(inputs.as_ref()) {
                let witness_assignment = input_to_value(initial_witness, *input)?;
                let lane = witness_assignment.try_to_u64();
                *it = lane.unwrap();
            }
            let output_state = keccakf1600(state)?;
            for (output_witness, value) in outputs.iter().zip(output_state.into_iter()) {
                insert_value(output_witness, F::from(u128::from(value)), initial_witness)?;
            }
            Ok(())
        }
        BlackBoxFuncCall::EcdsaSecp256k1 {
            public_key_x,
            public_key_y,
            signature,
            hashed_message: message,
            output,
            predicate,
        } => secp256k1_prehashed(
            initial_witness,
            public_key_x,
            public_key_y,
            signature,
            message.as_ref(),
            predicate,
            *output,
        ),
        BlackBoxFuncCall::EcdsaSecp256r1 {
            public_key_x,
            public_key_y,
            signature,
            hashed_message: message,
            output,
            predicate,
        } => secp256r1_prehashed(
            initial_witness,
            public_key_x,
            public_key_y,
            signature,
            message.as_ref(),
            predicate,
            *output,
        ),
        BlackBoxFuncCall::MultiScalarMul { points, scalars, outputs, predicate } => {
            multi_scalar_mul(backend, initial_witness, points, scalars, *predicate, *outputs)
        }
        BlackBoxFuncCall::EmbeddedCurveAdd { input1, input2, outputs, predicate } => {
            embedded_curve_add(backend, initial_witness, **input1, **input2, *predicate, *outputs)
        }
        // Recursive aggregation will be entirely handled by the backend and is not solved by the ACVM
        BlackBoxFuncCall::RecursiveAggregation { .. } => Ok(()),
        BlackBoxFuncCall::Sha256Compression { inputs, hash_values, outputs } => {
            solve_sha_256_permutation_opcode(initial_witness, inputs, hash_values, outputs)
        }
        BlackBoxFuncCall::Poseidon2Permutation { inputs, outputs } => {
            solve_poseidon2_permutation_opcode(backend, initial_witness, inputs, outputs)
        }
    }
}
