
use acvm::{AcirField, FieldElement, BlackBoxFunctionSolver};
use noirc_errors::Location;

use crate::{
    hir::comptime::{errors::IResult, InterpreterError, Value},
    macros_api::NodeInterner,
};

use super::builtin::{check_argument_count, get_u32, get_array};

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
fn posdon2_permutation(
    interner: &mut NodeInterner,
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(2, &arguments, location)?;

    let state_length = get_u32(arguments.pop().unwrap().0, location);
    let input = get_array(interner, arguments.pop().unwrap().0, location);

    let result = Bn254BlackBoxSolver.poseidon2_permutation(inputs, state_length);
}
