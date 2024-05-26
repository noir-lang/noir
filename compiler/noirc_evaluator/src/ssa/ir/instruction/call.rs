use fxhash::FxHashMap as HashMap;
use std::{collections::VecDeque, rc::Rc};

use acvm::{acir::AcirField, acir::BlackBoxFunc, BlackBoxResolutionError, FieldElement};
use iter_extended::vecmap;
use num_bigint::BigUint;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::{CallStack, DataFlowGraph},
        instruction::Intrinsic,
        map::Id,
        types::Type,
        value::{Value, ValueId},
    },
    opt::flatten_cfg::value_merger::ValueMerger,
};

use super::{Binary, BinaryOp, Endian, Instruction, SimplifyResult};

/// Try to simplify this call instruction. If the instruction can be simplified to a known value,
/// that value is returned. Otherwise None is returned.
///
/// The `block` parameter indicates the block any new instructions that are part of a call's
/// simplification will be inserted into. For example, all slice intrinsics require updates
/// to the slice length, which requires inserting a binary instruction. This update instruction
/// must be inserted into the same block that the call itself is being simplified into.
pub(super) fn simplify_call(
    func: ValueId,
    arguments: &[ValueId],
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    ctrl_typevars: Option<Vec<Type>>,
    call_stack: &CallStack,
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

                let (len_value, result_slice) =
                    constant_to_radix(endian, field, 2, limb_count, dfg);

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

                let (len_value, result_slice) =
                    constant_to_radix(endian, field, radix, limb_count, dfg);

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
                SimplifyResult::SimplifiedTo(dfg.make_constant(length, Type::length_type()))
            } else if matches!(dfg.type_of_value(arguments[1]), Type::Slice(_)) {
                SimplifyResult::SimplifiedTo(arguments[0])
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::AsSlice => {
            let array = dfg.get_array_constant(arguments[0]);
            if let Some((array, array_type)) = array {
                // Compute the resulting slice length by dividing the flattened
                // array length by the size of each array element
                let elements_size = array_type.element_size();
                let inner_element_types = array_type.element_types();
                assert_eq!(
                    0,
                    array.len() % elements_size,
                    "expected array length to be multiple of its elements size"
                );
                let slice_length_value = array.len() / elements_size;
                let slice_length =
                    dfg.make_constant(slice_length_value.into(), Type::length_type());
                let new_slice = dfg.make_array(array, Type::Slice(inner_element_types));
                SimplifyResult::SimplifiedToMultiple(vec![slice_length, new_slice])
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::SlicePushBack => {
            let slice = dfg.get_array_constant(arguments[1]);
            if let Some((mut slice, element_type)) = slice {
                // TODO(#2752): We need to handle the element_type size to appropriately handle slices of complex types.
                // This is reliant on dynamic indices of non-homogenous slices also being implemented.
                if element_type.element_size() != 1 {
                    // Old code before implementing multiple slice mergers
                    for elem in &arguments[2..] {
                        slice.push_back(*elem);
                    }

                    let new_slice_length =
                        update_slice_length(arguments[0], dfg, BinaryOp::Add, block);

                    let new_slice = dfg.make_array(slice, element_type);
                    return SimplifyResult::SimplifiedToMultiple(vec![new_slice_length, new_slice]);
                }

                simplify_slice_push_back(slice, element_type, arguments, dfg, block)
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::SlicePushFront => {
            let slice = dfg.get_array_constant(arguments[1]);
            if let Some((mut slice, element_type)) = slice {
                for elem in arguments[2..].iter().rev() {
                    slice.push_front(*elem);
                }

                let new_slice_length = update_slice_length(arguments[0], dfg, BinaryOp::Add, block);

                let new_slice = dfg.make_array(slice, element_type);
                SimplifyResult::SimplifiedToMultiple(vec![new_slice_length, new_slice])
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::SlicePopBack => {
            let slice = dfg.get_array_constant(arguments[1]);
            if let Some((_, typ)) = slice {
                simplify_slice_pop_back(typ, arguments, dfg, block)
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::SlicePopFront => {
            let slice = dfg.get_array_constant(arguments[1]);
            if let Some((mut slice, typ)) = slice {
                let element_count = typ.element_size();

                // We must pop multiple elements in the case of a slice of tuples
                let mut results = vecmap(0..element_count, |_| {
                    slice.pop_front().expect("There are no elements in this slice to be removed")
                });

                let new_slice_length = update_slice_length(arguments[0], dfg, BinaryOp::Sub, block);

                results.push(new_slice_length);

                let new_slice = dfg.make_array(slice, typ);

                // The slice is the last item returned for pop_front
                results.push(new_slice);
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

                // Do not simplify the index is greater than the slice capacity
                // or else we will panic inside of the im::Vector insert method
                // Constraints should be generated during SSA gen to tell the user
                // they are attempting to insert at too large of an index
                if index > slice.len() {
                    return SimplifyResult::None;
                }

                for elem in &arguments[3..] {
                    slice.insert(index, *elem);
                    index += 1;
                }

                let new_slice_length = update_slice_length(arguments[0], dfg, BinaryOp::Add, block);

                let new_slice = dfg.make_array(slice, typ);
                SimplifyResult::SimplifiedToMultiple(vec![new_slice_length, new_slice])
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

                // Do not simplify if the index is not less than the slice capacity
                // or else we will panic inside of the im::Vector remove method.
                // Constraints should be generated during SSA gen to tell the user
                // they are attempting to remove at too large of an index.
                if index >= slice.len() {
                    return SimplifyResult::None;
                }

                for _ in 0..element_count {
                    results.push(slice.remove(index));
                }

                let new_slice = dfg.make_array(slice, typ);
                results.insert(0, new_slice);

                let new_slice_length = update_slice_length(arguments[0], dfg, BinaryOp::Sub, block);

                results.insert(0, new_slice_length);

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
        Intrinsic::ApplyRangeConstraint => {
            let value = arguments[0];
            let max_bit_size = dfg.get_numeric_constant(arguments[1]);
            if let Some(max_bit_size) = max_bit_size {
                let max_bit_size = max_bit_size.to_u128() as u32;
                let max_potential_bits = dfg.get_value_max_num_bits(value);
                if max_potential_bits < max_bit_size {
                    SimplifyResult::Remove
                } else {
                    SimplifyResult::SimplifiedToInstruction(Instruction::RangeCheck {
                        value,
                        max_bit_size,
                        assert_message: Some("call to assert_max_bit_size".to_owned()),
                    })
                }
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::BlackBox(bb_func) => simplify_black_box_func(bb_func, arguments, dfg),
        Intrinsic::AsField => {
            let instruction = Instruction::Cast(
                arguments[0],
                Type::Numeric(crate::ssa::ir::types::NumericType::NativeField),
            );
            SimplifyResult::SimplifiedToInstruction(instruction)
        }
        Intrinsic::FromField => {
            let incoming_type = Type::field();
            let target_type = ctrl_typevars.unwrap().remove(0);

            let truncate = Instruction::Truncate {
                value: arguments[0],
                bit_size: target_type.bit_size(),
                max_bit_size: incoming_type.bit_size(),
            };
            let truncated_value = dfg
                .insert_instruction_and_results(
                    truncate,
                    block,
                    Some(vec![incoming_type]),
                    call_stack.clone(),
                )
                .first();

            let instruction = Instruction::Cast(truncated_value, target_type);
            SimplifyResult::SimplifiedToInstruction(instruction)
        }
        Intrinsic::AsWitness => SimplifyResult::None,
        Intrinsic::IsUnconstrained => SimplifyResult::None,
    }
}

/// Slices have a tuple structure (slice length, slice contents) to enable logic
/// that uses dynamic slice lengths (such as with merging slices in the flattening pass).
/// This method codegens an update to the slice length.
///
/// The binary operation performed on the slice length is always an addition or subtraction of `1`.
/// This is because the slice length holds the user length (length as displayed by a `.len()` call),
/// and not a flattened length used internally to represent arrays of tuples.
fn update_slice_length(
    slice_len: ValueId,
    dfg: &mut DataFlowGraph,
    operator: BinaryOp,
    block: BasicBlockId,
) -> ValueId {
    let one = dfg.make_constant(FieldElement::one(), Type::length_type());
    let instruction = Instruction::Binary(Binary { lhs: slice_len, operator, rhs: one });
    let call_stack = dfg.get_value_call_stack(slice_len);
    dfg.insert_instruction_and_results(instruction, block, None, call_stack).first()
}

fn simplify_slice_push_back(
    mut slice: im::Vector<ValueId>,
    element_type: Type,
    arguments: &[ValueId],
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
) -> SimplifyResult {
    // The capacity must be an integer so that we can compare it against the slice length
    let capacity = dfg.make_constant((slice.len() as u128).into(), Type::length_type());
    let len_equals_capacity_instr =
        Instruction::Binary(Binary { lhs: arguments[0], operator: BinaryOp::Eq, rhs: capacity });
    let call_stack = dfg.get_value_call_stack(arguments[0]);
    let len_equals_capacity = dfg
        .insert_instruction_and_results(len_equals_capacity_instr, block, None, call_stack.clone())
        .first();
    let len_not_equals_capacity_instr = Instruction::Not(len_equals_capacity);
    let len_not_equals_capacity = dfg
        .insert_instruction_and_results(
            len_not_equals_capacity_instr,
            block,
            None,
            call_stack.clone(),
        )
        .first();

    let new_slice_length = update_slice_length(arguments[0], dfg, BinaryOp::Add, block);

    for elem in &arguments[2..] {
        slice.push_back(*elem);
    }
    let slice_size = slice.len();
    let element_size = element_type.element_size();
    let new_slice = dfg.make_array(slice, element_type);

    let set_last_slice_value_instr = Instruction::ArraySet {
        array: new_slice,
        index: arguments[0],
        value: arguments[2],
        mutable: false,
    };

    let set_last_slice_value = dfg
        .insert_instruction_and_results(set_last_slice_value_instr, block, None, call_stack)
        .first();

    let mut slice_sizes = HashMap::default();
    slice_sizes.insert(set_last_slice_value, slice_size / element_size);
    slice_sizes.insert(new_slice, slice_size / element_size);

    let unknown = &mut HashMap::default();
    let mut value_merger = ValueMerger::new(dfg, block, &mut slice_sizes, unknown, None);

    let new_slice = value_merger.merge_values(
        len_not_equals_capacity,
        len_equals_capacity,
        set_last_slice_value,
        new_slice,
    );

    SimplifyResult::SimplifiedToMultiple(vec![new_slice_length, new_slice])
}

fn simplify_slice_pop_back(
    element_type: Type,
    arguments: &[ValueId],
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
) -> SimplifyResult {
    let element_types = match element_type.clone() {
        Type::Slice(element_types) | Type::Array(element_types, _) => element_types,
        _ => {
            unreachable!("ICE: Expected slice or array, but got {element_type}");
        }
    };

    let element_count = element_type.element_size();
    let mut results = VecDeque::with_capacity(element_count + 1);

    let new_slice_length = update_slice_length(arguments[0], dfg, BinaryOp::Sub, block);

    let element_size = dfg.make_constant((element_count as u128).into(), Type::length_type());
    let flattened_len_instr = Instruction::binary(BinaryOp::Mul, arguments[0], element_size);
    let mut flattened_len = dfg
        .insert_instruction_and_results(flattened_len_instr, block, None, CallStack::new())
        .first();
    flattened_len = update_slice_length(flattened_len, dfg, BinaryOp::Sub, block);

    // We must pop multiple elements in the case of a slice of tuples
    for _ in 0..element_count {
        let get_last_elem_instr =
            Instruction::ArrayGet { array: arguments[1], index: flattened_len };
        let get_last_elem = dfg
            .insert_instruction_and_results(
                get_last_elem_instr,
                block,
                Some(element_types.to_vec()),
                CallStack::new(),
            )
            .first();
        results.push_front(get_last_elem);

        flattened_len = update_slice_length(flattened_len, dfg, BinaryOp::Sub, block);
    }

    results.push_front(arguments[1]);

    results.push_front(new_slice_length);
    SimplifyResult::SimplifiedToMultiple(results.into())
}

/// Try to simplify this black box call. If the call can be simplified to a known value,
/// that value is returned. Otherwise [`SimplifyResult::None`] is returned.
fn simplify_black_box_func(
    bb_func: BlackBoxFunc,
    arguments: &[ValueId],
    dfg: &mut DataFlowGraph,
) -> SimplifyResult {
    match bb_func {
        BlackBoxFunc::SHA256 => simplify_hash(dfg, arguments, acvm::blackbox_solver::sha256),
        BlackBoxFunc::Blake2s => simplify_hash(dfg, arguments, acvm::blackbox_solver::blake2s),
        BlackBoxFunc::Blake3 => simplify_hash(dfg, arguments, acvm::blackbox_solver::blake3),
        BlackBoxFunc::Keccakf1600 => SimplifyResult::None, //TODO(Guillaume)
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
        BlackBoxFunc::Poseidon2Permutation => SimplifyResult::None, //TODO(Guillaume)
        BlackBoxFunc::EcdsaSecp256k1 => {
            simplify_signature(dfg, arguments, acvm::blackbox_solver::ecdsa_secp256k1_verify)
        }
        BlackBoxFunc::EcdsaSecp256r1 => {
            simplify_signature(dfg, arguments, acvm::blackbox_solver::ecdsa_secp256r1_verify)
        }

        BlackBoxFunc::MultiScalarMul
        | BlackBoxFunc::SchnorrVerify
        | BlackBoxFunc::PedersenCommitment
        | BlackBoxFunc::PedersenHash
        | BlackBoxFunc::EmbeddedCurveAdd => {
            // Currently unsolvable here as we rely on an implementation in the backend.
            SimplifyResult::None
        }
        BlackBoxFunc::BigIntAdd
        | BlackBoxFunc::BigIntSub
        | BlackBoxFunc::BigIntMul
        | BlackBoxFunc::BigIntDiv
        | BlackBoxFunc::RecursiveAggregation
        | BlackBoxFunc::BigIntFromLeBytes
        | BlackBoxFunc::BigIntToLeBytes => SimplifyResult::None,

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
        BlackBoxFunc::Sha256Compression => SimplifyResult::None, //TODO(Guillaume)
        BlackBoxFunc::AES128Encrypt => SimplifyResult::None,
    }
}

fn make_constant_array(dfg: &mut DataFlowGraph, results: Vec<FieldElement>, typ: Type) -> ValueId {
    let result_constants = vecmap(results, |element| dfg.make_constant(element, typ.clone()));

    let typ = Type::Array(Rc::new(vec![typ]), result_constants.len());
    dfg.make_array(result_constants.into(), typ)
}

fn make_constant_slice(
    dfg: &mut DataFlowGraph,
    results: Vec<FieldElement>,
    typ: Type,
) -> (ValueId, ValueId) {
    let result_constants = vecmap(results, |element| dfg.make_constant(element, typ.clone()));

    let typ = Type::Slice(Rc::new(vec![typ]));
    let length = FieldElement::from(result_constants.len() as u128);
    (dfg.make_constant(length, Type::length_type()), dfg.make_array(result_constants.into(), typ))
}

/// Returns a slice (represented by a tuple (len, slice)) of constants corresponding to the limbs of the radix decomposition.
fn constant_to_radix(
    endian: Endian,
    field: FieldElement,
    radix: u32,
    limb_count: u32,
    dfg: &mut DataFlowGraph,
) -> (ValueId, ValueId) {
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
    make_constant_slice(dfg, limbs, Type::unsigned(bit_size))
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
