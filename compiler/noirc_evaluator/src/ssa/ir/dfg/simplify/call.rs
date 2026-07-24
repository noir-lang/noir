use noirc_errors::call_stack::CallStackId;
use std::{collections::VecDeque, sync::Arc};

use acvm::{
    AcirField as _, FieldElement,
    acir::{
        BlackBoxFunc,
        brillig::lengths::{ElementTypesLength, SemanticLength, SemiFlattenedLength},
    },
};
use bn254_blackbox_solver::derive_generators;
use iter_extended::vecmap;
use num_bigint::BigUint;

use crate::{
    brillig::assert_u32,
    ssa::ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        instruction::{Binary, BinaryOp, ConstrainError, Endian, Hint, Instruction, Intrinsic},
        integer::IntegerConstant,
        types::{NumericType, Type},
        value::{Value, ValueId},
    },
};

use super::SimplifyResult;
use super::bail_malformed;

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
    ctrl_typevars: Option<&[Type]>,
    call_stack: CallStackId,
) -> SimplifyResult {
    let intrinsic = match &dfg[func] {
        Value::Intrinsic(intrinsic) => *intrinsic,
        _ => return SimplifyResult::None,
    };

    let return_type = ctrl_typevars.and_then(|return_types| return_types.first());

    let constant_args: Option<Vec<_>> =
        arguments.iter().map(|value_id| dfg.get_numeric_constant(*value_id)).collect();

    let simplified_result = match intrinsic {
        Intrinsic::ToBits(endian) => {
            // TODO: simplify to a range constraint if `limb_count == 1`
            if let (Some(constant_args), Some(return_type)) = (constant_args, return_type) {
                let field = constant_args[0];
                let Type::Array(_, limb_count) = return_type else {
                    unreachable!("ICE: Intrinsic::ToRadix return type must be array")
                };
                simplify_constant_to_radix(endian, field, 2, limb_count.0, |values| {
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
            if let (Some(constant_args), Some(return_type)) = (constant_args, return_type) {
                let field = constant_args[0];
                let radix = constant_args[1].to_u128() as u32;
                let Type::Array(_, limb_count) = return_type else {
                    unreachable!("ICE: Intrinsic::ToRadix return type must be array")
                };
                simplify_constant_to_radix(endian, field, radix, limb_count.0, |values| {
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
            let length = match *dfg.type_of_value(arguments[0]) {
                Type::Array(_, length) => {
                    dfg.make_constant(FieldElement::from(length.0), NumericType::length_type())
                }
                Type::Numeric(NumericType::Unsigned { bit_size: 32 }) => {
                    if !matches!(*dfg.type_of_value(arguments[1]), Type::Vector(_)) {
                        bail_malformed!(
                            dfg,
                            "ArrayLen of a u32 length expects a vector second argument, got {:?}",
                            dfg.type_of_value(arguments[1])
                        );
                    }
                    arguments[0]
                }
                _ => bail_malformed!(
                    dfg,
                    "ArrayLen first argument must be an array or a vector length, got {:?}",
                    dfg.type_of_value(arguments[0])
                ),
            };
            SimplifyResult::SimplifiedTo(length)
        }
        // Strings are already arrays of bytes in SSA
        Intrinsic::ArrayAsStrUnchecked => SimplifyResult::SimplifiedTo(arguments[0]),
        Intrinsic::AsVector => {
            if let Some(result) =
                simplify_as_vector_for_zero_sized_vector(arguments, dfg, block, call_stack)
            {
                return result;
            }

            if let Some((array, array_type)) = dfg.get_array_constant(arguments[0]) {
                // Compute the resulting vector length
                let inner_element_types = array_type.element_types();
                let Some(vector_length_value) = dfg.try_get_vector_capacity(arguments[0]) else {
                    bail_malformed!(
                        dfg,
                        "AsVector argument has no vector capacity, got {:?}",
                        dfg.type_of_value(arguments[0])
                    );
                };
                let vector_length =
                    dfg.make_constant(vector_length_value.0.into(), NumericType::length_type());
                let new_vector =
                    make_array(dfg, array, Type::Vector(inner_element_types), block, call_stack);
                return SimplifyResult::SimplifiedToMultiple(vec![vector_length, new_vector]);
            }

            // In ACIR we can simplify `as_vector(array)`, to:
            //
            // ```
            // v0 = array_get array, index u32 0 -> T
            // v1 = array_get array, index u32 1 -> T
            // ...
            // vN = make_array [v0, v1, ...]
            // ```
            //
            // We don't do this for Brillig because it sometimes leads to more opcodes.
            if !dfg.runtime().is_acir() {
                return SimplifyResult::None;
            }

            let array_type = dfg.type_of_value(arguments[0]);
            let Type::Array(element_types, length) = array_type.as_ref() else {
                panic!("Expected as_vector input to be an array")
            };
            let element_types = element_types.clone();
            let length = *length;

            let mut elements = im::Vector::default();
            let mut index: u32 = 0;
            for _ in 0..length.0 {
                for element_type in element_types.iter() {
                    let index_value = dfg.make_constant(index.into(), NumericType::length_type());
                    let array_get =
                        Instruction::ArrayGet { array: arguments[0], index: index_value };
                    let element = dfg
                        .insert_instruction_and_results(
                            array_get,
                            block,
                            Some(vec![element_type.clone()]),
                            call_stack,
                        )
                        .first();
                    elements.push_back(element);
                    index += 1;
                }
            }
            let new_vector =
                make_array(dfg, elements, Type::Vector(element_types), block, call_stack);
            let vector_length = dfg.make_constant(length.0.into(), NumericType::length_type());
            SimplifyResult::SimplifiedToMultiple(vec![vector_length, new_vector])
        }
        Intrinsic::VectorPushBack => {
            if let Some(result) = simplify_vector_push_back_or_front_for_zero_sized_vector(
                arguments, dfg, block, call_stack,
            ) {
                return result;
            }

            let vector = dfg.get_array_constant(arguments[1]);
            if let Some((mut vector, vector_type)) = vector {
                if let Some(IntegerConstant::Unsigned { value: vector_len, .. }) =
                    dfg.get_integer_constant(arguments[0])
                {
                    let elements_size = vector_type.element_size();
                    let semi_flattened_vector_len =
                        SemanticLength(vector_len as u32) * elements_size;

                    // This simplification, which push back directly on the vector, only works if the real vector_len is the
                    // the length of the vector (taking the elements size into account).
                    if semi_flattened_vector_len == SemiFlattenedLength(vector.len() as u32) {
                        // Old code before implementing multiple vector mergers
                        for elem in &arguments[2..] {
                            vector.push_back(*elem);
                        }

                        let new_vector_length =
                            increment_vector_length(arguments[0], dfg, block, call_stack);

                        let new_vector = make_array(dfg, vector, vector_type, block, call_stack);
                        return SimplifyResult::SimplifiedToMultiple(vec![
                            new_vector_length,
                            new_vector,
                        ]);
                    }
                }

                simplify_vector_push_back(vector, vector_type, arguments, dfg, block, call_stack)
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::VectorPushFront => {
            if let Some(result) = simplify_vector_push_back_or_front_for_zero_sized_vector(
                arguments, dfg, block, call_stack,
            ) {
                return result;
            }

            let vector = dfg.get_array_constant(arguments[1]);
            if let Some((mut vector, vector_type)) = vector {
                for elem in arguments[2..].iter().rev() {
                    vector.push_front(*elem);
                }

                let new_vector_length =
                    increment_vector_length(arguments[0], dfg, block, call_stack);

                let new_vector = make_array(dfg, vector, vector_type, block, call_stack);
                SimplifyResult::SimplifiedToMultiple(vec![new_vector_length, new_vector])
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::VectorPopBack => {
            if let Some(result) = simplify_vector_pop_back_or_front_for_zero_sized_vector(
                arguments, dfg, block, call_stack,
            ) {
                return result;
            }

            let length = dfg.get_numeric_constant(arguments[0]);
            if length.is_none_or(|length| length.is_zero()) {
                // If the length is zero then we're trying to pop the last element from an empty vector.
                // Defer the error to acir_gen.
                return SimplifyResult::None;
            }

            let vector = dfg.get_array_constant(arguments[1]);
            if let Some((vector, typ)) = vector {
                simplify_vector_pop_back(vector, typ, arguments, dfg, block, call_stack)
            } else {
                SimplifyResult::None
            }
        }
        Intrinsic::VectorPopFront => {
            if let Some(result) = simplify_vector_pop_back_or_front_for_zero_sized_vector(
                arguments, dfg, block, call_stack,
            ) {
                return result;
            }

            let length = dfg.get_numeric_constant(arguments[0]);
            if length.is_none_or(|length| length.is_zero()) {
                // If the length is zero then we're trying to pop the first element from an empty vector.
                // Defer the error to acir_gen.
                return SimplifyResult::None;
            }

            let vector = dfg.get_array_constant(arguments[1]);
            if let Some((mut vector, typ)) = vector {
                let element_count = typ.element_size();

                if vector.len() < element_count.to_usize() {
                    bail_malformed!(
                        dfg,
                        "VectorPopFront: vector has {} elements, fewer than its element size {}",
                        vector.len(),
                        element_count.to_usize()
                    );
                }

                // We must pop multiple elements in the case of a vector of tuples
                let mut results = vecmap(0..element_count.to_usize(), |_| {
                    vector.pop_front().expect("vector length checked against element size above")
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
            if let Some(result) =
                simplify_vector_insert_for_zero_sized_vector(arguments, dfg, block, call_stack)
            {
                return result;
            }

            let vector = dfg.get_array_constant(arguments[1]);
            let index = dfg.get_numeric_constant(arguments[2]);
            if let (Some((mut vector, typ)), Some(index)) = (vector, index) {
                let elements = &arguments[3..];
                let start = index.to_u128() as usize * elements.len();

                // Do not simplify if the index is greater than the vector capacity
                // or else we will panic inside of the im::Vector insert method
                // Constraints should be generated during SSA gen to tell the user
                // they are attempting to insert at too large of an index
                if start > vector.len() {
                    return SimplifyResult::None;
                }

                for (offset, elem) in elements.iter().enumerate() {
                    vector.insert(start + offset, *elem);
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
            if let Some(result) =
                simplify_vector_remove_for_zero_sized_vector(arguments, dfg, block, call_stack)
            {
                return result;
            }

            let length = dfg.get_numeric_constant(arguments[0]);
            if length.is_none_or(|length| length.is_zero()) {
                // If the length is zero then we're trying to remove an element from an empty vector.
                // Defer the error to acir_gen.
                return SimplifyResult::None;
            }

            let vector = dfg.get_array_constant(arguments[1]);
            let index = dfg.get_numeric_constant(arguments[2]);
            if let (Some((mut vector, typ)), Some(index)) = (vector, index) {
                let element_count = typ.element_size().to_usize();
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
                bail_malformed!(
                    dfg,
                    "static_assert expects at least 2 arguments, got {}",
                    arguments.len()
                );
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
            let Some(max_bit_size) = dfg.get_numeric_constant(arguments[1]) else {
                bail_malformed!(dfg, "ApplyRangeConstraint bit-size must be a numeric constant");
            };
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
            if let Some(Type::Array(_, len)) = return_type {
                simplify_derive_generators(dfg, arguments, len.0, block, call_stack)
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
            dfg.type_of_value(*result).as_ref(),
            expected_types,
            "Simplification should not alter return type"
        );
    }

    simplified_result
}

fn simplify_as_vector_for_zero_sized_vector(
    arguments: &[ValueId],
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    call_stack: CallStackId,
) -> Option<SimplifyResult> {
    let array_type = dfg.type_of_value(arguments[0]);
    let Type::Array(element_types, length) = array_type.as_ref() else {
        bail_malformed!(
            @ret Some(SimplifyResult::None);
            dfg,
            "AsVector expects an array argument, got {array_type:?}"
        );
    };
    if !element_types.is_empty() {
        return None;
    }
    // If this is a zero-sized arrays it can never have values in it, so we can always simplify
    // it to (length, @[])
    let element_types = element_types.clone();
    let vector_length = dfg.make_constant(length.0.into(), NumericType::length_type());
    let new_vector =
        make_array(dfg, im::Vector::new(), Type::Vector(element_types), block, call_stack);
    Some(SimplifyResult::SimplifiedToMultiple(vec![vector_length, new_vector]))
}

/// Guard for the `*_for_zero_sized_vector` simplifications: succeeds only when the vector argument
/// has zero-sized elements (e.g. `[()]`), the case those simplifications handle.
///
/// Vector intrinsics take the length as `arguments[0]` and the vector as `arguments[1]`; on success
/// those two values are returned as `(length, vector)`. Returns `None` to defer to the general
/// handling when the elements are not zero-sized. Bails as malformed if the argument is not a
/// vector type at all, which cannot arise from well-formed SSA.
fn zero_sized_vector_length_and_value(
    arguments: &[ValueId],
    dfg: &DataFlowGraph,
) -> Option<(ValueId, ValueId)> {
    let length = arguments[0];
    let vector = arguments[1];
    let vector_type = dfg.type_of_value(vector);
    let Type::Vector(element_types) = vector_type.as_ref() else {
        bail_malformed!(
            @ret None;
            dfg,
            "vector intrinsic expects a vector argument, got {vector_type:?}"
        );
    };
    element_types.is_empty().then_some((length, vector))
}

/// Simplify a push back/front on a vector whose elements are zero-sized (e.g. `[()]`).
///
/// Zero-sized elements carry no data, so the backing array is always empty and the push only needs
/// to increment the semantic length. Returns `None` (deferring to the general handling) if the
/// element type is not zero-sized.
fn simplify_vector_push_back_or_front_for_zero_sized_vector(
    arguments: &[ValueId],
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    call_stack: CallStackId,
) -> Option<SimplifyResult> {
    let (length, vector) = zero_sized_vector_length_and_value(arguments, dfg)?;

    // If this is a zero-sized vector then it can never have values in it, so we can just
    // return an incremented length and return the same vector.
    let new_vector_length = increment_vector_length(length, dfg, block, call_stack);
    Some(SimplifyResult::SimplifiedToMultiple(vec![new_vector_length, vector]))
}

/// Insert `constrain length != 0`, failing with `message`, so that a subsequent unchecked decrement
/// of a zero-sized vector's length cannot underflow into a wrapped value.
fn constrain_vector_not_empty(
    length: ValueId,
    message: &str,
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    call_stack: CallStackId,
) {
    let zero_u32 = dfg.make_constant(FieldElement::zero(), NumericType::length_type());
    let length_eq_zero = dfg
        .insert_instruction_and_results(
            Instruction::Binary(Binary { lhs: length, operator: BinaryOp::Eq, rhs: zero_u32 }),
            block,
            None,
            call_stack,
        )
        .first();
    let false_value = dfg.make_constant(FieldElement::zero(), NumericType::bool());
    let message = Some(ConstrainError::StaticString(message.into()));
    dfg.insert_instruction_and_results(
        Instruction::Constrain(length_eq_zero, false_value, message),
        block,
        None,
        call_stack,
    );
}

/// Decrement a zero-sized vector's length, returning the `[new_length, vector]` results.
///
/// When the length is a statically-known zero the caller's empty-vector guard always fails, so the
/// decrement is skipped to avoid emitting an underflowing `unchecked_sub`; the returned length is
/// irrelevant on that trapping path.
fn decrement_zero_sized_vector_length(
    length: ValueId,
    vector: ValueId,
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    call_stack: CallStackId,
) -> SimplifyResult {
    let new_length = if dfg.get_numeric_constant(length).is_some_and(|len| len.is_zero()) {
        length
    } else {
        decrement_vector_length(length, dfg, block, call_stack)
    };
    SimplifyResult::SimplifiedToMultiple(vec![new_length, vector])
}

/// Simplify a pop back/front on a vector whose elements are zero-sized (e.g. `[()]`).
///
/// The backing array is always empty, so the pop only changes the semantic length. A guard against
/// popping from an empty vector is inserted so the unchecked length decrement cannot wrap; we emit
/// it here rather than relying on a check inserted elsewhere, so the simplification is self-contained
/// for any well-typed SSA. Returns `None` (deferring to the general handling) if the element type is
/// not zero-sized.
fn simplify_vector_pop_back_or_front_for_zero_sized_vector(
    arguments: &[ValueId],
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    call_stack: CallStackId,
) -> Option<SimplifyResult> {
    let (length, vector) = zero_sized_vector_length_and_value(arguments, dfg)?;
    constrain_vector_not_empty(length, "Cannot pop from an empty vector", dfg, block, call_stack);
    Some(decrement_zero_sized_vector_length(length, vector, dfg, block, call_stack))
}

/// Simplify a `vector_insert` on a vector whose elements are zero-sized (e.g. `[()]`).
///
/// The backing array is always empty, so the insert only increments the semantic length (the
/// in-bounds check is already emitted in `FunctionContext::codegen_intrinsic_call_checks`). Returns
/// `None` (deferring to the general handling) if the element type is not zero-sized.
fn simplify_vector_insert_for_zero_sized_vector(
    arguments: &[ValueId],
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    call_stack: CallStackId,
) -> Option<SimplifyResult> {
    let (length, vector) = zero_sized_vector_length_and_value(arguments, dfg)?;

    // If this is a zero-sized vector we would need to check if the index is in bounds.
    // However, this was already done in FunctionContext::codegen_intrinsic_call_checks so there's
    // no need to repeat that here.
    let new_vector_length = increment_vector_length(length, dfg, block, call_stack);

    Some(SimplifyResult::SimplifiedToMultiple(vec![new_vector_length, vector]))
}

/// Simplify a `vector_remove` on a vector whose elements are zero-sized (e.g. `[()]`).
///
/// The backing array is always empty, so the remove only changes the semantic length. A guard
/// against removing from an empty vector is inserted so the unchecked length decrement cannot wrap;
/// we emit it here rather than relying on the frontend's access check, which is absent from
/// directly-constructed SSA. Returns `None` (deferring to the general handling) if the element type
/// is not zero-sized.
fn simplify_vector_remove_for_zero_sized_vector(
    arguments: &[ValueId],
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    call_stack: CallStackId,
) -> Option<SimplifyResult> {
    let (length, vector) = zero_sized_vector_length_and_value(arguments, dfg)?;
    constrain_vector_not_empty(
        length,
        "Cannot remove from an empty vector",
        dfg,
        block,
        call_stack,
    );
    Some(decrement_zero_sized_vector_length(length, vector, dfg, block, call_stack))
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
    // `BigUint::to_radix_le` represents zero as a single zero limb (`[0]`), which would make a
    // zero value appear to require one limb. Decomposing zero requires no significant limbs, so
    // treat it as an empty decomposition (zero-padded up to `limb_count`).
    let decomposed_integer = if field.is_zero() {
        Vec::new()
    } else {
        let big_integer = BigUint::from_bytes_be(&field.to_be_bytes());
        // Decompose the integer into its radix digits in little endian form.
        big_integer.to_radix_le(radix)
    };
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

    let typ = Type::Array(
        Arc::new(vec![Type::Numeric(typ)]),
        SemanticLength(assert_u32(result_constants.len())),
    );
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
    // The subtraction is unchecked because every caller reaches this point only once the length is
    // known to be non-zero: either it is a known non-zero constant (`simplify_vector_pop_back`), or
    // a bounds check guaranteeing a non-empty vector has already been emitted (the zero-sized
    // pop/remove paths, which run for dynamic lengths too). In well-formed SSA it cannot underflow.
    update_vector_length(vector_len, dfg, BinaryOp::Sub { unchecked: true }, block, call_stack)
}

/// Simplify a vector push back when the length is not known to equal capacity, ie. we don't
/// know whether we to push new items and grow the capacity of the vector, or overwrite the
/// next padding item.
///
/// The strategy is to:
/// 1. Create a new vector where the new item is pushed to the end, extending its capacity
/// 2. Set the item at the original semantic length as well
///
/// There result is that the vector will physically always be extended by 1, with the pushed
/// item appearing at the end, and potentially in the middle of the vector if we weren't at capacity.
fn simplify_vector_push_back(
    mut vector: im::Vector<ValueId>,
    element_type: Type,
    arguments: &[ValueId],
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    call_stack: CallStackId,
) -> SimplifyResult {
    // TODO(#2752): We need to handle the element_type size to appropriately handle vectors of complex types.
    // This is reliant on dynamic indices of non-homogenous vectors also being implemented.
    if element_type.element_size() != ElementTypesLength(1) {
        return SimplifyResult::None;
    }
    if arguments.len() != 3 {
        bail_malformed!(dfg, "vector push expects 3 arguments, got {}", arguments.len());
    }

    let new_vector_length = increment_vector_length(arguments[0], dfg, block, call_stack);

    vector.push_back(arguments[2]);

    let extended_vector = make_array(dfg, vector, element_type, block, call_stack);

    // Set the value at the semantic length: if the vector had extra capacity, this will set the first
    // padding to the item we wanted to push. By doing this on the extended vector, we guarantee that
    // there will be extra capacity. If we tried to do this on the original, we could get Index OOB if
    // the capacity and the size were the same.
    let set_last_vector_instr = Instruction::ArraySet {
        array: extended_vector,
        index: arguments[0],
        value: arguments[2],
        mutable: false,
    };

    let set_last_vector =
        dfg.insert_instruction_and_results(set_last_vector_instr, block, None, call_stack).first();

    SimplifyResult::SimplifiedToMultiple(vec![new_vector_length, set_last_vector])
}

/// Simplify a `vector_pop_back` whose backing array is a known constant.
///
/// Decrements the semantic length and reads the popped element(s) off the end of the flattened
/// array, returning `[new_length, new_vector, popped_elements..]`. A vector of tuples pops several
/// flattened slots per element, so the elements are read in reverse from the tail.
fn simplify_vector_pop_back(
    mut vector: im::Vector<ValueId>,
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
        vector.pop_back();
    }

    let new_vector = make_array(dfg, vector, vector_type, block, call_stack);
    results.push_front(new_vector);

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
    cfg_if::cfg_if! {
        if #[cfg(feature = "bn254")] {
            let solver = bn254_blackbox_solver::Bn254BlackBoxSolver;
        } else {
            let solver = acvm::blackbox_solver::StubbedBlackBoxSolver;
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

                    let input: [u64; 25] = match const_input.try_into() {
                        Ok(input) => input,
                        Err(input) => bail_malformed!(
                            dfg,
                            "keccakf1600 input: expected length 25, got {}",
                            input.len()
                        ),
                    };
                    let state = acvm::blackbox_solver::keccakf1600(input)
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
            let Some(domain_separator_bytes): Option<Vec<u8>> = domain_separator_string
                .0
                .iter()
                .map(|&x| dfg.get_numeric_constant(x).map(|c| c.to_u128() as u8))
                .collect()
            else {
                bail_malformed!(dfg, "derive_generators domain separator must be constant bytes");
            };
            let Some(starting_index) = starting_index.try_to_u32() else {
                bail_malformed!(dfg, "derive_generators starting_index must fit in a u32");
            };
            let generators =
                derive_generators(&domain_separator_bytes, num_generators, starting_index);
            let mut results = Vec::new();
            for generator in generators {
                let x = FieldElement::from_repr(generator.x);
                let y = FieldElement::from_repr(generator.y);
                results.push(dfg.make_constant(x, NumericType::NativeField));
                results.push(dfg.make_constant(y, NumericType::NativeField));
            }
            let len = results.len() as u32;
            assert!(
                len.is_multiple_of(2),
                "The number of results from derive_generators must be a multiple of 2"
            );
            let typ =
                Type::Array(vec![Type::field(), Type::field()].into(), SemanticLength(len / 2));
            let result = make_array(dfg, results.into(), typ, block, call_stack);
            SimplifyResult::SimplifiedTo(result)
        } else {
            SimplifyResult::None
        }
    } else {
        bail_malformed!(dfg, "derive_generators expects 2 arguments, got {}", arguments.len());
    }
}

#[cfg(test)]
mod tests {
    use acvm::{AcirField, FieldElement};

    use crate::ssa::ir::instruction::Endian;
    use crate::{
        assert_ssa_snapshot,
        ssa::{
            Ssa, ir::dfg::simplify::call::constant_to_radix,
            opt::assert_ssa_does_not_change_after_simplifying,
        },
    };

    #[test]
    fn constant_to_radix_decomposes_zero_into_zero_limbs() {
        let limbs = constant_to_radix(Endian::Little, FieldElement::zero(), 256, 0);
        assert_eq!(limbs, Some(Vec::new()));
    }

    #[test]
    fn constant_to_radix_zero_pads_zero_value() {
        let limbs = constant_to_radix(Endian::Little, FieldElement::zero(), 256, 3);
        assert_eq!(limbs, Some(vec![FieldElement::zero(); 3]));
    }

    #[test]
    fn constant_to_radix_rejects_non_zero_value_with_zero_limbs() {
        let limbs = constant_to_radix(Endian::Little, FieldElement::from(5u128), 256, 0);
        assert_eq!(limbs, None);
    }

    #[test]
    fn simplify_derive_generators_has_correct_type() {
        let src = r#"
            brillig(inline) fn main func {
              block():
                separator = make_array b"DEFAULT_DOMAIN_SEPARATOR"

                // This call was previously incorrectly simplified to something that returned `[Field; 3]`
                result = call derive_pedersen_generators(separator, u32 0) -> [(Field, Field); 1]

                return result
            }
            "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn main f0 {
          b0():
            v15 = make_array b"DEFAULT_DOMAIN_SEPARATOR"
            v18 = make_array [Field 3728882899078719075161482178784387565366481897740339799480980287259621149274, Field -9903063709032878667290627648209915537972247634463802596148419711785767431332] : [(Field, Field); 1]
            return v18
        }
        "#);
    }

    // `keccakf1600` operates on a fixed `[u64; 25]` state; an all-constant call with a different
    // length is malformed SSA, which panics under the default strict simplification.
    #[test]
    #[should_panic(expected = "malformed SSA reached simplify")]
    fn wrong_sized_keccakf1600_panics_under_strict_simplify() {
        let state = vec!["u64 0"; 24].join(", ");
        let src = format!(
            r#"
            acir(inline) fn main f0 {{
              b0():
                v0 = make_array [{state}] : [u64; 24]
                v1 = call keccakf1600(v0) -> [u64; 25]
                return v1
            }}"#
        );
        let _ = Ssa::from_str_simplifying(&src);
    }

    // With `allow_malformed_simplify` enabled (as the `ssa_fuzzer` does), the same malformed call is
    // left intact rather than panicking. Validation is skipped because it would reject the length too.
    #[test]
    fn wrong_sized_keccakf1600_is_left_intact_when_malformed_allowed() {
        let state = vec!["u64 0"; 24].join(", ");
        let src = format!(
            r#"
            acir(inline) fn main f0 {{
              b0():
                v0 = make_array [{state}] : [u64; 24]
                v1 = call keccakf1600(v0) -> [u64; 25]
                return v1
            }}"#
        );
        let ssa = Ssa::from_str_impl(&src, true, false, true).unwrap();
        let lowered = ssa.to_string();
        assert!(
            lowered.contains("call keccakf1600"),
            "a malformed call must be left intact under allow_malformed_simplify, got:\n{lowered}"
        );
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

    #[should_panic(
        expected = "malformed SSA reached simplify: ArrayLen first argument must be an array or a vector length"
    )]
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
    fn simplifies_as_vector_for_known_array() {
        let src = r#"
        acir(inline) fn main func {
          b0():
            v0 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
            v1, v2 = call as_vector(v0) -> (u32, [Field])
            return v1, v2
        }
        "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v3 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
            v4 = make_array [Field 1, Field 2, Field 3] : [Field]
            return u32 3, v4
        }
        ");
    }

    #[test]
    fn simplifies_as_vector_for_unknown_array_in_acir() {
        let src = r#"
        acir(inline) fn main func {
          b0(v0: [Field; 3]):
            v1, v2 = call as_vector(v0) -> (u32, [Field])
            return v1, v2
        }
        "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v2 = array_get v0, index u32 0 -> Field
            v4 = array_get v0, index u32 1 -> Field
            v6 = array_get v0, index u32 2 -> Field
            v7 = make_array [v2, v4, v6] : [Field]
            return u32 3, v7
        }
        ");
    }

    #[test]
    fn does_not_simplify_as_vector_for_unknown_array_in_brillig() {
        let src = r#"
        brillig(inline) fn main func {
          b0(v0: [Field; 3]):
            v1, v2 = call as_vector(v0) -> (u32, [Field])
            return v1, v2
        }
        "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v2, v3 = call as_vector(v0) -> (u32, [Field])
            return v2, v3
        }
        ");
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

    #[test]
    fn simplifies_vector_push_back_from_make_array_simple() {
        let src = r#"
        acir(inline) fn main func {
          b0():
            v0 = make_array [Field 1, Field 2] : [Field]
            v2, v3 = call vector_push_back(u32 2, v0, Field 3) -> (u32, [Field])
            return v2, v3
        }
        "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v2 = make_array [Field 1, Field 2] : [Field]
            v4 = make_array [Field 1, Field 2, Field 3] : [Field]
            return u32 3, v4
        }
        ");
    }

    #[test]
    fn simplifies_vector_push_back_from_make_array_complex() {
        let src = r#"
        acir(inline) fn main func {
          b0():
            v0 = make_array [Field 1, Field 2, Field 3, Field 4] : [(Field, Field)]
            v2, v3 = call vector_push_back(u32 2, v0, Field 5, Field 6) -> (u32, [(Field, Field)])
            return v2, v3
        }
        "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v4 = make_array [Field 1, Field 2, Field 3, Field 4] : [(Field, Field)]
            v7 = make_array [Field 1, Field 2, Field 3, Field 4, Field 5, Field 6] : [(Field, Field)]
            return u32 3, v7
        }
        ");
    }

    #[test]
    fn does_not_simplify_vector_push_back_from_make_array_if_length_different_from_capacity_and_complex()
     {
        // Here the semantic length is different from the vector capacity.
        // A situation like this is possible when we merge vectors of different length across different branches,
        // which results in the ValueMerger allocating elements to hold the longer one, and the semantic length
        // becoming a formula. Then, if constant folding with Brillig optimizes out the condition, the semantic
        // length can become a known constant.
        // At the moment the only handling for complex type is the pushing to the last position.
        let src = r#"
        acir(inline) fn main func {
          b0():
            v0 = make_array [Field 1, Field 2, Field 3, Field 4] : [(Field, Field)]
            v2, v3 = call vector_push_back(u32 1, v0, Field 5, Field 6) -> (u32, [(Field, Field)])
            return v2, v3
        }
        "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v4 = make_array [Field 1, Field 2, Field 3, Field 4] : [(Field, Field)]
            v9, v10 = call vector_push_back(u32 1, v4, Field 5, Field 6) -> (u32, [(Field, Field)])
            return v9, v10
        }
        ");
    }

    #[test]
    fn simplify_vector_push_back_from_make_array_if_length_different_from_capacity_and_simple() {
        // Here the semantic length is different from the vector capacity, but the elements are simple.
        // In this case we can do a merge strategy.
        let src = r#"
        acir(inline) fn main func {
          b0():
            v0 = make_array [Field 1, Field 2, Field 3] : [Field]
            v2, v3 = call vector_push_back(u32 1, v0, Field 5) -> (u32, [(Field, Field)])
            return v2, v3
        }
        "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v3 = make_array [Field 1, Field 2, Field 3] : [Field]
            v5 = make_array [Field 1, Field 2, Field 3, Field 5] : [Field]
            v7 = array_set v5, index u32 1, value Field 5
            return u32 2, v7
        }
        ");
    }

    #[test]
    fn simplifies_vector_push_back_with_unknown_length() {
        let src = r#"
        acir(inline) fn main func {
          b0(v0: u32):
            v1 = make_array [Field 3, Field 4] : [Field]
            v2, v3 = call vector_push_back(v0, v1, Field 5) -> (u32, [Field])
            return v2, v3
        }
        "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        // We can see how we start with a `make_array` that pushed `Field 5` to the end of the
        // original `make_array`, sets the element at `v0` to `Field 5` (which can result in
        // one of `[5, 4, 5]`, `[3, 5, 5]` or `[3, 4, 5]` depending on the value of `v0`),
        // and then merge that new array with `[3, 4, 5]`.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v3 = make_array [Field 3, Field 4] : [Field]
            v5 = add v0, u32 1
            v7 = make_array [Field 3, Field 4, Field 5] : [Field]
            v8 = array_set v7, index v0, value Field 5
            return v5, v8
        }
        ");
    }

    #[test]
    fn simplifies_vector_insert_on_make_array_and_known_middle_index() {
        let src = r#"
        acir(inline) fn main func {
          b0(v0: u32):
            v1 = make_array [Field 3, Field 4] : [Field]
            v10, v11 = call vector_insert(u32 2, v1, u32 1, Field 2) -> (u32, [Field])
            return v10, v11
        }
        "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v3 = make_array [Field 3, Field 4] : [Field]
            v5 = make_array [Field 3, Field 2, Field 4] : [Field]
            return u32 3, v5
        }
        ");
    }

    #[test]
    fn simplifies_vector_insert_on_make_array_and_known_index_right_past_end() {
        let src = r#"
        acir(inline) fn main func {
          b0(v0: u32):
            v1 = make_array [Field 3, Field 4] : [Field]
            v10, v11 = call vector_insert(u32 2, v1, u32 2, Field 2) -> (u32, [Field])
            return v10, v11
        }
        "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v3 = make_array [Field 3, Field 4] : [Field]
            v5 = make_array [Field 3, Field 4, Field 2] : [Field]
            return u32 3, v5
        }
        ");
    }

    #[test]
    fn does_not_simplify_vector_insert_on_make_array_and_known_index_past_end() {
        let src = r#"
        acir(inline) fn main func {
          b0(v0: u32):
            v1 = make_array [Field 3, Field 4] : [Field]
            v10, v11 = call vector_insert(u32 2, v1, u32 3, Field 2) -> (u32, [Field])
            return v10, v11
        }
        "#;
        assert_ssa_does_not_change_after_simplifying(src);
    }

    #[test]
    fn simplifies_as_vector_for_zero_sized_array() {
        let src = r"
        acir(inline) fn main func {
          b0(v0: [(); 3]):
            v1, v2 = call as_vector(v0) -> [()]
            return v1, v2
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [(); 3]):
            v1 = make_array [] : [()]
            return u32 3, v1
        }
        ");
    }

    #[test]
    fn simplifies_vector_insert_for_zero_sized_array() {
        let src = r"
        acir(inline) fn main func {
          b0(v0: u32, v1: [()], v2: u32):
            v3, v4 = call vector_insert(v0, v1, v2) -> (u32, [()])
            return v3, v4
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32, v1: [()], v2: u32):
            v4 = add v0, u32 1
            return v4, v1
        }
        ");
    }

    #[test]
    fn simplifies_vector_remove_for_zero_sized_array() {
        let src = r"
        acir(inline) fn main func {
          b0(v0: u32, v1: [()], v2: u32):
            v3, v4 = call vector_remove(v0, v1, v2) -> (u32, [()])
            return v3, v4
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: u32, v1: [()], v2: u32):
            v4 = eq v0, u32 0
            constrain v4 == u1 0, "Cannot remove from an empty vector"
            v7 = unchecked_sub v0, u32 1
            return v7, v1
        }
        "#);
    }

    #[test]
    fn simplifies_vector_remove_for_empty_zero_sized_array() {
        // Removing from a statically-empty zero-sized vector must not produce a wrapping length:
        // see https://github.com/noir-lang/noir/issues/1394.
        let src = r"
        acir(inline) fn main func {
          b0():
            v0 = make_array [] : [()]
            v1, v2 = call vector_remove(u32 0, v0, u32 0) -> (u32, [()])
            return v1
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [] : [()]
            constrain u1 1 == u1 0, "Cannot remove from an empty vector"
            return u32 0
        }
        "#);
    }

    #[test]
    fn simplifies_vector_push_back_for_zero_sized_array() {
        let src = r"
        acir(inline) fn main func {
          b0(v0: u32, v1: [()]):
            v2, v3 = call vector_push_back(v0, v1) -> (u32, [()])
            return v2, v3
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32, v1: [()]):
            v3 = add v0, u32 1
            return v3, v1
        }
        ");
    }

    #[test]
    fn simplifies_vector_push_front_for_zero_sized_array() {
        let src = r"
        acir(inline) fn main func {
          b0(v0: u32, v1: [()]):
            v2, v3 = call vector_push_front(v0, v1) -> (u32, [()])
            return v2, v3
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32, v1: [()]):
            v3 = add v0, u32 1
            return v3, v1
        }
        ");
    }

    #[test]
    fn simplifies_vector_pop_front_for_zero_sized_array_in_acir() {
        let src = r"
        acir(inline) fn main func {
          b0(v0: u32, v1: [()]):
            v2, v3 = call vector_pop_front(v0, v1) -> (u32, [()])
            return v2, v3
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: u32, v1: [()]):
            v3 = eq v0, u32 0
            constrain v3 == u1 0, "Cannot pop from an empty vector"
            v6 = unchecked_sub v0, u32 1
            return v6, v1
        }
        "#);
    }

    #[test]
    fn simplifies_vector_pop_back_for_zero_sized_array_in_acir() {
        let src = r"
        acir(inline) fn main func {
          b0(v0: u32, v1: [()]):
            v2, v3 = call vector_pop_back(v0, v1) -> (u32, [()])
            return v2, v3
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: u32, v1: [()]):
            v3 = eq v0, u32 0
            constrain v3 == u1 0, "Cannot pop from an empty vector"
            v6 = unchecked_sub v0, u32 1
            return v6, v1
        }
        "#);
    }

    #[test]
    fn simplifies_vector_pop_front_for_zero_sized_array_in_brillig() {
        let src = r"
        brillig(inline) fn main func {
          b0(v0: u32, v1: [()]):
            v2, v3 = call vector_pop_front(v0, v1) -> (u32, [()])
            return v2, v3
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();
        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: [()]):
            v3 = eq v0, u32 0
            constrain v3 == u1 0, "Cannot pop from an empty vector"
            v6 = unchecked_sub v0, u32 1
            return v6, v1
        }
        "#);
    }

    #[test]
    fn simplifies_vector_pop_back_for_zero_sized_array_in_brillig() {
        let src = r"
        brillig(inline) fn main func {
          b0(v0: u32, v1: [()]):
            v2, v3 = call vector_pop_back(v0, v1) -> (u32, [()])
            return v2, v3
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();
        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: [()]):
            v3 = eq v0, u32 0
            constrain v3 == u1 0, "Cannot pop from an empty vector"
            v6 = unchecked_sub v0, u32 1
            return v6, v1
        }
        "#);
    }
}
