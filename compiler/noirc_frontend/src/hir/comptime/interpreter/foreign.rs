//! The foreign function counterpart to `interpreter/builtin.rs`, defines how to call
//! all foreign functions available to the interpreter.
use acvm::{BlackBoxResolutionError, FieldElement, blackbox_solver::BlackBoxFunctionSolver};
use bn254_blackbox_solver::Bn254BlackBoxSolver; // Currently locked to only bn254!
use im::{Vector, vector};
use iter_extended::vecmap;
use noirc_errors::Location;

use crate::{
    Type,
    hir::comptime::{
        InterpreterError, Value, errors::IResult,
        interpreter::builtin::builtin_helpers::to_byte_array,
    },
    signed_field::SignedField,
};

use super::{
    Interpreter,
    builtin::builtin_helpers::{
        check_arguments, check_one_argument, check_three_arguments, check_two_arguments,
        get_array_map, get_bool, get_field, get_fixed_array_map, get_struct_field,
        get_struct_fields, get_u8, get_u32, get_u64, to_byte_slice, to_struct,
    },
};

impl Interpreter<'_, '_> {
    pub(super) fn call_foreign(
        &mut self,
        name: &str,
        arguments: Vec<(Value, Location)>,
        return_type: Type,
        location: Location,
    ) -> IResult<Value> {
        call_foreign(name, arguments, return_type, location, self.elaborator.pedantic_solving())
    }
}

/// Calls the given foreign function.
///
/// Similar to `evaluate_black_box` in `brillig_vm`.
fn call_foreign(
    name: &str,
    args: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
    pedantic_solving: bool,
) -> IResult<Value> {
    match name {
        "aes128_encrypt" => aes128_encrypt(args, location),
        "blake2s" => blake_hash(args, location, acvm::blackbox_solver::blake2s),
        "blake3" => blake_hash(args, location, acvm::blackbox_solver::blake3),
        // cSpell:disable-next-line
        "ecdsa_secp256k1" => {
            ecdsa_secp256_verify(args, location, acvm::blackbox_solver::ecdsa_secp256k1_verify)
        }
        // cSpell:disable-next-line
        "ecdsa_secp256r1" => {
            ecdsa_secp256_verify(args, location, acvm::blackbox_solver::ecdsa_secp256r1_verify)
        }
        "embedded_curve_add" => embedded_curve_add(args, return_type, location, pedantic_solving),
        "multi_scalar_mul" => multi_scalar_mul(args, return_type, location, pedantic_solving),
        "poseidon2_permutation" => poseidon2_permutation(args, location, pedantic_solving),
        "keccakf1600" => keccakf1600(args, location),
        "sha256_compression" => sha256_compression(args, location),
        _ => {
            let explanation = match name {
                "and" | "xor" => "It should be turned into a binary operation.".into(),
                "recursive_aggregation" => "A proof cannot be verified at comptime.".into(),
                _ => {
                    let item = format!("Comptime evaluation for foreign function '{name}'");
                    return Err(InterpreterError::Unimplemented { item, location });
                }
            };

            let item = format!("Attempting to evaluate foreign function '{name}'");
            Err(InterpreterError::InvalidInComptimeContext { item, location, explanation })
        }
    }
}

/// `pub fn aes128_encrypt<let N: u32>(input: [u8; N], iv: [u8; 16], key: [u8; 16]) -> [u8]`
fn aes128_encrypt(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    let (inputs, iv, key) = check_three_arguments(arguments, location)?;

    let (inputs, _) = get_array_map(inputs, get_u8)?;
    let (iv, _) = get_fixed_array_map(iv, get_u8)?;
    let (key, _) = get_fixed_array_map(key, get_u8)?;

    let output = acvm::blackbox_solver::aes128_encrypt(&inputs, iv, key)
        .map_err(|e| InterpreterError::BlackBoxError(e, location))?;

    Ok(to_byte_slice(&output))
}

/// Run one of the Blake hash functions.
/// ```text
/// pub fn blake2s<let N: u32>(input: [u8; N]) -> [u8; 32]
/// pub fn blake3<let N: u32>(input: [u8; N]) -> [u8; 32]
/// ```
fn blake_hash(
    arguments: Vec<(Value, Location)>,
    location: Location,
    f: impl Fn(&[u8]) -> Result<[u8; 32], BlackBoxResolutionError>,
) -> IResult<Value> {
    let inputs = check_one_argument(arguments, location)?;

    let (inputs, _) = get_array_map(inputs, get_u8)?;
    let output = f(&inputs).map_err(|e| InterpreterError::BlackBoxError(e, location))?;

    Ok(to_byte_array(&output))
}

// cSpell:disable-next-line
/// Run one of the Secp256 signature verifications.
/// ```text
/// pub fn _verify_signature(
///     public_key_x: [u8; 32],
///     public_key_y: [u8; 32],
///     signature: [u8; 64],
///     message_hash: [u8; 32],
///     predicate: bool,
/// ) -> bool
// cSpell:disable-next-line
fn ecdsa_secp256_verify(
    arguments: Vec<(Value, Location)>,
    location: Location,
    f: impl Fn(&[u8; 32], &[u8; 32], &[u8; 32], &[u8; 64]) -> Result<bool, BlackBoxResolutionError>,
) -> IResult<Value> {
    let [pub_key_x, pub_key_y, sig, msg_hash, predicate] = check_arguments(arguments, location)?;
    assert_eq!(predicate.0, Value::Bool(true), "verify_signature predicate should be true");

    let (pub_key_x, _) = get_fixed_array_map(pub_key_x, get_u8)?;
    let (pub_key_y, _) = get_fixed_array_map(pub_key_y, get_u8)?;
    let (sig, _) = get_fixed_array_map(sig, get_u8)?;
    let (msg_hash, _) = get_fixed_array_map(msg_hash.clone(), get_u8)?;

    let is_valid = f(&msg_hash, &pub_key_x, &pub_key_y, &sig)
        .map_err(|e| InterpreterError::BlackBoxError(e, location))?;

    Ok(Value::Bool(is_valid))
}

/// ```text
/// fn embedded_curve_add(
///     point1: EmbeddedCurvePoint,
///     point2: EmbeddedCurvePoint,
/// ) -> [EmbeddedCurvePoint; 1]
/// ```
fn embedded_curve_add(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
    pedantic_solving: bool,
) -> IResult<Value> {
    let (point1, point2, predicate) = check_three_arguments(arguments, location)?;
    assert_eq!(predicate.0, Value::Bool(true), "ec_add predicate should be true");

    let embedded_curve_point_typ = point1.0.get_type().into_owned();

    let (p1x, p1y, p1inf) = get_embedded_curve_point(point1)?;
    let (p2x, p2y, p2inf) = get_embedded_curve_point(point2)?;

    let (x, y, inf) = Bn254BlackBoxSolver(pedantic_solving)
        .ec_add(
            &p1x,
            &p1y,
            &p1inf.into(),
            &p2x,
            &p2y,
            &p2inf.into(),
            true, // Predicate is always true as interpreter has control flow to handle false case
        )
        .map_err(|e| InterpreterError::BlackBoxError(e, location))?;

    Ok(Value::Array(
        vector![to_embedded_curve_point(x, y, inf > 0_usize.into(), embedded_curve_point_typ)],
        return_type,
    ))
}

/// ```text
/// pub fn multi_scalar_mul<let N: u32>(
///     points: [EmbeddedCurvePoint; N],
///     scalars: [EmbeddedCurveScalar; N],
///     predicate: bool,
/// ) -> [EmbeddedCurvePoint; 1]
/// ```
fn multi_scalar_mul(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
    pedantic_solving: bool,
) -> IResult<Value> {
    let (points, scalars, predicate) = check_three_arguments(arguments, location)?;
    assert_eq!(predicate.0, Value::Bool(true), "multi_scalar_mul predicate should be true");

    let (points, _) = get_array_map(points, get_embedded_curve_point)?;
    let (scalars, _) = get_array_map(scalars, get_embedded_curve_scalar)?;

    let points: Vec<_> = points.into_iter().flat_map(|(x, y, inf)| [x, y, inf.into()]).collect();
    let mut scalars_lo = Vec::new();
    let mut scalars_hi = Vec::new();
    for (lo, hi) in scalars {
        scalars_lo.push(lo);
        scalars_hi.push(hi);
    }

    let (x, y, inf) = Bn254BlackBoxSolver(pedantic_solving)
        .multi_scalar_mul(
            &points,
            &scalars_lo,
            &scalars_hi,
            true, // Predicate is always true as interpreter has control flow to handle false case
        )
        .map_err(|e| InterpreterError::BlackBoxError(e, location))?;

    let embedded_curve_point_typ = match &return_type {
        Type::Array(_, item_type) => item_type.as_ref().clone(),
        _ => {
            return Err(InterpreterError::TypeMismatch {
                expected: "[EmbeddedCurvePoint; 1]".to_string(),
                actual: return_type.clone(),
                location,
            });
        }
    };

    Ok(Value::Array(
        vector![to_embedded_curve_point(x, y, inf > 0_usize.into(), embedded_curve_point_typ)],
        return_type,
    ))
}

/// `poseidon2_permutation<let N: u32>(_input: [Field; N], _state_length: u32) -> [Field; N]`
fn poseidon2_permutation(
    arguments: Vec<(Value, Location)>,
    location: Location,
    pedantic_solving: bool,
) -> IResult<Value> {
    let input = check_one_argument(arguments, location)?;

    let (input, typ) = get_array_map(input, get_field)?;
    let input = vecmap(input, SignedField::to_field_element);

    let fields = Bn254BlackBoxSolver(pedantic_solving)
        .poseidon2_permutation(&input)
        .map_err(|error| InterpreterError::BlackBoxError(error, location))?;

    let array = fields.into_iter().map(|f| Value::Field(SignedField::positive(f))).collect();
    Ok(Value::Array(array, typ))
}

/// `fn keccakf1600(input: [u64; 25]) -> [u64; 25] {}`
fn keccakf1600(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    let input = check_one_argument(arguments, location)?;

    let (state, typ) = get_fixed_array_map(input, get_u64)?;

    let result_lanes = acvm::blackbox_solver::keccakf1600(state)
        .map_err(|error| InterpreterError::BlackBoxError(error, location))?;

    let array: Vector<Value> = result_lanes.into_iter().map(Value::U64).collect();
    Ok(Value::Array(array, typ))
}

/// `pub fn sha256_compression(input: [u32; 16], state: [u32; 8]) -> [u32; 8]`
fn sha256_compression(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    let (input, state) = check_two_arguments(arguments, location)?;

    let (input, _) = get_fixed_array_map(input, get_u32)?;
    let (mut state, typ) = get_fixed_array_map(state, get_u32)?;

    acvm::blackbox_solver::sha256_compression(&mut state, &input);

    let state = state.into_iter().map(Value::U32).collect();
    Ok(Value::Array(state, typ))
}

/// Decode an `EmbeddedCurvePoint` struct.
///
/// Returns `(x, y, is_infinite)`.
fn get_embedded_curve_point(
    (value, location): (Value, Location),
) -> IResult<(FieldElement, FieldElement, bool)> {
    let (fields, typ) = get_struct_fields("EmbeddedCurvePoint", (value, location))?;
    let x = get_struct_field("x", &fields, &typ, location, get_field)?;
    let y = get_struct_field("y", &fields, &typ, location, get_field)?;
    let is_infinite = get_struct_field("is_infinite", &fields, &typ, location, get_bool)?;
    Ok((x.to_field_element(), y.to_field_element(), is_infinite))
}

/// Decode an `EmbeddedCurveScalar` struct.
///
/// Returns `(lo, hi)`.
fn get_embedded_curve_scalar(
    (value, location): (Value, Location),
) -> IResult<(FieldElement, FieldElement)> {
    let (fields, typ) = get_struct_fields("EmbeddedCurveScalar", (value, location))?;
    let lo = get_struct_field("lo", &fields, &typ, location, get_field)?;
    let hi = get_struct_field("hi", &fields, &typ, location, get_field)?;
    Ok((lo.to_field_element(), hi.to_field_element()))
}

fn to_embedded_curve_point(
    x: FieldElement,
    y: FieldElement,
    is_infinite: bool,
    typ: Type,
) -> Value {
    to_struct(
        [
            ("x", Value::Field(SignedField::positive(x))),
            ("y", Value::Field(SignedField::positive(y))),
            ("is_infinite", Value::Bool(is_infinite)),
        ],
        typ,
    )
}

#[cfg(test)]
mod tests {
    use acvm::acir::BlackBoxFunc;
    use noirc_errors::Location;
    use strum::IntoEnumIterator;

    use crate::Type;
    use crate::hir::comptime::InterpreterError::{
        ArgumentCountMismatch, InvalidInComptimeContext, Unimplemented,
    };

    use super::call_foreign;

    /// Check that all `BlackBoxFunc` are covered by `call_foreign`.
    #[test]
    fn test_blackbox_implemented() {
        let no_location = Location::dummy();
        let mut not_implemented = Vec::new();

        for blackbox in BlackBoxFunc::iter() {
            let name = blackbox.name();
            let pedantic_solving = true;
            match call_foreign(name, Vec::new(), Type::Unit, no_location, pedantic_solving) {
                Ok(_) => {
                    // Exists and works with no args (unlikely)
                }
                Err(ArgumentCountMismatch { .. }) => {
                    // Exists but doesn't work with no args (expected)
                }
                Err(InvalidInComptimeContext { .. }) => {}
                Err(Unimplemented { .. }) => not_implemented.push(name),
                Err(other) => panic!("unexpected error: {other:?}"),
            };
        }

        assert!(
            not_implemented.is_empty(),
            "unimplemented blackbox functions: {not_implemented:?}"
        );
    }
}
