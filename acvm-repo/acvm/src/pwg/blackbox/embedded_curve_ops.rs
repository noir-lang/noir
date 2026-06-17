use acir::{
    AcirField, BlackBoxFunc,
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
    outputs: (Witness, Witness),
) -> Result<(), OpcodeResolutionError<F>> {
    let (res_x, res_y) =
        execute_multi_scalar_mul(backend, initial_witness, points, scalars, predicate)?;

    // Insert the resulting point into the witness map
    insert_value(&outputs.0, res_x, initial_witness)?;
    insert_value(&outputs.1, res_y, initial_witness)?;
    Ok(())
}

pub(crate) fn execute_multi_scalar_mul<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &WitnessMap<F>,
    points: &[FunctionInput<F>],
    scalars: &[FunctionInput<F>],
    predicate: FunctionInput<F>,
) -> Result<(F, F), OpcodeResolutionError<F>> {
    assert!(scalars.len().is_multiple_of(2), "Number of scalars must be even");
    assert!(points.len().is_multiple_of(2), "Number of points must be a multiple of 2");
    assert_eq!(
        scalars.len() / 2,
        points.len() / 2,
        "Number of scalars must be the same as the number of points"
    );

    for point in points.chunks(2) {
        check_all_or_nothing_coordinates(BlackBoxFunc::MultiScalarMul, point[0], point[1])?;
    }

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
    let (res_x, res_y) = backend.multi_scalar_mul(&points, &scalars_lo, &scalars_hi, predicate)?;
    Ok((res_x, res_y))
}

pub(super) fn embedded_curve_add<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &mut WitnessMap<F>,
    input1: [FunctionInput<F>; 2],
    input2: [FunctionInput<F>; 2],
    predicate: FunctionInput<F>,
    outputs: (Witness, Witness),
) -> Result<(), OpcodeResolutionError<F>> {
    let (res_x, res_y) =
        execute_embedded_curve_add(backend, initial_witness, input1, input2, predicate)?;

    insert_value(&outputs.0, res_x, initial_witness)?;
    insert_value(&outputs.1, res_y, initial_witness)?;
    Ok(())
}

pub(crate) fn execute_embedded_curve_add<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &WitnessMap<F>,
    input1: [FunctionInput<F>; 2],
    input2: [FunctionInput<F>; 2],
    predicate: FunctionInput<F>,
) -> Result<(F, F), OpcodeResolutionError<F>> {
    check_all_or_nothing_coordinates(BlackBoxFunc::EmbeddedCurveAdd, input1[0], input1[1])?;
    check_all_or_nothing_coordinates(BlackBoxFunc::EmbeddedCurveAdd, input2[0], input2[1])?;

    let input1_x = input_to_value(initial_witness, input1[0])?;
    let input1_y = input_to_value(initial_witness, input1[1])?;
    let input2_x = input_to_value(initial_witness, input2[0])?;
    let input2_y = input_to_value(initial_witness, input2[1])?;
    let predicate = input_to_value(initial_witness, predicate)?.is_one();
    let (res_x, res_y) = backend.ec_add(&input1_x, &input1_y, &input2_x, &input2_y, predicate)?;

    Ok((res_x, res_y))
}

/// Checks that x and y are either both witnesses or both constants, erroring otherwise.
fn check_all_or_nothing_coordinates<F: AcirField>(
    func: BlackBoxFunc,
    x: FunctionInput<F>,
    y: FunctionInput<F>,
) -> Result<(), OpcodeResolutionError<F>> {
    match (x, y) {
        (FunctionInput::Witness(_), FunctionInput::Witness(_))
        | (FunctionInput::Constant(_), FunctionInput::Constant(_)) => Ok(()),
        _ => Err(OpcodeResolutionError::BlackBoxFunctionFailed(
            func,
            format!(
                "Coordinates must be either both witnesses or both constants. Found: {x:?}, {y:?}"
            ),
        )),
    }
}
