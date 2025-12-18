use noirc_errors::call_stack::CallStackId;
use rustc_hash::FxHashMap as HashMap;
use std::{collections::VecDeque, sync::Arc};

use acvm::{AcirField as _, FieldElement, acir::BlackBoxFunc};
use bn254_blackbox_solver::derive_generators;
use iter_extended::vecmap;
use num_bigint::BigUint;

use crate::ssa::ir::{
    basic_block::BasicBlockId,
    dfg::{DataFlowGraph, simplify::value_merger::ValueMerger},
    instruction::{Binary, BinaryOp, Endian, Hint, Instruction, Intrinsic},
    integer::IntegerConstant,
    types::{NumericType, Type},
    value::{Value, ValueId},
};

use super::SimplifyResult;

mod blackbox;

/// Try to simplify this call instruction. If the instruction can be simplified to a known value,
/// that value is returned. Otherwise None is returned.
///
/// The `block` parameter indicates the block any new instructions that are part of a call's
/// simplification will be inserted into. For example, all vector intrinsics require updates
/// to the vector length, which requires inserting a binary instruction. This update instruction
/// must be inserted into the same block that the call itself is being simplified into.
pub(super) fn simplify_call(
    func: ValueId,
    arguments: &[ValueId],
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    ctrl_typevars: Option<Vec<Type>>,
    call_stack: CallStackId,
) -> SimplifyResult {
    let intrinsic = match &dfg[func] {
        Value::Intrinsic(intrinsic) => *intrinsic,
        _ => return SimplifyResult::None,
    };

    let return_type = ctrl_typevars.and_then(|return_types| return_types.first().cloned());

    let constant_args: Option<Vec<_>> =
        arguments.iter().map(|value_id| dfg.get_numeric_constant(*value_id)).collect();

    let simplified_result = match intrinsic {
        Intrinsic::ToBits(endian) => {
            // TODO: simplify to a range constraint if `limb_count == 1`
            if let (Some(constant_args), Some(return_type)) = (constant_args, return_type.clone()) {
                let field = constant_args[0];
                let limb_count = if let Type::Array(_, array_len) = return_type {
                    array_len
                } else {
                    unreachable!("ICE: Intrinsic::ToRadix return type must be array")
                };
                simplify_constant_to_radix(endian, field, 2, limb_count, |values| {
                    make_constant_array(
                        dfg,
                        values.into_iter(),
                        NumericType::bool(),
                        block,
                        call_stack,
                    )
                })
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::ToRadix(endian) => {
            // TODO: simplify to a range constraint if `limb_count == 1`
            if let (Some(constant_args), Some(return_type)) = (constant_args, return_type.clone()) {
                let field = constant_args[0];
                let radix = constant_args[1].to_u128() as u32;
                let limb_count = if let Type::Array(_, array_len) = return_type {
                    array_len
                } else {
                    unreachable!("ICE: Intrinsic::ToRadix return type must be array")
                };
                simplify_constant_to_radix(endian, field, radix, limb_count, |values| {
                    make_constant_array(
                        dfg,
                        values.into_iter(),
                        NumericType::Unsigned { bit_size: 8 },
                        block,
                        call_stack,
                    )
                })
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::ArrayLen => {
            let length = match dfg.type_of_value(arguments[0]) {
                Type::Array(_, length) => {
                    dfg.make_constant(FieldElement::from(length), NumericType::length_type())
                }
                Type::Numeric(NumericType::Unsigned { bit_size: 32 }) => {
                    assert!(matches!(dfg.type_of_value(arguments[1]), Type::Vector(_)));
                    arguments[0]
                }
                _ => panic!("First argument to ArrayLen must be an array or a vector length"),
            };
            SimplifyResult::SimplifiedTo(length)
        }
        // Strings are already arrays of bytes in SSA
        Intrinsic::ArrayAsStrUnchecked => SimplifyResult::SimplifiedTo(arguments[0]),
        Intrinsic::AsVector => {
            let array = dfg.get_array_constant(arguments[0]);
            if let Some((array, array_type)) = array {
                // Compute the resulting vector length
                let inner_element_types = array_type.element_types();
                let vector_length_value = dfg.try_get_vector_capacity(arguments[0]).unwrap();
                let vector_length =
                    dfg.make_constant(vector_length_value.into(), NumericType::length_type());
                let new_vector =
                    make_array(dfg, array, Type::Vector(inner_element_types), block, call_stack);
                SimplifyResult::SimplifiedToMultiple(vec![vector_length, new_vector])
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::VectorPushBack => {
            let vector = dfg.get_array_constant(arguments[1]);
            if let Some((mut vector, element_type)) = vector {
                // TODO(#2752): We need to handle the element_type size to appropriately handle vectors of complex types.
                // This is reliant on dynamic indices of non-homogenous vectors also being implemented.
                if element_type.element_size() != 1 {
                    if let Some(IntegerConstant::Unsigned { value: vector_len, .. }) =
                        dfg.get_integer_constant(arguments[0])
                    {
                        // This simplification, which push back directly on the vector, only works if the real vector_len is the
                        // the length of the vector.
                        if vector_len as usize == vector.len() {
                            // Old code before implementing multiple vector mergers
                            for elem in &arguments[2..] {
                                vector.push_back(*elem);
                            }

                            let new_vector_length =
                                increment_vector_length(arguments[0], dfg, block, call_stack);

                            let new_vector =
                                make_array(dfg, vector, element_type, block, call_stack);
                            return SimplifyResult::SimplifiedToMultiple(vec![
                                new_vector_length,
                                new_vector,
                            ]);
                        }
                    }
                    return SimplifyResult::None;
                }

                simplify_vector_push_back(vector, element_type, arguments, dfg, block, call_stack)
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::VectorPushFront => {
            let vector = dfg.get_array_constant(arguments[1]);
            if let Some((mut vector, element_type)) = vector {
                for elem in arguments[2..].iter().rev() {
                    vector.push_front(*elem);
                }

                let new_vector_length =
                    increment_vector_length(arguments[0], dfg, block, call_stack);

                let new_vector = make_array(dfg, vector, element_type, block, call_stack);
                SimplifyResult::SimplifiedToMultiple(vec![new_vector_length, new_vector])
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::VectorPopBack => {
            let length = dfg.get_numeric_constant(arguments[0]);
            if length.is_none_or(|length| length.is_zero()) {
                // If the length is zero then we're trying to pop the last element from an empty vector.
                // Defer the error to acir_gen.
                return SimplifyResult::None;
            }

            let vector = dfg.get_array_constant(arguments[1]);
            if let Some((_, typ)) = vector {
                simplify_vector_pop_back(typ, arguments, dfg, block, call_stack)
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::VectorPopFront => {
            let length = dfg.get_numeric_constant(arguments[0]);
            if length.is_none_or(|length| length.is_zero()) {
                // If the length is zero then we're trying to pop the first element from an empty vector.
                // Defer the error to acir_gen.
                return SimplifyResult::None;
            }

            let vector = dfg.get_array_constant(arguments[1]);
            if let Some((mut vector, typ)) = vector {
                let element_count = typ.element_size();

                // We must pop multiple elements in the case of a vector of tuples
                let mut results = vecmap(0..element_count, |_| {
                    vector.pop_front().expect("There are no elements in this vector to be removed")
                });

                let new_vector_length =
                    decrement_vector_length(arguments[0], dfg, block, call_stack);

                results.push(new_vector_length);

                let new_vector = make_array(dfg, vector, typ, block, call_stack);

                // The vector is the last item returned for pop_front
                results.push(new_vector);
                SimplifyResult::SimplifiedToMultiple(results)
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::VectorInsert => {
            let vector = dfg.get_array_constant(arguments[1]);
            let index = dfg.get_numeric_constant(arguments[2]);
            if let (Some((mut vector, typ)), Some(index)) = (vector, index) {
                let elements = &arguments[3..];
                let mut index = index.to_u128() as usize * elements.len();

                // Do not simplify the index is greater than the vector capacity
                // or else we will panic inside of the im::Vector insert method
                // Constraints should be generated during SSA gen to tell the user
                // they are attempting to insert at too large of an index
                if index > vector.len() {
                    return SimplifyResult::None;
                }

                for elem in &arguments[3..] {
                    vector.insert(index, *elem);
                    index += 1;
                }

                let new_vector_length =
                    increment_vector_length(arguments[0], dfg, block, call_stack);

                let new_vector = make_array(dfg, vector, typ, block, call_stack);
                SimplifyResult::SimplifiedToMultiple(vec![new_vector_length, new_vector])
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::VectorRemove => {
            let length = dfg.get_numeric_constant(arguments[0]);
            if length.is_none_or(|length| length.is_zero()) {
                // If the length is zero then we're trying to remove an element from an empty vector.
                // Defer the error to acir_gen.
                return SimplifyResult::None;
            }

            let vector = dfg.get_array_constant(arguments[1]);
            let index = dfg.get_numeric_constant(arguments[2]);
            if let (Some((mut vector, typ)), Some(index)) = (vector, index) {
                let element_count = typ.element_size();
                let mut results = Vec::with_capacity(element_count + 1);
                let index = index.to_u128() as usize * element_count;

                // Do not simplify if the index is not less than the vector capacity
                // or else we will panic inside of the im::Vector remove method.
                // Constraints should be generated during SSA gen to tell the user
                // they are attempting to remove at too large of an index.
                if index >= vector.len() {
                    return SimplifyResult::None;
                }

                for _ in 0..element_count {
                    results.push(vector.remove(index));
                }

                let new_vector = make_array(dfg, vector, typ, block, call_stack);
                results.insert(0, new_vector);

                let new_vector_length =
                    decrement_vector_length(arguments[0], dfg, block, call_stack);

                results.insert(0, new_vector_length);

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
        Intrinsic::StaticAssert => {
            if arguments.len() < 2 {
                panic!("ICE: static_assert called with wrong number of arguments")
            }

            // Arguments at positions `1..` form the message and they must all be constant.
            if arguments.iter().skip(1).any(|argument| !dfg.is_constant(*argument)) {
                return SimplifyResult::None;
            }

            if dfg.is_constant_true(arguments[0]) {
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
        Intrinsic::Hint(Hint::BlackBox) => SimplifyResult::None,
        Intrinsic::BlackBox(bb_func) => {
            simplify_black_box_func(bb_func, arguments, dfg, block, call_stack)
        }
        Intrinsic::AsWitness => SimplifyResult::None,
        Intrinsic::IsUnconstrained => {
            let result = dfg.runtime().is_brillig().into();
            SimplifyResult::SimplifiedTo(dfg.make_constant(result, NumericType::bool()))
        }
        Intrinsic::DerivePedersenGenerators => {
            if let Some(Type::Array(_, len)) = return_type.clone() {
                simplify_derive_generators(dfg, arguments, len, block, call_stack)
            } else {
                unreachable!("Derive Pedersen Generators must return an array");
            }
        }
        Intrinsic::FieldLessThan => {
            if let Some(constants) = constant_args {
                let lhs = constants[0];
                let rhs = constants[1];
                let result = dfg.make_constant((lhs < rhs).into(), NumericType::bool());
                SimplifyResult::SimplifiedTo(result)
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::ArrayRefCount | Intrinsic::VectorRefCount => {
            if dfg.runtime.is_acir() {
                // In ACIR, ref counts are not tracked so we always simplify them to zero.
                let zero = dfg.make_constant(FieldElement::zero(), NumericType::unsigned(32));
                SimplifyResult::SimplifiedTo(zero)
            } else {
                SimplifyResult::None
            }
        }
    };

    if let (Some(expected_types), SimplifyResult::SimplifiedTo(result)) =
        (return_type, &simplified_result)
    {
        assert_eq!(
            dfg.type_of_value(*result),
            expected_types,
            "Simplification should not alter return type"
        );
    }

    simplified_result
}

/// Returns a vector (represented by a tuple (len, vector)) of constants corresponding to the limbs of the radix decomposition.
fn simplify_constant_to_radix(
    endian: Endian,
    field: FieldElement,
    radix: u32,
    limb_count: u32,
    mut make_array: impl FnMut(Vec<FieldElement>) -> ValueId,
) -> SimplifyResult {
    match constant_to_radix(endian, field, radix, limb_count) {
        Some(result) => SimplifyResult::SimplifiedTo(make_array(result)),
        None => SimplifyResult::None,
    }
}

pub(crate) fn constant_to_radix(
    endian: Endian,
    field: FieldElement,
    radix: u32,
    limb_count: u32,
) -> Option<Vec<FieldElement>> {
    let bit_size = u32::BITS - (radix - 1).leading_zeros();
    let radix_big = BigUint::from(radix);
    let radix_range = BigUint::from(2u128)..=BigUint::from(256u128);
    if !radix_range.contains(&radix_big) || BigUint::from(2u128).pow(bit_size) != radix_big {
        // NOTE: expect an error to be thrown later in
        // acir::generated_acir::radix_le_decompose
        return None;
    }
    let big_integer = BigUint::from_bytes_be(&field.to_be_bytes());

    // Decompose the integer into its radix digits in little endian form.
    let decomposed_integer = big_integer.to_radix_le(radix);
    if limb_count < decomposed_integer.len() as u32 {
        // `field` cannot be represented as `limb_count` bits.
        // defer error to acir_gen.
        None
    } else {
        let mut limbs = vecmap(0..limb_count, |i| match decomposed_integer.get(i as usize) {
            Some(digit) => FieldElement::from_be_bytes_reduce(&[*digit]),
            None => FieldElement::zero(),
        });
        if endian == Endian::Big {
            limbs.reverse();
        }
        Some(limbs)
    }
}

fn make_constant_array(
    dfg: &mut DataFlowGraph,
    results: impl Iterator<Item = FieldElement>,
    typ: NumericType,
    block: BasicBlockId,
    call_stack: CallStackId,
) -> ValueId {
    let result_constants: im::Vector<_> =
        results.map(|element| dfg.make_constant(element, typ)).collect();

    let typ = Type::Array(Arc::new(vec![Type::Numeric(typ)]), result_constants.len() as u32);
    make_array(dfg, result_constants, typ, block, call_stack)
}

fn make_array(
    dfg: &mut DataFlowGraph,
    elements: im::Vector<ValueId>,
    typ: Type,
    block: BasicBlockId,
    call_stack: CallStackId,
) -> ValueId {
    let instruction = Instruction::MakeArray { elements, typ };
    dfg.insert_instruction_and_results(instruction, block, None, call_stack).first()
}

/// Vectors have a tuple structure (vector length, vector contents) to enable logic
/// that uses dynamic vector lengths (such as with merging vectors in the flattening pass).
/// This method codegens an update to the vector length.
///
/// The binary operation performed on the vector length is always an addition or subtraction of `1`.
/// This is because the vector length holds the user length (length as displayed by a `.len()` call),
/// and not a flattened length used internally to represent arrays of tuples.
fn update_vector_length(
    vector_len: ValueId,
    dfg: &mut DataFlowGraph,
    operator: BinaryOp,
    block: BasicBlockId,
    call_stack: CallStackId,
) -> ValueId {
    let one = dfg.make_constant(FieldElement::one(), NumericType::length_type());
    let instruction = Instruction::Binary(Binary { lhs: vector_len, operator, rhs: one });
    dfg.insert_instruction_and_results(instruction, block, None, call_stack).first()
}

fn increment_vector_length(
    vector_len: ValueId,
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    call_stack: CallStackId,
) -> ValueId {
    update_vector_length(vector_len, dfg, BinaryOp::Add { unchecked: false }, block, call_stack)
}

fn decrement_vector_length(
    vector_len: ValueId,
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    call_stack: CallStackId,
) -> ValueId {
    // Simplifications only run if the length is a known non-zero constant, so the subtraction should never overflow.
    update_vector_length(vector_len, dfg, BinaryOp::Sub { unchecked: true }, block, call_stack)
}

fn simplify_vector_push_back(
    mut vector: im::Vector<ValueId>,
    element_type: Type,
    arguments: &[ValueId],
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    call_stack: CallStackId,
) -> SimplifyResult {
    // The capacity must be an integer so that we can compare it against the vector length
    let capacity = dfg.make_constant((vector.len() as u128).into(), NumericType::length_type());
    let len_equals_capacity_instr =
        Instruction::Binary(Binary { lhs: arguments[0], operator: BinaryOp::Eq, rhs: capacity });
    let len_equals_capacity = dfg
        .insert_instruction_and_results(len_equals_capacity_instr, block, None, call_stack)
        .first();
    let len_not_equals_capacity_instr = Instruction::Not(len_equals_capacity);
    let len_not_equals_capacity = dfg
        .insert_instruction_and_results(len_not_equals_capacity_instr, block, None, call_stack)
        .first();

    let new_vector_length = increment_vector_length(arguments[0], dfg, block, call_stack);

    for elem in &arguments[2..] {
        vector.push_back(*elem);
    }
    let vector_size = vector.len() as u32;
    let element_size = element_type.element_size() as u32;
    let new_vector = make_array(dfg, vector, element_type, block, call_stack);

    let set_last_vector_value_instr = Instruction::ArraySet {
        array: new_vector,
        index: arguments[0],
        value: arguments[2],
        mutable: false,
    };

    let set_last_vector_value = dfg
        .insert_instruction_and_results(set_last_vector_value_instr, block, None, call_stack)
        .first();

    let mut vector_sizes = HashMap::default();
    vector_sizes.insert(set_last_vector_value, vector_size / element_size);
    vector_sizes.insert(new_vector, vector_size / element_size);

    let mut value_merger = ValueMerger::new(dfg, block, &vector_sizes, call_stack);

    let Ok(new_vector) = value_merger.merge_values(
        len_not_equals_capacity,
        len_equals_capacity,
        set_last_vector_value,
        new_vector,
    ) else {
        // If we were to percolate up the error here, it'd get to insert_instruction and eventually
        // all of ssa. Instead we just choose not to simplify the vector call since this should
        // be a rare case.
        return SimplifyResult::None;
    };

    SimplifyResult::SimplifiedToMultiple(vec![new_vector_length, new_vector])
}

fn simplify_vector_pop_back(
    vector_type: Type,
    arguments: &[ValueId],
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    call_stack: CallStackId,
) -> SimplifyResult {
    let element_types = vector_type.element_types();
    let element_count = element_types.len();
    let mut results = VecDeque::with_capacity(element_count + 1);

    let new_vector_length = decrement_vector_length(arguments[0], dfg, block, call_stack);

    let element_size =
        dfg.make_constant((element_count as u128).into(), NumericType::length_type());
    // Compute the flattened length doing an unchecked mul
    // (it shouldn't overflow because it would have overflowed before when the vector was created)
    let flattened_len_instr =
        Instruction::binary(BinaryOp::Mul { unchecked: true }, arguments[0], element_size);
    let mut flattened_len =
        dfg.insert_instruction_and_results(flattened_len_instr, block, None, call_stack).first();

    // We must pop multiple elements in the case of a vector of tuples
    // Iterating through element types in reverse here since we're popping from the end
    for element_type in element_types.iter().rev() {
        flattened_len = decrement_vector_length(flattened_len, dfg, block, call_stack);
        let get_last_elem_instr =
            Instruction::ArrayGet { array: arguments[1], index: flattened_len };

        let element_type = Some(vec![element_type.clone()]);
        let get_last_elem = dfg
            .insert_instruction_and_results(get_last_elem_instr, block, element_type, call_stack)
            .first();
        results.push_front(get_last_elem);
    }

    results.push_front(arguments[1]);

    results.push_front(new_vector_length);
    SimplifyResult::SimplifiedToMultiple(results.into())
}

/// Try to simplify this black box call. If the call can be simplified to a known value,
/// that value is returned. Otherwise [`SimplifyResult::None`] is returned.
fn simplify_black_box_func(
    bb_func: BlackBoxFunc,
    arguments: &[ValueId],
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    call_stack: CallStackId,
) -> SimplifyResult {
    let pedantic_solving = true;
    cfg_if::cfg_if! {
        if #[cfg(feature = "bn254")] {
            let solver = bn254_blackbox_solver::Bn254BlackBoxSolver(pedantic_solving);
        } else {
            let solver = acvm::blackbox_solver::StubbedBlackBoxSolver(pedantic_solving);
        }
    };
    match bb_func {
        BlackBoxFunc::Blake2s => blackbox::simplify_hash(
            dfg,
            arguments,
            acvm::blackbox_solver::blake2s,
            block,
            call_stack,
        ),
        BlackBoxFunc::Blake3 => blackbox::simplify_hash(
            dfg,
            arguments,
            acvm::blackbox_solver::blake3,
            block,
            call_stack,
        ),
        BlackBoxFunc::Keccakf1600 => {
            if let Some((array_input, _)) = dfg.get_array_constant(arguments[0]) {
                if array_is_constant(dfg, &array_input) {
                    let const_input: Vec<u64> = array_input
                        .iter()
                        .map(|id| {
                            let field = dfg
                                .get_numeric_constant(*id)
                                .expect("value id from array should point at constant");
                            field.to_u128() as u64
                        })
                        .collect();

                    let state = acvm::blackbox_solver::keccakf1600(
                        const_input.try_into().expect("Keccakf1600 input should have length of 25"),
                    )
                    .expect("Rust solvable black box function should not fail");
                    let state_values = state.iter().map(|x| FieldElement::from(u128::from(*x)));
                    let result_array = make_constant_array(
                        dfg,
                        state_values,
                        NumericType::Unsigned { bit_size: 64 },
                        block,
                        call_stack,
                    );
                    SimplifyResult::SimplifiedTo(result_array)
                } else {
                    SimplifyResult::None
                }
            } else {
                SimplifyResult::None
            }
        }
        BlackBoxFunc::Poseidon2Permutation => {
            blackbox::simplify_poseidon2_permutation(dfg, solver, arguments, block, call_stack)
        }
        BlackBoxFunc::EcdsaSecp256k1 => blackbox::simplify_signature(
            dfg,
            arguments,
            acvm::blackbox_solver::ecdsa_secp256k1_verify,
        ),
        BlackBoxFunc::EcdsaSecp256r1 => blackbox::simplify_signature(
            dfg,
            arguments,
            acvm::blackbox_solver::ecdsa_secp256r1_verify,
        ),

        BlackBoxFunc::MultiScalarMul => {
            blackbox::simplify_msm(dfg, solver, arguments, block, call_stack)
        }
        BlackBoxFunc::EmbeddedCurveAdd => {
            blackbox::simplify_ec_add(dfg, solver, arguments, block, call_stack)
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
        BlackBoxFunc::Sha256Compression => {
            blackbox::simplify_sha256_compression(dfg, arguments, block, call_stack)
        }
        BlackBoxFunc::AES128Encrypt => SimplifyResult::None,
    }
}

fn to_u8_vec(dfg: &DataFlowGraph, values: im::Vector<ValueId>) -> Vec<u8> {
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

fn array_is_constant(dfg: &DataFlowGraph, values: &im::Vector<ValueId>) -> bool {
    values.iter().all(|value| dfg.get_numeric_constant(*value).is_some())
}

/// Replaces a call to `derive_pedersen_generators` with the results of the computation.
///
/// It only works if the arguments to the call are both constants, which means that the
/// function which contains this call needs to be inlined into its caller, where the
/// arguments are known. This is taken care of by the `#[no_predicates]` attribute,
/// which forces inlining after flattening.
///
/// This intrinsic must not reach Brillig-gen.
fn simplify_derive_generators(
    dfg: &mut DataFlowGraph,
    arguments: &[ValueId],
    num_generators: u32,
    block: BasicBlockId,
    call_stack: CallStackId,
) -> SimplifyResult {
    if arguments.len() == 2 {
        let domain_separator_string = dfg.get_array_constant(arguments[0]);
        let starting_index = dfg.get_numeric_constant(arguments[1]);
        if let (Some(domain_separator_string), Some(starting_index)) =
            (domain_separator_string, starting_index)
        {
            let domain_separator_bytes = domain_separator_string
                .0
                .iter()
                .map(|&x| dfg.get_numeric_constant(x).unwrap().to_u128() as u8)
                .collect::<Vec<u8>>();
            let generators = derive_generators(
                &domain_separator_bytes,
                num_generators,
                starting_index.try_to_u32().expect("argument is declared as u32"),
            );
            let is_infinite = dfg.make_constant(FieldElement::zero(), NumericType::bool());
            let mut results = Vec::new();
            for generator in generators {
                let x_big: BigUint = generator.x.into();
                let x = FieldElement::from_be_bytes_reduce(&x_big.to_bytes_be());
                let y_big: BigUint = generator.y.into();
                let y = FieldElement::from_be_bytes_reduce(&y_big.to_bytes_be());
                results.push(dfg.make_constant(x, NumericType::NativeField));
                results.push(dfg.make_constant(y, NumericType::NativeField));
                results.push(is_infinite);
            }
            let len = results.len() as u32;
            let typ =
                Type::Array(vec![Type::field(), Type::field(), Type::unsigned(1)].into(), len / 3);
            let result = make_array(dfg, results.into(), typ, block, call_stack);
            SimplifyResult::SimplifiedTo(result)
        } else {
            SimplifyResult::None
        }
    } else {
        unreachable!("Unexpected number of arguments to derive_generators");
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::Ssa};

    #[test]
    fn simplify_derive_generators_has_correct_type() {
        let src = r#"
            brillig(inline) fn main func {
              block():
                separator = make_array b"DEFAULT_DOMAIN_SEPARATOR"

                // This call was previously incorrectly simplified to something that returned `[Field; 3]`
                result = call derive_pedersen_generators(separator, u32 0) -> [(Field, Field, u1); 1]

                return result
            }
            "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn main f0 {
          b0():
            v15 = make_array b"DEFAULT_DOMAIN_SEPARATOR"
            v19 = make_array [Field 3728882899078719075161482178784387565366481897740339799480980287259621149274, Field -9903063709032878667290627648209915537972247634463802596148419711785767431332, u1 0] : [(Field, Field, u1); 1]
            return v19
        }
        "#);
    }

    #[test]
    fn simplifies_array_refcount_in_acir_to_zero() {
        let src = r#"
        acir(inline) fn main func {
          b0(v0: [Field; 3]):
            v1 = call array_refcount(v0) -> u32
            return v1
        }
        "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            return u32 0
        }
        ");
    }

    #[test]
    fn does_not_simplify_array_refcount_in_brillig() {
        let src = r#"
        brillig(inline) fn main func {
          b0(v0: [Field; 3]):
            v1 = call array_refcount(v0) -> u32
            return v1
        }
        "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v2 = call array_refcount(v0) -> u32
            return v2
        }
        ");
    }

    #[test]
    fn simplifies_vector_refcount_in_acir_to_zero() {
        let src = r#"
        acir(inline) fn main func {
          b0(v0: [Field]):
            v1 = call vector_refcount(u32 3, v0) -> u32
            return v1
        }
        "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field]):
            return u32 0
        }
        ");
    }

    #[test]
    fn does_not_simplify_vector_refcount_in_brillig() {
        let src = r#"
        brillig(inline) fn main func {
          b0(v0: [Field]):
            v1 = call vector_refcount(u32 3, v0) -> u32
            return v1
        }
        "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: [Field]):
            v3 = call vector_refcount(u32 3, v0) -> u32
            return v3
        }
        ");
    }

    #[test]
    fn simplifies_array_len_for_array() {
        let src = r#"
        acir(inline) fn main func {
          b0(v0: [Field; 3]):
            v1 = call array_len(v0) -> u32
            return v1
        }
        "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            return u32 3
        }
        ");
    }

    #[test]
    fn simplifies_array_len_for_vector() {
        let src = r#"
        acir(inline) fn main func {
          b0(v0: u32, v1: [Field]):
            v2 = call array_len(v0, v1) -> u32
            return v2
        }
        "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32, v1: [Field]):
            return v0
        }
        ");
    }

    #[should_panic(expected = "First argument to ArrayLen must be an array or a vector length")]
    #[test]
    fn panics_on_array_len_with_wrong_type() {
        let src = r#"
        acir(inline) fn main func {
          b0(v0: u64):
            v2 = call array_len(v0) -> u32
            return v2
        }
        "#;
        let _ = Ssa::from_str_simplifying(src).unwrap();
    }

    #[test]
    fn can_handle_zero_len_vector() {
        let src = r#"
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [] : [(); 1]
            v1 = make_array [] : [()]
            return
        }
        "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [] : [(); 1]
            v1 = make_array [] : [()]
            return
        }
        ");
    }
}
