use acir::{
    AcirField,
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
};
use acvm_blackbox_solver::BlackBoxFunctionSolver;

use crate::pwg::{OpcodeResolutionError, input_to_value, insert_value};

pub(super) fn multi_scalar_mul<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &mut WitnessMap<F>,
    points: &[FunctionInput<F>],
    scalars: &[FunctionInput<F>],
    predicate: FunctionInput<F>,
    outputs: (Witness, Witness, Witness),
) -> Result<(), OpcodeResolutionError<F>> {
    let (res_x, res_y, is_infinite) =
        execute_multi_scalar_mul(backend, initial_witness, points, scalars, predicate)?;

    // Insert the resulting point into the witness map
    insert_value(&outputs.0, res_x, initial_witness)?;
    insert_value(&outputs.1, res_y, initial_witness)?;
    insert_value(&outputs.2, is_infinite, initial_witness)?;
    Ok(())
}

pub(crate) fn execute_multi_scalar_mul<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &WitnessMap<F>,
    points: &[FunctionInput<F>],
    scalars: &[FunctionInput<F>],
    predicate: FunctionInput<F>,
) -> Result<(F, F, F), OpcodeResolutionError<F>> {
    validate_multi_scalar_mul_inputs(points, scalars)?;

    let points: Result<Vec<_>, _> =
        points.iter().map(|input| input_to_value(initial_witness, *input)).collect();
    let points: Vec<_> = points?.into_iter().collect();

    let scalars: Result<Vec<_>, _> =
        scalars.iter().map(|input| input_to_value(initial_witness, *input)).collect();

    let predicate = input_to_value(initial_witness, predicate)?.is_one();

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
        backend.multi_scalar_mul(&points, &scalars_lo, &scalars_hi, predicate)?;
    Ok((res_x, res_y, is_infinite))
}

fn validate_multi_scalar_mul_inputs<F: AcirField>(
    points: &[FunctionInput<F>],
    scalars: &[FunctionInput<F>],
) -> Result<(), OpcodeResolutionError<F>> {
    if !scalars.len().is_multiple_of(2) {
        return Err(OpcodeResolutionError::BlackBoxFunctionFailed(
            acir::BlackBoxFunc::MultiScalarMul,
            format!("expected an even number of scalar limbs, got {}", scalars.len()),
        ));
    }

    if !points.len().is_multiple_of(3) {
        return Err(OpcodeResolutionError::BlackBoxFunctionFailed(
            acir::BlackBoxFunc::MultiScalarMul,
            format!(
                "expected point inputs in triples of (x, y, is_infinite), got {} values",
                points.len()
            ),
        ));
    }

    let point_count = points.len() / 3;
    let scalar_count = scalars.len() / 2;
    if scalar_count != point_count {
        return Err(OpcodeResolutionError::BlackBoxFunctionFailed(
            acir::BlackBoxFunc::MultiScalarMul,
            format!(
                "expected the same number of points and scalars, got {point_count} points and {scalar_count} scalars"
            ),
        ));
    }

    Ok(())
}

pub(super) fn embedded_curve_add<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &mut WitnessMap<F>,
    input1: [FunctionInput<F>; 3],
    input2: [FunctionInput<F>; 3],
    predicate: FunctionInput<F>,
    outputs: (Witness, Witness, Witness),
) -> Result<(), OpcodeResolutionError<F>> {
    let (res_x, res_y, res_infinite) =
        execute_embedded_curve_add(backend, initial_witness, input1, input2, predicate)?;

    insert_value(&outputs.0, res_x, initial_witness)?;
    insert_value(&outputs.1, res_y, initial_witness)?;
    insert_value(&outputs.2, res_infinite, initial_witness)?;
    Ok(())
}

pub(crate) fn execute_embedded_curve_add<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &WitnessMap<F>,
    input1: [FunctionInput<F>; 3],
    input2: [FunctionInput<F>; 3],
    predicate: FunctionInput<F>,
) -> Result<(F, F, F), OpcodeResolutionError<F>> {
    let input1_x = input_to_value(initial_witness, input1[0])?;
    let input1_y = input_to_value(initial_witness, input1[1])?;
    let input1_infinite = input_to_value(initial_witness, input1[2])?;
    let input2_x = input_to_value(initial_witness, input2[0])?;
    let input2_y = input_to_value(initial_witness, input2[1])?;
    let input2_infinite = input_to_value(initial_witness, input2[2])?;
    let predicate = input_to_value(initial_witness, predicate)?.is_one();
    let (res_x, res_y, res_infinite) = backend.ec_add(
        &input1_x,
        &input1_y,
        &input1_infinite,
        &input2_x,
        &input2_y,
        &input2_infinite,
        predicate,
    )?;

    Ok((res_x, res_y, res_infinite))
}
