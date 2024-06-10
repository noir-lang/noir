use acir::{
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
    AcirField,
};
use acvm_blackbox_solver::BlackBoxFunctionSolver;

use crate::pwg::{insert_value, witness_to_value, OpcodeResolutionError};

pub(super) fn multi_scalar_mul<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &mut WitnessMap<F>,
    points: &[FunctionInput],
    scalars: &[FunctionInput],
    outputs: (Witness, Witness, Witness),
) -> Result<(), OpcodeResolutionError<F>> {
    let points: Result<Vec<_>, _> =
        points.iter().map(|input| witness_to_value(initial_witness, input.witness)).collect();
    let points: Vec<_> = points?.into_iter().cloned().collect();

    let scalars: Result<Vec<_>, _> =
        scalars.iter().map(|input| witness_to_value(initial_witness, input.witness)).collect();
    let mut scalars_lo = Vec::new();
    let mut scalars_hi = Vec::new();
    for (i, scalar) in scalars?.into_iter().enumerate() {
        if i % 2 == 0 {
            scalars_lo.push(*scalar);
        } else {
            scalars_hi.push(*scalar);
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
    input1: [FunctionInput; 3],
    input2: [FunctionInput; 3],
    outputs: (Witness, Witness, Witness),
) -> Result<(), OpcodeResolutionError<F>> {
    let input1_x = witness_to_value(initial_witness, input1[0].witness)?;
    let input1_y = witness_to_value(initial_witness, input1[1].witness)?;
    let input1_infinite = witness_to_value(initial_witness, input1[2].witness)?;
    let input2_x = witness_to_value(initial_witness, input2[0].witness)?;
    let input2_y = witness_to_value(initial_witness, input2[1].witness)?;
    let input2_infinite = witness_to_value(initial_witness, input2[2].witness)?;
    let (res_x, res_y, res_infinite) =
        backend.ec_add(input1_x, input1_y, input1_infinite, input2_x, input2_y, input2_infinite)?;

    insert_value(&outputs.0, res_x, initial_witness)?;
    insert_value(&outputs.1, res_y, initial_witness)?;
    insert_value(&outputs.2, res_infinite, initial_witness)?;
    Ok(())
}
