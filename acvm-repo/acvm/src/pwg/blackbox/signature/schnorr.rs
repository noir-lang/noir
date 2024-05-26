use crate::{
    pwg::{
        blackbox::utils::{to_u8_array, to_u8_vec},
        insert_value, witness_to_value, OpcodeResolutionError,
    },
    BlackBoxFunctionSolver,
};
use acir::{
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
    AcirField,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn schnorr_verify<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &mut WitnessMap<F>,
    public_key_x: FunctionInput,
    public_key_y: FunctionInput,
    signature: &[FunctionInput; 64],
    message: &[FunctionInput],
    output: Witness,
) -> Result<(), OpcodeResolutionError<F>> {
    let public_key_x: &F = witness_to_value(initial_witness, public_key_x.witness)?;
    let public_key_y: &F = witness_to_value(initial_witness, public_key_y.witness)?;

    let signature = to_u8_array(initial_witness, signature)?;
    let message = to_u8_vec(initial_witness, message)?;

    let valid_signature =
        backend.schnorr_verify(public_key_x, public_key_y, &signature, &message)?;

    insert_value(&output, F::from(valid_signature), initial_witness)?;

    Ok(())
}
