use acir::{
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
    AcirField,
};
use acvm_blackbox_solver::{ecdsa_secp256k1_verify, ecdsa_secp256r1_verify};

use crate::{
    pwg::{
        blackbox::utils::{to_u8_array, to_u8_vec},
        insert_value,
    },
    OpcodeResolutionError,
};

pub(crate) fn secp256k1_prehashed<F: AcirField>(
    initial_witness: &mut WitnessMap<F>,
    public_key_x_inputs: &[FunctionInput; 32],
    public_key_y_inputs: &[FunctionInput; 32],
    signature_inputs: &[FunctionInput; 64],
    hashed_message_inputs: &[FunctionInput],
    output: Witness,
) -> Result<(), OpcodeResolutionError<F>> {
    let hashed_message = to_u8_vec(initial_witness, hashed_message_inputs)?;

    let pub_key_x: [u8; 32] = to_u8_array(initial_witness, public_key_x_inputs)?;
    let pub_key_y: [u8; 32] = to_u8_array(initial_witness, public_key_y_inputs)?;
    let signature: [u8; 64] = to_u8_array(initial_witness, signature_inputs)?;

    let is_valid = ecdsa_secp256k1_verify(&hashed_message, &pub_key_x, &pub_key_y, &signature)?;

    insert_value(&output, F::from(is_valid), initial_witness)
}

pub(crate) fn secp256r1_prehashed<F: AcirField>(
    initial_witness: &mut WitnessMap<F>,
    public_key_x_inputs: &[FunctionInput; 32],
    public_key_y_inputs: &[FunctionInput; 32],
    signature_inputs: &[FunctionInput; 64],
    hashed_message_inputs: &[FunctionInput],
    output: Witness,
) -> Result<(), OpcodeResolutionError<F>> {
    let hashed_message = to_u8_vec(initial_witness, hashed_message_inputs)?;

    let pub_key_x: [u8; 32] = to_u8_array(initial_witness, public_key_x_inputs)?;
    let pub_key_y: [u8; 32] = to_u8_array(initial_witness, public_key_y_inputs)?;
    let signature: [u8; 64] = to_u8_array(initial_witness, signature_inputs)?;

    let is_valid = ecdsa_secp256r1_verify(&hashed_message, &pub_key_x, &pub_key_y, &signature)?;

    insert_value(&output, F::from(is_valid), initial_witness)
}
