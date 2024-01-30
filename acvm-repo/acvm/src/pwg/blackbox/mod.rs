use acir::{
    circuit::opcodes::{BlackBoxFuncCall, FunctionInput},
    native_types::{Witness, WitnessMap},
    FieldElement,
};
use acvm_blackbox_solver::{blake2s, blake3, keccak256, keccakf1600, sha256};

use self::pedersen::pedersen_hash;

use super::{insert_value, OpcodeNotSolvable, OpcodeResolutionError};
use crate::{pwg::witness_to_value, BlackBoxFunctionSolver};

mod fixed_base_scalar_mul;
mod hash;
mod logic;
mod pedersen;
mod range;
mod signature;

use fixed_base_scalar_mul::{embedded_curve_add, fixed_base_scalar_mul};
// Hash functions should eventually be exposed for external consumers.
use hash::solve_generic_256_hash_opcode;
use logic::{and, xor};
use pedersen::pedersen;
use range::solve_range_opcode;
use signature::{
    ecdsa::{secp256k1_prehashed, secp256r1_prehashed},
    schnorr::schnorr_verify,
};

/// Check if all of the inputs to the function have assignments
///
/// Returns the first missing assignment if any are missing
fn first_missing_assignment(
    witness_assignments: &WitnessMap,
    inputs: &[FunctionInput],
) -> Option<Witness> {
    inputs.iter().find_map(|input| {
        if witness_assignments.contains_key(&input.witness) {
            None
        } else {
            Some(input.witness)
        }
    })
}

/// Check if all of the inputs to the function have assignments
fn contains_all_inputs(witness_assignments: &WitnessMap, inputs: &[FunctionInput]) -> bool {
    inputs.iter().all(|input| witness_assignments.contains_key(&input.witness))
}

pub(crate) fn solve(
    backend: &impl BlackBoxFunctionSolver,
    initial_witness: &mut WitnessMap,
    bb_func: &BlackBoxFuncCall,
) -> Result<(), OpcodeResolutionError> {
    let inputs = bb_func.get_inputs_vec();
    if !contains_all_inputs(initial_witness, &inputs) {
        let unassigned_witness = first_missing_assignment(initial_witness, &inputs)
            .expect("Some assignments must be missing because it does not contains all inputs");
        return Err(OpcodeResolutionError::OpcodeNotSolvable(
            OpcodeNotSolvable::MissingAssignment(unassigned_witness.0),
        ));
    }

    match bb_func {
        BlackBoxFuncCall::AND { lhs, rhs, output } => and(initial_witness, lhs, rhs, output),
        BlackBoxFuncCall::XOR { lhs, rhs, output } => xor(initial_witness, lhs, rhs, output),
        BlackBoxFuncCall::RANGE { input } => solve_range_opcode(initial_witness, input),
        BlackBoxFuncCall::SHA256 { inputs, outputs } => solve_generic_256_hash_opcode(
            initial_witness,
            inputs,
            None,
            outputs,
            sha256,
            bb_func.get_black_box_func(),
        ),
        BlackBoxFuncCall::Blake2s { inputs, outputs } => solve_generic_256_hash_opcode(
            initial_witness,
            inputs,
            None,
            outputs,
            blake2s,
            bb_func.get_black_box_func(),
        ),
        BlackBoxFuncCall::Blake3 { inputs, outputs } => solve_generic_256_hash_opcode(
            initial_witness,
            inputs,
            None,
            outputs,
            blake3,
            bb_func.get_black_box_func(),
        ),
        BlackBoxFuncCall::Keccak256 { inputs, outputs } => solve_generic_256_hash_opcode(
            initial_witness,
            inputs,
            None,
            outputs,
            keccak256,
            bb_func.get_black_box_func(),
        ),
        BlackBoxFuncCall::Keccak256VariableLength { inputs, var_message_size, outputs } => {
            solve_generic_256_hash_opcode(
                initial_witness,
                inputs,
                Some(var_message_size),
                outputs,
                keccak256,
                bb_func.get_black_box_func(),
            )
        }
        BlackBoxFuncCall::Keccakf1600 { inputs, outputs } => {
            let mut state = [0; 25];
            for (i, input) in inputs.iter().enumerate() {
                let witness = input.witness;
                let num_bits = input.num_bits as usize;
                assert_eq!(num_bits, 64);
                let witness_assignment = witness_to_value(initial_witness, witness)?;
                let lane = witness_assignment.try_to_u64();
                state[i] = lane.unwrap();
            }
            let output_state = keccakf1600(state)?;
            for (output_witness, value) in outputs.iter().zip(output_state.into_iter()) {
                insert_value(output_witness, FieldElement::from(value as u128), initial_witness)?;
            }
            Ok(())
        }
        BlackBoxFuncCall::SchnorrVerify {
            public_key_x,
            public_key_y,
            signature,
            message,
            output,
        } => schnorr_verify(
            backend,
            initial_witness,
            *public_key_x,
            *public_key_y,
            signature,
            message,
            *output,
        ),
        BlackBoxFuncCall::PedersenCommitment { inputs, domain_separator, outputs } => {
            pedersen(backend, initial_witness, inputs, *domain_separator, *outputs)
        }
        BlackBoxFuncCall::PedersenHash { inputs, domain_separator, output } => {
            pedersen_hash(backend, initial_witness, inputs, *domain_separator, *output)
        }
        BlackBoxFuncCall::EcdsaSecp256k1 {
            public_key_x,
            public_key_y,
            signature,
            hashed_message: message,
            output,
        } => secp256k1_prehashed(
            initial_witness,
            public_key_x,
            public_key_y,
            signature,
            message,
            *output,
        ),
        BlackBoxFuncCall::EcdsaSecp256r1 {
            public_key_x,
            public_key_y,
            signature,
            hashed_message: message,
            output,
        } => secp256r1_prehashed(
            initial_witness,
            public_key_x,
            public_key_y,
            signature,
            message,
            *output,
        ),
        BlackBoxFuncCall::FixedBaseScalarMul { low, high, outputs } => {
            fixed_base_scalar_mul(backend, initial_witness, *low, *high, *outputs)
        }
        BlackBoxFuncCall::EmbeddedCurveAdd { input1_x, input1_y, input2_x, input2_y, outputs } => {
            embedded_curve_add(
                backend,
                initial_witness,
                *input1_x,
                *input1_y,
                *input2_x,
                *input2_y,
                *outputs,
            )
        }
        // Recursive aggregation will be entirely handled by the backend and is not solved by the ACVM
        BlackBoxFuncCall::RecursiveAggregation { .. } => Ok(()),
        BlackBoxFuncCall::BigIntAdd { .. } => todo!(),
        BlackBoxFuncCall::BigIntNeg { .. } => todo!(),
        BlackBoxFuncCall::BigIntMul { .. } => todo!(),
        BlackBoxFuncCall::BigIntDiv { .. } => todo!(),
        BlackBoxFuncCall::BigIntFromLeBytes { .. } => todo!(),
        BlackBoxFuncCall::BigIntToLeBytes { .. } => todo!(),
        BlackBoxFuncCall::Poseidon2Permutation { .. } => todo!(),
        BlackBoxFuncCall::Sha256Compression { .. } => todo!(),
    }
}
