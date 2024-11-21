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
    Type,
};

use super::{
    builtin::builtin_helpers::{
        check_arguments, check_one_argument, check_three_arguments, check_two_arguments,
        get_array_map, get_field, get_fixed_array_map, get_slice_map, get_u32, get_u64, get_u8,
        to_byte_slice,
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
        "blake2s" => blake_hash(interner, args, location, acvm::blackbox_solver::blake2s),
        "blake3" => blake_hash(interner, args, location, acvm::blackbox_solver::blake3),
        "ecdsa_secp256k1" => ecdsa_secp256_verify(
            interner,
            args,
            location,
            acvm::blackbox_solver::ecdsa_secp256k1_verify,
        ),
        "ecdsa_secp256r1" => ecdsa_secp256_verify(
            interner,
            args,
            location,
            acvm::blackbox_solver::ecdsa_secp256r1_verify,
        ),
        "poseidon2_permutation" => poseidon2_permutation(interner, args, location),
        "keccakf1600" => keccakf1600(interner, args, location),
        "range" => apply_range_constraint(args, location),
        "sha256_compression" => sha256_compression(interner, args, location),
        _ => {
            let item = format!("Comptime evaluation for foreign function '{name}'");
            let explanation = match name {
                "schnorr_verify" => Some("Schnorr verification will be removed."),
                "and" | "xor" => Some("It should be turned a binary operation instead."),
                "recursive_aggregation" => Some("A proof cannot be verified at comptime."),
                _ => None,
            };
            if let Some(e) = explanation {
                Err(InterpreterError::WillNotImplement {
                    item,
                    location,
                    explanation: e.to_string(),
                })
            } else {
                Err(InterpreterError::Unimplemented { item, location })
            }
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
    let (iv, _) = get_fixed_array_map(interner, iv, get_u8)?;
    let (key, _) = get_fixed_array_map(interner, key, get_u8)?;

    let output = acvm::blackbox_solver::aes128_encrypt(&inputs, iv, key)
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

/// Run one of the Blake hash functions.
/// ```text
/// pub fn blake2s<let N: u32>(input: [u8; N]) -> [u8; 32]
/// pub fn blake3<let N: u32>(input: [u8; N]) -> [u8; 32]
/// ```
fn blake_hash(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
    f: impl Fn(&[u8]) -> Result<[u8; 32], BlackBoxResolutionError>,
) -> IResult<Value> {
    let inputs = check_one_argument(arguments, location)?;

    let (inputs, _) = get_array_map(interner, inputs, get_u8)?;
    let output = f(&inputs).map_err(|e| InterpreterError::BlackBoxError(e, location))?;

    Ok(to_byte_array(&output))
}

/// Run one of the Secp256 signature verifications.
/// ```text
/// pub fn verify_signature<let N: u32>(
///   public_key_x: [u8; 32],
///   public_key_y: [u8; 32],
///   signature: [u8; 64],
///   message_hash: [u8; N],
/// ) -> bool

/// pub fn verify_signature_slice(
///   public_key_x: [u8; 32],
///   public_key_y: [u8; 32],
///   signature: [u8; 64],
///   message_hash: [u8],
/// ) -> bool
/// ```
fn ecdsa_secp256_verify(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
    f: impl Fn(&[u8], &[u8; 32], &[u8; 32], &[u8; 64]) -> Result<bool, BlackBoxResolutionError>,
) -> IResult<Value> {
    let [pub_key_x, pub_key_y, sig, msg_hash] = check_arguments(arguments, location)?;

    let (pub_key_x, _) = get_fixed_array_map(interner, pub_key_x, get_u8)?;
    let (pub_key_y, _) = get_fixed_array_map(interner, pub_key_y, get_u8)?;
    let (sig, _) = get_fixed_array_map(interner, sig, get_u8)?;

    // Hash can be an array or slice.
    let (msg_hash, _) = if matches!(msg_hash.0.get_type().as_ref(), Type::Array(_, _)) {
        get_array_map(interner, msg_hash.clone(), get_u8)?
    } else {
        get_slice_map(interner, msg_hash, get_u8)?
    };

    let is_valid = f(&msg_hash, &pub_key_x, &pub_key_y, &sig)
        .map_err(|e| InterpreterError::BlackBoxError(e, location))?;

    Ok(Value::Bool(is_valid))
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

/// `fn keccakf1600(input: [u64; 25]) -> [u64; 25] {}`
fn keccakf1600(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let input = check_one_argument(arguments, location)?;

    let (state, typ) = get_fixed_array_map(interner, input, get_u64)?;

    let result_lanes = acvm::blackbox_solver::keccakf1600(state)
        .map_err(|error| InterpreterError::BlackBoxError(error, location))?;

    let array: Vector<Value> = result_lanes.into_iter().map(Value::U64).collect();
    Ok(Value::Array(array, typ))
}

/// `pub fn sha256_compression(input: [u32; 16], state: [u32; 8]) -> [u32; 8]`
fn sha256_compression(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (input, state) = check_two_arguments(arguments, location)?;

    let (input, _) = get_fixed_array_map(interner, input, get_u32)?;
    let (mut state, typ) = get_fixed_array_map(interner, state, get_u32)?;

    acvm::blackbox_solver::sha256_compression(&mut state, &input);

    let state = state.into_iter().map(Value::U32).collect();
    Ok(Value::Array(state, typ))
}

#[cfg(test)]
mod tests {
    use acvm::acir::BlackBoxFunc;
    use noirc_errors::Location;
    use strum::IntoEnumIterator;

    use crate::hir::comptime::tests::with_interpreter;
    use crate::hir::comptime::InterpreterError::{
        ArgumentCountMismatch, Unimplemented, WillNotImplement,
    };

    use super::call_foreign;

    /// Check that all `BlackBoxFunc` are covered by `call_foreign`.
    #[test]
    fn test_blackbox_implemented() {
        let dummy = "
        comptime fn main() -> pub u8 {
            0
        }
        ";

        let not_implemented = with_interpreter(dummy, |interpreter, _, _| {
            let no_location = Location::dummy();
            let mut not_implemented = Vec::new();

            for blackbox in BlackBoxFunc::iter() {
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
                    Err(WillNotImplement { .. }) => {}
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
