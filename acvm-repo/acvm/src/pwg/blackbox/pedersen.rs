use acir::{
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
    AcirField,
};

use crate::{
    pwg::{input_to_value, insert_value, OpcodeResolutionError},
    BlackBoxFunctionSolver,
};

pub(super) fn pedersen<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &mut WitnessMap<F>,
    inputs: &[FunctionInput<F>],
    domain_separator: u32,
    outputs: (Witness, Witness),
) -> Result<(), OpcodeResolutionError<F>> {
    let scalars: Result<Vec<_>, _> =
        inputs.iter().map(|input| input_to_value(initial_witness, *input)).collect();
    let scalars: Vec<_> = scalars?.into_iter().collect();

    let (res_x, res_y) = backend.pedersen_commitment(&scalars, domain_separator)?;

    insert_value(&outputs.0, res_x, initial_witness)?;
    insert_value(&outputs.1, res_y, initial_witness)?;

    Ok(())
}

pub(super) fn pedersen_hash<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    initial_witness: &mut WitnessMap<F>,
    inputs: &[FunctionInput<F>],
    domain_separator: u32,
    output: Witness,
) -> Result<(), OpcodeResolutionError<F>> {
    let scalars: Result<Vec<_>, _> =
        inputs.iter().map(|input| input_to_value(initial_witness, *input)).collect();
    let scalars: Vec<_> = scalars?.into_iter().collect();

    let res = backend.pedersen_hash(&scalars, domain_separator)?;

    insert_value(&output, res, initial_witness)?;

    Ok(())
}
