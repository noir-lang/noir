use acvm::BlackBoxFunctionSolver;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use iter_extended::try_vecmap;
use noirc_errors::Location;

use crate::{
    hir::comptime::{errors::IResult, interpreter::builtin::get_field, InterpreterError, Value},
    macros_api::NodeInterner,
};

use super::builtin::{check_argument_count, get_array, get_u32};

pub(super) fn call_foreign(
    interner: &mut NodeInterner,
    name: &str,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    match name {
        "poseidon2_permutation" => poseidon2_permutation(interner, arguments, location),
        _ => {
            let item = format!("Comptime evaluation for builtin function {name}");
            Err(InterpreterError::Unimplemented { item, location })
        }
    }
}

// poseidon2_permutation<let N: u32>(_input: [Field; N], _state_length: u32) -> [Field; N]
fn poseidon2_permutation(
    interner: &mut NodeInterner,
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(2, &arguments, location)?;

    let state_length = get_u32(arguments.pop().unwrap().0, location)?;
    let (input, typ) = get_array(interner, arguments.pop().unwrap().0, location)?;

    let input = try_vecmap(input, |integer| get_field(integer, location))?;

    // Currently locked to only bn254!
    let fields = Bn254BlackBoxSolver
        .poseidon2_permutation(&input, state_length)
        .map_err(|error| InterpreterError::BlackBoxError(error, location))?;

    let array = fields.into_iter().map(Value::Field).collect();
    Ok(Value::Array(array, typ))
}
