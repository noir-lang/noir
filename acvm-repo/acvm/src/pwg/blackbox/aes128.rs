use acir::{
    AcirField,
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
};
use acvm_blackbox_solver::aes128_encrypt;

use crate::{OpcodeResolutionError, pwg::insert_value};

use super::utils::{to_u8_array, to_u8_vec};

pub(super) fn solve_aes128_encryption_opcode<F: AcirField>(
    initial_witness: &mut WitnessMap<F>,
    inputs: &[FunctionInput<F>],
    iv: &[FunctionInput<F>; 16],
    key: &[FunctionInput<F>; 16],
    outputs: &[Witness],
) -> Result<(), OpcodeResolutionError<F>> {
    let scalars = to_u8_vec(initial_witness, inputs)?;

    let iv = to_u8_array(initial_witness, iv)?;
    let key = to_u8_array(initial_witness, key)?;

    let ciphertext = aes128_encrypt(&scalars, iv, key)?;

    // Write witness assignments
    for (output_witness, value) in outputs.iter().zip(ciphertext.into_iter()) {
        insert_value(output_witness, F::from(value as u128), initial_witness)?;
    }

    Ok(())
}
