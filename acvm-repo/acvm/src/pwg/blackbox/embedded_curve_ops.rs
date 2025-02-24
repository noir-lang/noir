use acir::{
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
    AcirField,
};
use acvm_blackbox_solver::BlackBoxFunctionSolver;

use crate::pwg::{input_to_value, insert_value, OpcodeResolutionError};

pub(super) fn multi_scalar_mul<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &mut WitnessMap<F>,
    points: &[FunctionInput<F>],
    scalars: &[FunctionInput<F>],
    outputs: (Witness, Witness, Witness),
) -> Result<(), OpcodeResolutionError<F>> {
    let points: Result<Vec<_>, _> =
        points.iter().map(|input| input_to_value(initial_witness, *input, false)).collect();
    let points: Vec<_> = points?.into_iter().collect();

    let scalars: Result<Vec<_>, _> =
        scalars.iter().map(|input| input_to_value(initial_witness, *input, false)).collect();
    let mut scalars_lo = Vec::new();
    let mut scalars_hi = Vec::new();
    for (i, scalar) in scalars?.into_iter().enumerate() {
        if i % 2 == 0 {
            scalars_lo.push(scalar);
        } else {
            scalars_hi.push(scalar);
        }
    }
    // Call the backend's multi-scalar multiplication function
    let (res_x, res_y, is_infinite) =
        backend.multi_scalar_mul(&points, &scalars_lo, &scalars_hi)?;

    // Insert the resulting point into the witness map
    insert_value(&outputs.0, res_x, initial_witness)?;
    insert_value(&outputs.1, res_y, initial_witness)?;
    insert_value(&outputs.2, is_infinite, initial_witness)?;
    Ok(())
}

pub(super) fn embedded_curve_add<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &mut WitnessMap<F>,
    input1: [FunctionInput<F>; 3],
    input2: [FunctionInput<F>; 3],
    outputs: (Witness, Witness, Witness),
) -> Result<(), OpcodeResolutionError<F>> {
    let input1_x = input_to_value(initial_witness, input1[0], false)?;
    let input1_y = input_to_value(initial_witness, input1[1], false)?;
    let input1_infinite = input_to_value(initial_witness, input1[2], false)?;
    let input2_x = input_to_value(initial_witness, input2[0], false)?;
    let input2_y = input_to_value(initial_witness, input2[1], false)?;
    let input2_infinite = input_to_value(initial_witness, input2[2], false)?;
    let (res_x, res_y, res_infinite) = backend.ec_add(
        &input1_x,
        &input1_y,
        &input1_infinite,
        &input2_x,
        &input2_y,
        &input2_infinite,
    )?;

    insert_value(&outputs.0, res_x, initial_witness)?;
    insert_value(&outputs.1, res_y, initial_witness)?;
    insert_value(&outputs.2, res_infinite, initial_witness)?;
    Ok(())
}
