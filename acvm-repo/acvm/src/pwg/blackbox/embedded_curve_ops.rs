use acir::{
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
};
use acvm_blackbox_solver::BlackBoxFunctionSolver;

use crate::pwg::{insert_value, witness_to_value, OpcodeResolutionError};

pub(super) fn multi_scalar_mul(
    backend: &impl BlackBoxFunctionSolver,
    initial_witness: &mut WitnessMap,
    points: &[FunctionInput],
    scalars: &[FunctionInput],
    outputs: (Witness, Witness),
) -> Result<(), OpcodeResolutionError> {
    let points: Result<Vec<_>, _> =
        points.iter().map(|input| witness_to_value(initial_witness, input.witness)).collect();
    let points: Vec<_> = points?.into_iter().cloned().collect();

    let scalars: Result<Vec<_>, _> =
        scalars.iter().map(|input| witness_to_value(initial_witness, input.witness)).collect();
    let scalars: Vec<_> = scalars?.into_iter().cloned().collect();

    // Call the backend's multi-scalar multiplication function
    let (res_x, res_y) = backend.multi_scalar_mul(&points, &scalars)?;

    // Insert the resulting point into the witness map
    insert_value(&outputs.0, res_x, initial_witness)?;
    insert_value(&outputs.1, res_y, initial_witness)?;

    Ok(())
}

pub(super) fn embedded_curve_add(
    backend: &impl BlackBoxFunctionSolver,
    initial_witness: &mut WitnessMap,
    input1_x: FunctionInput,
    input1_y: FunctionInput,
    input2_x: FunctionInput,
    input2_y: FunctionInput,
    outputs: (Witness, Witness),
) -> Result<(), OpcodeResolutionError> {
    let input1_x = witness_to_value(initial_witness, input1_x.witness)?;
    let input1_y = witness_to_value(initial_witness, input1_y.witness)?;
    let input2_x = witness_to_value(initial_witness, input2_x.witness)?;
    let input2_y = witness_to_value(initial_witness, input2_y.witness)?;
    let (res_x, res_y) = backend.ec_add(input1_x, input1_y, input2_x, input2_y)?;

    insert_value(&outputs.0, res_x, initial_witness)?;
    insert_value(&outputs.1, res_y, initial_witness)?;

    Ok(())
}
