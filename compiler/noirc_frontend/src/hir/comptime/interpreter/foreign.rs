use acvm::{
    acir::BlackBoxFunc, blackbox_solver::BlackBoxFunctionSolver, AcirField, BlackBoxResolutionError,
};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use im::Vector;
use iter_extended::try_vecmap;
use noirc_errors::Location;

use crate::{
    hir::comptime::{errors::IResult, InterpreterError, Value},
    node_interner::NodeInterner,
};

use super::builtin::builtin_helpers::{
    check_one_argument, check_two_arguments, get_array, get_field, get_u32, get_u64,
};

pub(super) fn call_foreign(
    interner: &mut NodeInterner,
    name: &str,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    match name {
        "poseidon2_permutation" => poseidon2_permutation(interner, arguments, location),
        "keccakf1600" => keccakf1600(interner, arguments, location),
        _ => {
            let item = format!("Comptime evaluation for builtin function {name}");
            Err(InterpreterError::Unimplemented { item, location })
        }
    }
}

pub(super) fn apply_range_constraint(
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (value, num_bits) = check_two_arguments(arguments, location)?;

    let input = get_field(value)?;
    let num_bits = get_u32(num_bits)?;

    if input.num_bits() < num_bits {
        Ok(Value::Unit)
    } else {
        Err(InterpreterError::BlackBoxError(
            BlackBoxResolutionError::Failed(
                BlackBoxFunc::RANGE,
                "value exceeds range check bounds".to_owned(),
            ),
            location,
        ))
    }
}

// poseidon2_permutation<let N: u32>(_input: [Field; N], _state_length: u32) -> [Field; N]
fn poseidon2_permutation(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (input, state_length) = check_two_arguments(arguments, location)?;
    let input_location = input.1;

    let (input, typ) = get_array(interner, input)?;
    let state_length = get_u32(state_length)?;

    let input = try_vecmap(input, |integer| get_field((integer, input_location)))?;

    // Currently locked to only bn254!
    let fields = Bn254BlackBoxSolver
        .poseidon2_permutation(&input, state_length)
        .map_err(|error| InterpreterError::BlackBoxError(error, location))?;

    let array = fields.into_iter().map(Value::Field).collect();
    Ok(Value::Array(array, typ))
}

fn keccakf1600(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let input = check_one_argument(arguments, location)?;
    let input_location = input.1;

    let (input, typ) = get_array(interner, input)?;

    let input = try_vecmap(input, |integer| get_u64((integer, input_location)))?;

    let mut state = [0u64; 25];
    for (it, input_value) in state.iter_mut().zip(input.iter()) {
        *it = *input_value;
    }
    let result_lanes = acvm::blackbox_solver::keccakf1600(state)
        .map_err(|error| InterpreterError::BlackBoxError(error, location))?;

    let array: Vector<Value> = result_lanes.into_iter().map(Value::U64).collect();
    Ok(Value::Array(array, typ))
}
