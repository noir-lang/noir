use acir::{AcirField, circuit::opcodes::FunctionInput, native_types::WitnessMap};
use acvm_blackbox_solver::{ecdsa_secp256k1_verify, ecdsa_secp256r1_verify};

use crate::{
    OpcodeResolutionError,
    pwg::{blackbox::utils::to_u8_array, input_to_value},
};

pub(crate) fn execute_ecdsa<F: AcirField>(
    initial_witness: &WitnessMap<F>,
    public_key_x_inputs: &[FunctionInput<F>; 32],
    public_key_y_inputs: &[FunctionInput<F>; 32],
    signature_inputs: &[FunctionInput<F>; 64],
    hashed_message_inputs: &[FunctionInput<F>; 32],
    predicate: &FunctionInput<F>,
    k1: bool,
) -> Result<bool, OpcodeResolutionError<F>> {
    let pub_key_x: [u8; 32] = to_u8_array(initial_witness, public_key_x_inputs)?;
    let pub_key_y: [u8; 32] = to_u8_array(initial_witness, public_key_y_inputs)?;
    let signature: [u8; 64] = to_u8_array(initial_witness, signature_inputs)?;
    let hashed_message: [u8; 32] = to_u8_array(initial_witness, hashed_message_inputs)?;
    let predicate = input_to_value(initial_witness, *predicate)?.is_one();
    let is_valid = if predicate {
        if k1 {
            ecdsa_secp256k1_verify(&hashed_message, &pub_key_x, &pub_key_y, &signature)?
        } else {
            ecdsa_secp256r1_verify(&hashed_message, &pub_key_x, &pub_key_y, &signature)?
        }
    } else {
        true
    };

    Ok(is_valid)
}
