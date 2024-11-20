use acvm::{
    acir::BlackBoxFunc,
    blackbox_solver::{BigintSolverWithId, BlackBoxFunctionSolver},
    AcirField, BlackBoxResolutionError,
};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use im::Vector;
use noirc_errors::Location;

use crate::{
    hir::comptime::{
        errors::IResult, interpreter::builtin::builtin_helpers::to_byte_array, InterpreterError,
        Value,
    },
    node_interner::NodeInterner,
};

use super::{
    builtin::builtin_helpers::{
        check_one_argument, check_three_arguments, check_two_arguments, get_array_map, get_field,
        get_fixed_array_map, get_slice_map, get_u32, get_u64, get_u8, to_byte_slice,
    },
    Interpreter,
};

impl<'local, 'context> Interpreter<'local, 'context> {
    pub(super) fn call_foreign(
        &mut self,
        name: &str,
        arguments: Vec<(Value, Location)>,
        location: Location,
    ) -> IResult<Value> {
        call_foreign(self.elaborator.interner, &mut self.bigint_solver, name, arguments, location)
    }
}

// Similar to `evaluate_black_box` in `brillig_vm`.
fn call_foreign(
    interner: &mut NodeInterner,
    bigint_solver: &mut BigintSolverWithId,
    name: &str,
    args: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    use BlackBoxFunc::*;
    match name {
        "aes128_encrypt" => aes128_encrypt(interner, args, location),
        "bigint_from_le_bytes" => bigint_from_le_bytes(interner, bigint_solver, args, location),
        "bigint_to_le_bytes" => bigint_to_le_bytes(bigint_solver, args, location),
        "bigint_add" => bigint_op(bigint_solver, BigIntAdd, args, location),
        "bigint_sub" => bigint_op(bigint_solver, BigIntSub, args, location),
        "bigint_mul" => bigint_op(bigint_solver, BigIntMul, args, location),
        "bigint_div" => bigint_op(bigint_solver, BigIntDiv, args, location),
        "blake2s" => blake2s(interner, args, location),
        "poseidon2_permutation" => poseidon2_permutation(interner, args, location),
        "keccakf1600" => keccakf1600(interner, args, location),
        "range" => apply_range_constraint(args, location),
        _ => {
            let item = format!("Comptime evaluation for foreign function '{name}'");
            Err(InterpreterError::Unimplemented { item, location })
        }
    }
}

/// `pub fn aes128_encrypt<let N: u32>(input: [u8; N], iv: [u8; 16], key: [u8; 16]) -> [u8]`
fn aes128_encrypt(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (inputs, iv, key) = check_three_arguments(arguments, location)?;

    let (inputs, _) = get_array_map(interner, inputs, get_u8)?;
    let iv = get_fixed_array_map(interner, iv, get_u8)?;
    let key = get_fixed_array_map(interner, key, get_u8)?;

    let output = acvm_blackbox_solver::aes128_encrypt(&inputs, iv, key)
        .map_err(|e| InterpreterError::BlackBoxError(e, location))?;

    Ok(to_byte_slice(&output))
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

/// `fn from_le_bytes(bytes: [u8], modulus: [u8]) -> BigInt`
///
/// Returns the ID of the new bigint allocated by the solver.
fn bigint_from_le_bytes(
    interner: &mut NodeInterner,
    solver: &mut BigintSolverWithId,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (bytes, modulus) = check_two_arguments(arguments, location)?;

    let (bytes, _) = get_slice_map(interner, bytes, get_u8)?;
    let (modulus, _) = get_slice_map(interner, modulus, get_u8)?;

    let id = solver
        .bigint_from_bytes(&bytes, &modulus)
        .map_err(|e| InterpreterError::BlackBoxError(e, location))?;

    Ok(Value::U32(id))
}

/// `fn to_le_bytes(self) -> [u8; 32]`
///
/// Take the ID of a bigint and returned its content.
fn bigint_to_le_bytes(
    solver: &mut BigintSolverWithId,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let id = check_one_argument(arguments, location)?;
    let id = get_u32(id)?;

    let mut bytes =
        solver.bigint_to_bytes(id).map_err(|e| InterpreterError::BlackBoxError(e, location))?;

    assert!(bytes.len() <= 32);
    bytes.resize(32, 0);

    Ok(to_byte_array(&bytes))
}

/// `fn bigint_add(self, other: BigInt) -> BigInt`
///
/// Takes two previous allocated IDs, gets the values from the solver,
/// stores the result of the operation, returns the new ID.
fn bigint_op(
    solver: &mut BigintSolverWithId,
    func: BlackBoxFunc,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (lhs, rhs) = check_two_arguments(arguments, location)?;

    let lhs = get_u32(lhs)?;
    let rhs = get_u32(rhs)?;

    let id = solver
        .bigint_op(lhs, rhs, func)
        .map_err(|e| InterpreterError::BlackBoxError(e, location))?;

    Ok(Value::U32(id))
}

/// `pub fn blake2s<let N: u32>(input: [u8; N]) -> [u8; 32]`
fn blake2s(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let inputs = check_one_argument(arguments, location)?;

    let (inputs, _) = get_array_map(interner, inputs, get_u8)?;
    let output = acvm_blackbox_solver::blake2s(&inputs)
        .map_err(|e| InterpreterError::BlackBoxError(e, location))?;

    Ok(to_byte_array(&output))
}

/// `poseidon2_permutation<let N: u32>(_input: [Field; N], _state_length: u32) -> [Field; N]`
fn poseidon2_permutation(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (input, state_length) = check_two_arguments(arguments, location)?;

    let (input, typ) = get_array_map(interner, input, get_field)?;
    let state_length = get_u32(state_length)?;

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

    let (input, typ) = get_array_map(interner, input, get_u64)?;

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
    use im::HashSet;
    use noirc_errors::Location;
    use strum::IntoEnumIterator;

    use crate::hir::comptime::tests::with_interpreter;
    use crate::hir::comptime::InterpreterError::{ArgumentCountMismatch, Unimplemented};

    use super::call_foreign;

    /// Check that all `BlackBoxFunc` are covered by `call_foreign`.
    #[test]
    fn test_blackbox_implemented() {
        let dummy = "
        comptime fn main() -> pub u8 {
            0
        }
        ";

        let skip: HashSet<BlackBoxFunc> = HashSet::from_iter([
            // Schnorr to be removed
            BlackBoxFunc::SchnorrVerify,
            // `acvm::blackbox_solver::bit_xor` exists for field elements,
            // but the compiler says XOR on `Field` is not supported, only on ints,
            // and the the SSA to Brillig transformation also expects these to be
            // handled as a BinaryOp, which the Interpreter implements for ints.
            BlackBoxFunc::XOR,
            BlackBoxFunc::AND,
        ]);

        let not_implemented = with_interpreter(dummy, |interpreter, _, _| {
            let no_location = Location::dummy();
            let mut not_implemented = Vec::new();

            for blackbox in BlackBoxFunc::iter().filter(|func| !skip.contains(func)) {
                let name = blackbox.name();
                match call_foreign(
                    interpreter.elaborator.interner,
                    &mut interpreter.bigint_solver,
                    name,
                    Vec::new(),
                    no_location,
                ) {
                    Ok(_) => {
                        // Exists and works with no args (unlikely)
                    }
                    Err(ArgumentCountMismatch { .. }) => {
                        // Exists but doesn't work with no args (expected)
                    }
                    Err(Unimplemented { .. }) => not_implemented.push(name),
                    Err(other) => panic!("unexpected error: {other:?}"),
                };
            }

            not_implemented
        });

        assert!(
            not_implemented.is_empty(),
            "unimplemented blackbox functions: {not_implemented:?}"
        );
    }
}
