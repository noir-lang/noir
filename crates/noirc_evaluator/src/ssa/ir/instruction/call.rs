use std::{collections::VecDeque, rc::Rc};

use acvm::{acir::BlackBoxFunc, BlackBoxResolutionError, FieldElement};
use iter_extended::vecmap;
use num_bigint::BigUint;

use crate::ssa::ir::{
    basic_block::BasicBlockId,
    dfg::DataFlowGraph,
    instruction::Intrinsic,
    map::Id,
    types::Type,
    value::{Value, ValueId},
};

use super::{Binary, BinaryOp, Endian, Instruction, SimplifyResult};

/// Try to simplify this call instruction. If the instruction can be simplified to a known value,
/// that value is returned. Otherwise None is returned.
pub(super) fn simplify_call(
    func: ValueId,
    arguments: &[ValueId],
    block: BasicBlockId,
    dfg: &mut DataFlowGraph,
) -> SimplifyResult {
    let intrinsic = match &dfg[func] {
        Value::Intrinsic(intrinsic) => *intrinsic,
        _ => return SimplifyResult::None,
    };

    let constant_args: Option<Vec<_>> =
        arguments.iter().map(|value_id| dfg.get_numeric_constant(*value_id)).collect();

    match intrinsic {
        Intrinsic::ToBits(endian) => {
            if let Some(constant_args) = constant_args {
                let field = constant_args[0];
                let limb_count = constant_args[1].to_u128() as u32;

                let result_slice = constant_to_radix(endian, field, 2, limb_count, block, dfg);

                let length = dfg
                    .try_get_array_length(result_slice)
                    .expect("ICE: a constant array should have an associated length");
                let len_value =
                    dfg.make_constant(FieldElement::from(length as u128), Type::field());

                // `Intrinsic::ToBits` returns slices which are represented
                // by tuples with the structure (length, slice contents)
                SimplifyResult::SimplifiedToMultiple(vec![len_value, result_slice])
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::ToRadix(endian) => {
            if let Some(constant_args) = constant_args {
                let field = constant_args[0];
                let radix = constant_args[1].to_u128() as u32;
                let limb_count = constant_args[2].to_u128() as u32;

                let result_slice = constant_to_radix(endian, field, radix, limb_count, block, dfg);

                let length = dfg
                    .try_get_array_length(result_slice)
                    .expect("ICE: a constant array should have an associated length");
                let len_value =
                    dfg.make_constant(FieldElement::from(length as u128), Type::field());

                // `Intrinsic::ToRadix` returns slices which are represented
                // by tuples with the structure (length, slice contents)
                SimplifyResult::SimplifiedToMultiple(vec![len_value, result_slice])
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::ArrayLen => {
            if let Some(length) = dfg.try_get_array_length(arguments[0]) {
                let length = FieldElement::from(length as u128);
                SimplifyResult::SimplifiedTo(dfg.make_constant(length, Type::field()))
            } else if matches!(dfg.type_of_value(arguments[1]), Type::Slice(_)) {
                SimplifyResult::SimplifiedTo(arguments[0])
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::SlicePushBack => {
            if let Some((mut elements, typ)) = dfg.get_array_constant(arguments[1]) {
                for elem in &arguments[2..] {
                    elements.push_back(*elem);
                }

                let (length, array) = update_slice(elements, typ, arguments[0], dfg, BinaryOp::Add);
                SimplifyResult::SimplifiedToMultiple(vec![length, array])
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::SlicePushFront => {
            if let Some((mut elements, typ)) = dfg.get_array_constant(arguments[1]) {
                for elem in arguments[2..].iter().rev() {
                    elements.push_front(*elem);
                }

                let (length, array) = update_slice(elements, typ, arguments[0], dfg, BinaryOp::Add);
                SimplifyResult::SimplifiedToMultiple(vec![length, array])
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::SlicePopBack => {
            if let Some((mut slice, typ)) = dfg.get_array_constant(arguments[1]) {
                let element_count = typ.element_size();
                let mut results = VecDeque::with_capacity(element_count + 1);

                // We must pop multiple elements in the case of a slice of tuples
                for _ in 0..element_count {
                    let elem = slice
                        .pop_back()
                        .expect("There are no elements in this slice to be removed");
                    results.push_front(elem);
                }

                let (array, length) = update_slice(slice, typ, arguments[0], dfg, BinaryOp::Sub);
                results.push_front(array);
                results.push_front(length);

                SimplifyResult::SimplifiedToMultiple(results.into())
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::SlicePopFront => {
            if let Some((mut slice, typ)) = dfg.get_array_constant(arguments[1]) {
                let element_count = typ.element_size();

                // We must pop multiple elements in the case of a slice of tuples
                let mut results = vecmap(0..element_count, |_| {
                    slice.pop_front().expect("There are no elements in this slice to be removed")
                });

                // The slice is the last item returned for pop_front
                let (length, array) = update_slice(slice, typ, arguments[0], dfg, BinaryOp::Sub);
                results.push(length);
                results.push(array);
                SimplifyResult::SimplifiedToMultiple(results)
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::SliceInsert => {
            let slice = dfg.get_array_constant(arguments[1]);
            let index = dfg.get_numeric_constant(arguments[2]);
            if let (Some((mut slice, typ)), Some(index)) = (slice, index) {
                let elements = &arguments[3..];
                let mut index = index.to_u128() as usize * elements.len();

                for elem in &arguments[3..] {
                    slice.insert(index, *elem);
                    index += 1;
                }

                let (length, array) = update_slice(slice, typ, arguments[0], dfg, BinaryOp::Add);
                SimplifyResult::SimplifiedToMultiple(vec![length, array])
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::SliceRemove => {
            let slice = dfg.get_array_constant(arguments[1]);
            let index = dfg.get_numeric_constant(arguments[2]);
            if let (Some((mut slice, typ)), Some(index)) = (slice, index) {
                let element_count = typ.element_size();
                let mut results = Vec::with_capacity(element_count + 1);
                let index = index.to_u128() as usize * element_count;

                for _ in 0..element_count {
                    results.push(slice.remove(index));
                }

                let (length, array) = update_slice(slice, typ, arguments[0], dfg, BinaryOp::Sub);
                results.insert(0, array);
                results.insert(0, length);

                SimplifyResult::SimplifiedToMultiple(results)
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::StrAsBytes => {
            // Strings are already represented as bytes internally
            SimplifyResult::SimplifiedTo(arguments[0])
        }
        Intrinsic::AssertConstant => {
            if arguments.iter().all(|argument| dfg.is_constant(*argument)) {
                SimplifyResult::Remove
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::BlackBox(bb_func) => simplify_black_box_func(bb_func, arguments, block, dfg),
        Intrinsic::Sort => simplify_sort(dfg, arguments, block),
    }
}

/// Slices have a tuple structure (slice length, slice contents) to enable logic
/// that uses dynamic slice lengths (such as with merging slices in the flattening pass).
/// This method codegens an update to the slice length.
///
/// The binary operation performed on the slice length is always an addition or subtraction of `1`.
/// This is because the slice length holds the user length (length as displayed by a `.len()` call),
/// and not a flattened length used internally to represent arrays of tuples.
fn update_slice(
    elements: im::Vector<ValueId>,
    typ: Type,
    slice_len: ValueId,
    dfg: &mut DataFlowGraph,
    operator: BinaryOp,
) -> (ValueId, ValueId) {
    let one = dfg.make_constant(FieldElement::one(), Type::field());
    let block = dfg.make_block();
    let instruction = Instruction::Binary(Binary { lhs: slice_len, operator, rhs: one });
    let call_stack = dfg.get_value_call_stack(slice_len);
    let length =
        dfg.insert_instruction_and_results(instruction, block, None, call_stack.clone()).first();

    let instruction = Instruction::MakeArray { elements, typ };
    let array = dfg.insert_instruction_and_results(instruction, block, None, call_stack).first();

    (length, array)
}

/// Try to simplify this black box call. If the call can be simplified to a known value,
/// that value is returned. Otherwise [`SimplifyResult::None`] is returned.
fn simplify_black_box_func(
    bb_func: BlackBoxFunc,
    arguments: &[ValueId],
    block: BasicBlockId,
    dfg: &mut DataFlowGraph,
) -> SimplifyResult {
    match bb_func {
        BlackBoxFunc::SHA256 => simplify_hash(dfg, arguments, block, acvm::blackbox_solver::sha256),
        BlackBoxFunc::Blake2s => {
            simplify_hash(dfg, arguments, block, acvm::blackbox_solver::blake2s)
        }
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

                    let result_array =
                        make_constant_array(dfg, hash_values, Type::unsigned(8), block);
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

        BlackBoxFunc::RecursiveAggregation => SimplifyResult::None,

        BlackBoxFunc::AND => {
            unreachable!("ICE: `BlackBoxFunc::AND` calls should be transformed into a `BinaryOp`")
        }
        BlackBoxFunc::XOR => {
            unreachable!("ICE: `BlackBoxFunc::XOR` calls should be transformed into a `BinaryOp`")
        }
        BlackBoxFunc::RANGE => {
            unreachable!(
                "ICE: `BlackBoxFunc::RANGE` calls should be transformed into a `Instruction::Cast`"
            )
        }
    }
}

fn make_constant_array(
    dfg: &mut DataFlowGraph,
    results: Vec<FieldElement>,
    element_type: Type,
    block: BasicBlockId,
) -> ValueId {
    let results_len = results.len();
    let elements = results
        .into_iter()
        .map(|element| dfg.make_constant(element, element_type.clone()))
        .collect();

    let typ = Type::Array(Rc::new(vec![element_type]), results_len);
    let instruction = Instruction::MakeArray { elements, typ };

    // make_array instructions can't fail at runtime anyway, so this shouldn't need a call stack.
    let no_call_stack = im::Vector::new();
    dfg.insert_instruction_and_results(instruction, block, None, no_call_stack).first()
}

/// Returns a Value::Array of constants corresponding to the limbs of the radix decomposition.
fn constant_to_radix(
    endian: Endian,
    field: FieldElement,
    radix: u32,
    limb_count: u32,
    block: BasicBlockId,
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

    make_constant_array(dfg, limbs, Type::unsigned(bit_size), block)
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
    block: BasicBlockId,
    hash_function: fn(&[u8]) -> Result<[u8; 32], BlackBoxResolutionError>,
) -> SimplifyResult {
    match dfg.get_array_constant(arguments[0]) {
        Some((input, _)) if array_is_constant(dfg, &input) => {
            let input_bytes: Vec<u8> = to_u8_vec(dfg, input);

            let hash = hash_function(&input_bytes)
                .expect("Rust solvable black box function should not fail");

            let hash_values = vecmap(hash, |byte| FieldElement::from_be_bytes_reduce(&[byte]));

            let result_array = make_constant_array(dfg, hash_values, Type::unsigned(8), block);
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

fn simplify_sort(
    dfg: &mut DataFlowGraph,
    arguments: &[ValueId],
    block: BasicBlockId,
) -> SimplifyResult {
    match dfg.get_array_constant(arguments[0]) {
        Some((input, _)) => {
            let inputs: Option<Vec<FieldElement>> =
                input.iter().map(|id| dfg.get_numeric_constant(*id)).collect();

            let Some(mut sorted_inputs) = inputs else { return SimplifyResult::None };
            sorted_inputs.sort_unstable();

            let (_, element_type) = dfg.get_numeric_constant_with_type(input[0]).unwrap();
            let result_array = make_constant_array(dfg, sorted_inputs, element_type, block);
            SimplifyResult::SimplifiedTo(result_array)
        }
        _ => SimplifyResult::None,
    }
}
