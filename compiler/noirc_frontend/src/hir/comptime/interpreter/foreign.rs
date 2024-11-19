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
        "range" => apply_range_constraint(arguments, location),
        _ => {
            let item = format!("Comptime evaluation for builtin function {name}");
            Err(InterpreterError::Unimplemented { item, location })
        }
    }
}

fn apply_range_constraint(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
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

#[cfg(test)]
mod tests {
    use acvm::acir::BlackBoxFunc;
    use noirc_errors::Location;
    use strum::IntoEnumIterator;

    use crate::hir::comptime::tests::with_interpreter;
    use crate::hir::comptime::InterpreterError::{ArgumentCountMismatch, Unimplemented};

    use super::call_foreign;

    #[test]
    fn test_blackbox_implemented() {
        let dummy = "
        comptime fn main() -> pub u8 {
            0
        }
        ";
        let not_implemented = with_interpreter(&dummy, |interpreter, _, _| {
            let no_location = Location::dummy();
            let mut not_implemented = Vec::new();

            for blackbox in BlackBoxFunc::iter() {
                let name = blackbox.name();
                match call_foreign(interpreter.elaborator.interner, name, Vec::new(), no_location) {
                    Ok(_) => {
                        // Exists and works with no args
                    }
                    Err(ArgumentCountMismatch { .. }) => {
                        // Exists but doesn't work with no args
                    }
                    Err(Unimplemented { .. }) => not_implemented.push(name),
                    Err(other) => panic!("unexpected error: {other:?}"),
                };
            }

            not_implemented
        });

        assert_eq!(
            not_implemented.len(),
            0,
            "unimplemented blackbox functions: {not_implemented:?}"
        );
    }
}
