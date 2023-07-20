use std::rc::Rc;

use acvm::{acir::BlackBoxFunc, BlackBoxResolutionError, FieldElement};
use iter_extended::vecmap;
use num_bigint::BigUint;

use crate::ssa_refactor::ir::{
    dfg::DataFlowGraph,
    instruction::Intrinsic,
    map::Id,
    types::{NumericType, Type},
    value::{Value, ValueId},
};

use super::{Endian, SimplifyResult};

/// Try to simplify this call instruction. If the instruction can be simplified to a known value,
/// that value is returned. Otherwise None is returned.
pub(super) fn simplify_call(
    func: ValueId,
    arguments: &[ValueId],
    dfg: &mut DataFlowGraph,
) -> SimplifyResult {
    use SimplifyResult::*;
    let intrinsic = match &dfg[func] {
        Value::Intrinsic(intrinsic) => *intrinsic,
        _ => return None,
    };

    let constant_args: Option<Vec<_>> =
        arguments.iter().map(|value_id| dfg.get_numeric_constant(*value_id)).collect();

    match intrinsic {
        Intrinsic::ToBits(endian) => {
            if let Some(constant_args) = constant_args {
                let field = constant_args[0];
                let limb_count = constant_args[1].to_u128() as u32;
                SimplifiedTo(constant_to_radix(endian, field, 2, limb_count, dfg))
            } else {
                None
            }
        }
        Intrinsic::ToRadix(endian) => {
            if let Some(constant_args) = constant_args {
                let field = constant_args[0];
                let radix = constant_args[1].to_u128() as u32;
                let limb_count = constant_args[2].to_u128() as u32;
                SimplifiedTo(constant_to_radix(endian, field, radix, limb_count, dfg))
            } else {
                None
            }
        }
        Intrinsic::ArrayLen => {
            let slice = dfg.get_array_constant(arguments[0]);
            if let Some((slice, _)) = slice {
                SimplifiedTo(dfg.make_constant((slice.len() as u128).into(), Type::field()))
            } else if let Some(length) = dfg.try_get_array_length(arguments[0]) {
                SimplifiedTo(dfg.make_constant((length as u128).into(), Type::field()))
            } else {
                None
            }
        }
        Intrinsic::SlicePushBack => {
            let slice = dfg.get_array_constant(arguments[0]);
            if let (Some((mut slice, element_type)), elem) = (slice, arguments[1]) {
                slice.push_back(elem);
                let new_slice = dfg.make_array(slice, element_type);
                SimplifiedTo(new_slice)
            } else {
                None
            }
        }
        Intrinsic::SlicePushFront => {
            let slice = dfg.get_array_constant(arguments[0]);
            if let (Some((mut slice, element_type)), elem) = (slice, arguments[1]) {
                slice.push_front(elem);
                let new_slice = dfg.make_array(slice, element_type);
                SimplifiedTo(new_slice)
            } else {
                None
            }
        }
        Intrinsic::SlicePopBack => {
            let slice = dfg.get_array_constant(arguments[0]);
            if let Some((mut slice, element_type)) = slice {
                let elem =
                    slice.pop_back().expect("There are no elements in this slice to be removed");
                let new_slice = dfg.make_array(slice, element_type);
                SimplifiedToMultiple(vec![new_slice, elem])
            } else {
                None
            }
        }
        Intrinsic::SlicePopFront => {
            let slice = dfg.get_array_constant(arguments[0]);
            if let Some((mut slice, element_type)) = slice {
                let elem =
                    slice.pop_front().expect("There are no elements in this slice to be removed");
                let new_slice = dfg.make_array(slice, element_type);
                SimplifiedToMultiple(vec![elem, new_slice])
            } else {
                None
            }
        }
        Intrinsic::SliceInsert => {
            let slice = dfg.get_array_constant(arguments[0]);
            let index = dfg.get_numeric_constant(arguments[1]);
            if let (Some((mut slice, element_type)), Some(index), value) =
                (slice, index, arguments[2])
            {
                slice.insert(index.to_u128() as usize, value);
                let new_slice = dfg.make_array(slice, element_type);
                SimplifiedTo(new_slice)
            } else {
                None
            }
        }
        Intrinsic::SliceRemove => {
            let slice = dfg.get_array_constant(arguments[0]);
            let index = dfg.get_numeric_constant(arguments[1]);
            if let (Some((mut slice, element_type)), Some(index)) = (slice, index) {
                let removed_elem = slice.remove(index.to_u128() as usize);
                let new_slice = dfg.make_array(slice, element_type);
                SimplifiedToMultiple(vec![new_slice, removed_elem])
            } else {
                None
            }
        }
        Intrinsic::BlackBox(bb_func) => simplify_bb_func(bb_func, arguments, dfg),
        Intrinsic::Println | Intrinsic::Sort => None,
    }
}

/// Try to simplify this black box call. If the call can be simplified to a known value,
/// that value is returned. Otherwise [`SimplifyResult::None`] is returned.
fn simplify_bb_func(
    bb_func: BlackBoxFunc,
    arguments: &[ValueId],
    dfg: &mut DataFlowGraph,
) -> SimplifyResult {
    match bb_func {
        BlackBoxFunc::AND => {
            if let (Some((lhs, Type::Numeric(NumericType::Unsigned { bit_size }))), Some(rhs)) = (
                dfg.get_numeric_constant_with_type(arguments[0]),
                dfg.get_numeric_constant(arguments[1]),
            ) {
                let output = lhs.and(&rhs, bit_size);
                let output = dfg.make_constant(output, Type::unsigned(bit_size));
                SimplifyResult::SimplifiedTo(output)
            } else {
                SimplifyResult::None
            }
        }
        BlackBoxFunc::XOR => {
            if let (Some((lhs, Type::Numeric(NumericType::Unsigned { bit_size }))), Some(rhs)) = (
                dfg.get_numeric_constant_with_type(arguments[0]),
                dfg.get_numeric_constant(arguments[1]),
            ) {
                let output = lhs.xor(&rhs, bit_size);
                let output = dfg.make_constant(output, Type::unsigned(bit_size));
                SimplifyResult::SimplifiedTo(output)
            } else {
                SimplifyResult::None
            }
        }
        BlackBoxFunc::SHA256 => simplify_hash(dfg, arguments, acvm::blackbox_solver::sha256),
        BlackBoxFunc::Blake2s => simplify_hash(dfg, arguments, acvm::blackbox_solver::blake2s),
        BlackBoxFunc::Keccak256 => {
            match (dfg.get_array_constant(arguments[0]), dfg.get_numeric_constant(arguments[1])) {
                (Some((input, _)), Some(num_bytes)) if array_is_constant(dfg, &input) => {
                    let input_bytes: Vec<u8> = to_u8_vec(dfg, input);

                    let num_bytes = num_bytes.to_u128() as usize;
                    let truncated_input_bytes = &input_bytes[0..num_bytes];
                    let hash = acvm::blackbox_solver::keccak256(truncated_input_bytes)
                        .expect("Rust solvable black box function should not fail");

                    let hash_values =
                        vecmap(hash, |byte| FieldElement::from_be_bytes_reduce(&[byte]));

                    let result_array = make_constant_array(dfg, hash_values, Type::unsigned(8));
                    SimplifyResult::SimplifiedTo(result_array)
                }
                _ => SimplifyResult::None,
            }
        }
        BlackBoxFunc::HashToField128Security => match dfg.get_array_constant(arguments[0]) {
            Some((input, _)) if array_is_constant(dfg, &input) => {
                let input_bytes: Vec<u8> = to_u8_vec(dfg, input);

                let field = acvm::blackbox_solver::hash_to_field_128_security(&input_bytes)
                    .expect("Rust solvable black box function should not fail");

                let field_constant = dfg.make_constant(field, Type::field());
                SimplifyResult::SimplifiedTo(field_constant)
            }
            _ => SimplifyResult::None,
        },

        BlackBoxFunc::EcdsaSecp256k1 => {
            simplify_signature(dfg, arguments, acvm::blackbox_solver::ecdsa_secp256k1_verify)
        }
        BlackBoxFunc::EcdsaSecp256r1 => {
            simplify_signature(dfg, arguments, acvm::blackbox_solver::ecdsa_secp256r1_verify)
        }

        BlackBoxFunc::FixedBaseScalarMul | BlackBoxFunc::SchnorrVerify | BlackBoxFunc::Pedersen => {
            // Currently unsolvable here as we rely on an implementation in the backend.
            SimplifyResult::None
        }
        BlackBoxFunc::RANGE | BlackBoxFunc::RecursiveAggregation => SimplifyResult::None,
    }
}

fn make_constant_array(dfg: &mut DataFlowGraph, results: Vec<FieldElement>, typ: Type) -> ValueId {
    let result_constants = vecmap(results, |element| dfg.make_constant(element, typ.clone()));

    let typ = Type::Array(Rc::new(vec![typ]), result_constants.len());
    dfg.make_array(result_constants.into(), typ)
}

/// Returns a Value::Array of constants corresponding to the limbs of the radix decomposition.
fn constant_to_radix(
    endian: Endian,
    field: FieldElement,
    radix: u32,
    limb_count: u32,
    dfg: &mut DataFlowGraph,
) -> ValueId {
    let bit_size = u32::BITS - (radix - 1).leading_zeros();
    let radix_big = BigUint::from(radix);
    assert_eq!(BigUint::from(2u128).pow(bit_size), radix_big, "ICE: Radix must be a power of 2");
    let big_integer = BigUint::from_bytes_be(&field.to_be_bytes());

    // Decompose the integer into its radix digits in little endian form.
    let decomposed_integer = big_integer.to_radix_le(radix);
    let mut limbs = vecmap(0..limb_count, |i| match decomposed_integer.get(i as usize) {
        Some(digit) => FieldElement::from_be_bytes_reduce(&[*digit]),
        None => FieldElement::zero(),
    });
    if endian == Endian::Big {
        limbs.reverse();
    }

    // For legacy reasons (see #617) the to_radix interface supports 256 bits even though
    // FieldElement::max_num_bits() is only 254 bits. Any limbs beyond the specified count
    // become zero padding.
    let max_decomposable_bits: u32 = 256;
    let limb_count_with_padding = max_decomposable_bits / bit_size;
    while limbs.len() < limb_count_with_padding as usize {
        limbs.push(FieldElement::zero());
    }

    make_constant_array(dfg, limbs, Type::unsigned(bit_size))
}

fn to_u8_vec(dfg: &DataFlowGraph, values: im::Vector<Id<Value>>) -> Vec<u8> {
    values
        .iter()
        .map(|id| {
            let field = dfg
                .get_numeric_constant(*id)
                .expect("value id from array should point at constant");
            *field.to_be_bytes().last().unwrap()
        })
        .collect()
}

fn array_is_constant(dfg: &DataFlowGraph, values: &im::Vector<Id<Value>>) -> bool {
    values.iter().all(|value| dfg.get_numeric_constant(*value).is_some())
}

fn simplify_hash(
    dfg: &mut DataFlowGraph,
    arguments: &[ValueId],
    hash_function: fn(&[u8]) -> Result<[u8; 32], BlackBoxResolutionError>,
) -> SimplifyResult {
    match dfg.get_array_constant(arguments[0]) {
        Some((input, _)) if array_is_constant(dfg, &input) => {
            let input_bytes: Vec<u8> = to_u8_vec(dfg, input);

            let hash = hash_function(&input_bytes)
                .expect("Rust solvable black box function should not fail");

            let hash_values = vecmap(hash, |byte| FieldElement::from_be_bytes_reduce(&[byte]));

            let result_array = make_constant_array(dfg, hash_values, Type::unsigned(8));
            SimplifyResult::SimplifiedTo(result_array)
        }
        _ => SimplifyResult::None,
    }
}

type ECDSASignatureVerifier = fn(
    hashed_msg: &[u8],
    public_key_x: &[u8; 32],
    public_key_y: &[u8; 32],
    signature: &[u8; 64],
) -> Result<bool, BlackBoxResolutionError>;
fn simplify_signature(
    dfg: &mut DataFlowGraph,
    arguments: &[ValueId],
    signature_verifier: ECDSASignatureVerifier,
) -> SimplifyResult {
    match (
        dfg.get_array_constant(arguments[0]),
        dfg.get_array_constant(arguments[1]),
        dfg.get_array_constant(arguments[2]),
        dfg.get_array_constant(arguments[3]),
    ) {
        (
            Some((public_key_x, _)),
            Some((public_key_y, _)),
            Some((signature, _)),
            Some((hashed_message, _)),
        ) if array_is_constant(dfg, &public_key_x)
            && array_is_constant(dfg, &public_key_y)
            && array_is_constant(dfg, &signature)
            && array_is_constant(dfg, &hashed_message) =>
        {
            let public_key_x: [u8; 32] = to_u8_vec(dfg, public_key_x)
                .try_into()
                .expect("ECDSA public key fields are 32 bytes");
            let public_key_y: [u8; 32] = to_u8_vec(dfg, public_key_y)
                .try_into()
                .expect("ECDSA public key fields are 32 bytes");
            let signature: [u8; 64] =
                to_u8_vec(dfg, signature).try_into().expect("ECDSA signatures are 64 bytes");
            let hashed_message: Vec<u8> = to_u8_vec(dfg, hashed_message);

            let valid_signature =
                signature_verifier(&hashed_message, &public_key_x, &public_key_y, &signature)
                    .expect("Rust solvable black box function should not fail");

            let valid_signature = dfg.make_constant(valid_signature.into(), Type::bool());
            SimplifyResult::SimplifiedTo(valid_signature)
        }
        _ => SimplifyResult::None,
    }
}
