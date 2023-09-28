use super::to_u8_vec;
use crate::{
    pwg::{insert_value, witness_to_value, OpcodeResolutionError},
    BlackBoxFunctionSolver,
};
use acir::{
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
    FieldElement,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn schnorr_verify(
    backend: &impl BlackBoxFunctionSolver,
    initial_witness: &mut WitnessMap,
    public_key_x: FunctionInput,
    public_key_y: FunctionInput,
    signature: &[FunctionInput],
    message: &[FunctionInput],
    output: Witness,
) -> Result<(), OpcodeResolutionError> {
    let public_key_x: &FieldElement = witness_to_value(initial_witness, public_key_x.witness)?;
    let public_key_y: &FieldElement = witness_to_value(initial_witness, public_key_y.witness)?;

    let signature = to_u8_vec(initial_witness, signature)?;

    let message = to_u8_vec(initial_witness, message)?;

    let valid_signature =
        backend.schnorr_verify(public_key_x, public_key_y, &signature, &message)?;

    insert_value(&output, FieldElement::from(valid_signature), initial_witness)?;

    Ok(())
}
