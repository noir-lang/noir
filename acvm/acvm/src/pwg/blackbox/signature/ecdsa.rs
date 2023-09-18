use acir::{
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
    FieldElement,
};
use blackbox_solver::{ecdsa_secp256k1_verify, ecdsa_secp256r1_verify};

use crate::{pwg::insert_value, OpcodeResolutionError};

use super::to_u8_vec;

pub(crate) fn secp256k1_prehashed(
    initial_witness: &mut WitnessMap,
    public_key_x_inputs: &[FunctionInput],
    public_key_y_inputs: &[FunctionInput],
    signature_inputs: &[FunctionInput],
    hashed_message_inputs: &[FunctionInput],
    output: Witness,
) -> Result<(), OpcodeResolutionError> {
    let hashed_message = to_u8_vec(initial_witness, hashed_message_inputs)?;

    // These errors should never be emitted in practice as they would imply malformed ACIR generation.
    let pub_key_x: [u8; 32] =
        to_u8_vec(initial_witness, public_key_x_inputs)?.try_into().map_err(|_| {
            OpcodeResolutionError::BlackBoxFunctionFailed(
                acir::BlackBoxFunc::EcdsaSecp256k1,
                format!("expected pubkey_x size 32 but received {}", public_key_x_inputs.len()),
            )
        })?;

    let pub_key_y: [u8; 32] =
        to_u8_vec(initial_witness, public_key_y_inputs)?.try_into().map_err(|_| {
            OpcodeResolutionError::BlackBoxFunctionFailed(
                acir::BlackBoxFunc::EcdsaSecp256k1,
                format!("expected pubkey_y size 32 but received {}", public_key_y_inputs.len()),
            )
        })?;

    let signature: [u8; 64] =
        to_u8_vec(initial_witness, signature_inputs)?.try_into().map_err(|_| {
            OpcodeResolutionError::BlackBoxFunctionFailed(
                acir::BlackBoxFunc::EcdsaSecp256k1,
                format!("expected signature size 64 but received {}", signature_inputs.len()),
            )
        })?;

    let is_valid = ecdsa_secp256k1_verify(&hashed_message, &pub_key_x, &pub_key_y, &signature)?;

    insert_value(&output, FieldElement::from(is_valid), initial_witness)?;
    Ok(())
}

pub(crate) fn secp256r1_prehashed(
    initial_witness: &mut WitnessMap,
    public_key_x_inputs: &[FunctionInput],
    public_key_y_inputs: &[FunctionInput],
    signature_inputs: &[FunctionInput],
    hashed_message_inputs: &[FunctionInput],
    output: Witness,
) -> Result<(), OpcodeResolutionError> {
    let hashed_message = to_u8_vec(initial_witness, hashed_message_inputs)?;

    let pub_key_x: [u8; 32] =
        to_u8_vec(initial_witness, public_key_x_inputs)?.try_into().map_err(|_| {
            OpcodeResolutionError::BlackBoxFunctionFailed(
                acir::BlackBoxFunc::EcdsaSecp256r1,
                format!("expected pubkey_x size 32 but received {}", public_key_x_inputs.len()),
            )
        })?;

    let pub_key_y: [u8; 32] =
        to_u8_vec(initial_witness, public_key_y_inputs)?.try_into().map_err(|_| {
            OpcodeResolutionError::BlackBoxFunctionFailed(
                acir::BlackBoxFunc::EcdsaSecp256r1,
                format!("expected pubkey_y size 32 but received {}", public_key_y_inputs.len()),
            )
        })?;

    let signature: [u8; 64] =
        to_u8_vec(initial_witness, signature_inputs)?.try_into().map_err(|_| {
            OpcodeResolutionError::BlackBoxFunctionFailed(
                acir::BlackBoxFunc::EcdsaSecp256r1,
                format!("expected signature size 64 but received {}", signature_inputs.len()),
            )
        })?;

    let is_valid = ecdsa_secp256r1_verify(&hashed_message, &pub_key_x, &pub_key_y, &signature)?;

    insert_value(&output, FieldElement::from(is_valid), initial_witness)?;
    Ok(())
}
