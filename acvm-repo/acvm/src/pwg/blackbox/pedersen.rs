use acir::{
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
};

use crate::{
    pwg::{insert_value, witness_to_value, OpcodeResolutionError},
    BlackBoxFunctionSolver,
};

pub(super) fn pedersen(
    backend: &impl BlackBoxFunctionSolver,
    initial_witness: &mut WitnessMap,
    inputs: &[FunctionInput],
    domain_separator: u32,
    outputs: (Witness, Witness),
) -> Result<(), OpcodeResolutionError> {
    let scalars: Result<Vec<_>, _> =
        inputs.iter().map(|input| witness_to_value(initial_witness, input.witness)).collect();
    let scalars: Vec<_> = scalars?.into_iter().cloned().collect();

    let (res_x, res_y) = backend.pedersen(&scalars, domain_separator)?;

    insert_value(&outputs.0, res_x, initial_witness)?;
    insert_value(&outputs.1, res_y, initial_witness)?;

    Ok(())
}
