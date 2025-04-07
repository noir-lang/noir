// use acir::{
//     AcirField,
//     circuit::opcodes::FunctionInput,
//     native_types::{Witness, WitnessMap},
// };
// use acvm_blackbox_solver::{BlackBoxFunctionSolver, aes128_encrypt};

// use crate::{OpcodeResolutionError, pwg::insert_value};

// use super::utils::{to_u8_array, to_u8_vec};

// pub(crate) fn get_random<F: AcirField>(
//     backend: &impl BlackBoxFunctionSolver<F>,
//     outputs: &[Witness],
// ) -> Result<(), OpcodeResolutionError<F>> {
//     let state = backend.poseidon2_permutation(&state, len)?;

//     // Write witness assignments
//     for (output_witness, value) in outputs.iter().zip(state.into_iter()) {
//         insert_value(output_witness, value, initial_witness)?;
//     }
//     Ok(())
// }
