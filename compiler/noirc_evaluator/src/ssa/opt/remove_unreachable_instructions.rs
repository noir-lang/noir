//! This module defines an SSA pass to remove instructions that are unreachable.
//! For example, if an instruction in a block is `constrain u1 0 == u1 1`,
//! any subsequent instructions in that block will never be executed. This pass
//! then removes those subsequent instructions and replaces the block's terminator
//! with a special `unreachable` value.
//!
//! This pass might also add constrain checks after existing instructions,
//! for example binary operations that are guaranteed to overflow.
//!
//! ## Handling of `constrain`
//!
//! Given an SSA like this:
//!
//! ```ssa
//! constrain u1 0 == u1 1
//! v1 = load v0 -> Field
//! return v1
//! ```
//!
//! Because the constrain is guaranteed to fail, every instruction after it is removed
//! and the terminator is replaced with `unreachable`:
//!
//! ```ssa
//! constrain u1 0 == u1 1
//! unreachable
//! ```
//!
//! Similarly, `constrain u1 0 != u1 0` will have the same treatment.
//!
//! ## Handling of binary operations
//!
//! If a binary operation is guaranteed to overflow or fail:
//!
//! ```ssa
//! v4 = add u8 254, u8 127 // guaranteed to overflow
//! v5 = add v4, v6
//! return v5
//! ```
//!
//! the operation is left in the SSA but a constrain failure is added after it,
//! and every subsequent instruction is removed:
//!
//! ```ssa
//! v4 = add u8 254, u8 127
//! constrain u1 0 == u1 1, "attempt to add with overflow"
//! unreachable
//! ```
//!
//! Division by zero is an operation that is guaranteed to fail, but it does not overflow.
//!
//! Because binary operations can depend on a side-effects variable (`enable_side_effects`),
//! the constrain will check the value of this variable whenever a non-constant side effect
//! is active. Every subsequent instruction is then replaced with a zeroed-value.
//! So for example this SSA:
//!
//! ```ssa
//! enable_side_effects v3
//! v4 = add u8 254, u8 127 // guaranteed to overflow
//! v5 = add v4, v6
//! return v5
//! ```
//!
//! is replaced with this one:
//!
//! ```ssa
//! enable_side_effects v3
//! v4 = add u8 254, u8 127 // guaranteed to overflow
//! constrain u1 0 == v3, "attempt to add with overflow"
//! return u8 0
//! ```
//!
//! ## Handling of array_get and array_set
//!
//! If an array operation in ACIR is guaranteed to produce an index-out-of-bounds:
//!
//! ```ssa
//! v0 = allocate -> &mut Field
//! v1 = make_array [v0] -> [&mut Field; 0]
//! v2 = array_get v1, index u32 0 -> Field
//! v3 = add v2, Field 2
//! return v3
//! ```
//!
//! the array operation is replaced with a similar operation but on an `u1` array.
//! The reason is that the original array might contain references and by doing this
//! there's no longer a need to keep track of those references:
//!
//! ```ssa
//! v0 = allocate -> &mut Field
//! v1 = make_array [v0] -> [&mut Field; 0]
//! v2 = make_array [u1 0] -> [u1; 0]
//! v3 = array_get v2, index u32 2 -> u1
//! unreachable
//! ```
//!
//! ## Handling of slice operations
//!
//! If a slice operation like `slice_push_back` or `slice_pop_front` in ACIR is guaranteed
//! to fail, which can only happen if the slice is empty, the operation is removed
//! and replaced with a constrain failure, then the returned slice is replaced with
//! an empty slice. So this SSA:
//!
//! ```ssa
//! v0 = make_array [] -> [u32]
//! v2, v3, v4 = call slice_pop_front(u32 0, v0) -> (u32, u32, [u32])
//! v5 = add v2, u32 1
//! return v4, v5
//! ```
//!
//! is replaced with this SSA:
//!
//! ```ssa
//! v0 = make_array [] -> [u32]
//! constrain u1 0 == u1 1, "Index out of bounds"
//! v1 = make_array [] -> [u32]
//! return v1, u32 0
//! ```
//!
//! ## Preconditions:
//! - the [inlining][`super::inlining`] and [flatten_cfg][`super::flatten_cfg`] must
//!   not run after this pass as they can't handle the `unreachable` terminator.
use std::sync::Arc;

use acvm::{AcirField, FieldElement};
use noirc_errors::call_stack::CallStackId;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        function::{Function, FunctionId},
        instruction::{
            Binary, BinaryOp, ConstrainError, Instruction, Intrinsic, TerminatorInstruction,
            binary::{BinaryEvaluationResult, eval_constant_binary_op},
        },
        types::{NumericType, Type},
        value::{Value, ValueId},
    },
    opt::simple_optimization::SimpleOptimizationContext,
    ssa_gen::Ssa,
};

impl Ssa {
    pub(crate) fn remove_unreachable_instructions(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.remove_unreachable_instructions();
        }
        self
    }
}

#[derive(Debug, PartialEq)]
enum Reachability {
    /// By default instructions are reachable.
    Reachable,
    /// We encountered an instruction that fails, but only under a predicate.
    /// Until we encounter a new predicate, the instructions should have no side effect.
    UnreachableUnderPredicate,
    /// We encountered an instruction that always fails.
    Unreachable,
}

impl Function {
    fn remove_unreachable_instructions(&mut self) {
        let func_id = self.id();
        // The current block we are currently processing
        let mut current_block_id = None;

        // Whether the current block instructions were determined to be unreachable
        // after an always failing one was found.
        let mut current_block_reachability = Reachability::Reachable;

        let one = self.dfg.make_constant(1_u32.into(), NumericType::bool());
        let mut side_effects_condition = one;

        self.simple_optimization(|context| {
            let block_id = context.block_id;

            if current_block_id != Some(block_id) {
                current_block_id = Some(block_id);
                current_block_reachability = Reachability::Reachable;
                side_effects_condition = one;
            }

            if current_block_reachability == Reachability::Unreachable {
                context.remove_current_instruction();
                return;
            }

            let instruction = context.instruction();
            if let Instruction::EnableSideEffectsIf { condition } = instruction {
                side_effects_condition = *condition;
                current_block_reachability =
                    match context.dfg.get_numeric_constant(side_effects_condition) {
                        Some(predicate) if predicate.is_zero() => {
                            // We can replace side effecting instructions with defaults until the next predicate.
                            Reachability::UnreachableUnderPredicate
                        }
                        _ => Reachability::Reachable,
                    };
                return;
            };

            if current_block_reachability == Reachability::UnreachableUnderPredicate {
                // Instructions that don't interact with the predicate should be left alone,
                // because the `remove_enable_side_effects` pass might have moved the boundaries around them.
                if !instruction.requires_acir_gen_predicate(context.dfg) {
                    return;
                }
                remove_and_replace_with_defaults(context, func_id, block_id);
                return;
            }

            // Check if the current predicate is known to be enabled.
            let is_predicate_constant_one =
                || match context.dfg.get_numeric_constant(side_effects_condition) {
                    Some(predicate) => predicate.is_one(),
                    None => false, // The predicate is a variable
                };

            match instruction {
                Instruction::Constrain(lhs, rhs, _) => {
                    let Some(lhs_constant) = context.dfg.get_numeric_constant(*lhs) else {
                        return;
                    };
                    let Some(rhs_constant) = context.dfg.get_numeric_constant(*rhs) else {
                        return;
                    };
                    if lhs_constant != rhs_constant {
                        current_block_reachability = Reachability::Unreachable;
                    }
                }
                Instruction::ConstrainNotEqual(lhs, rhs, _) => {
                    let Some(lhs_constant) = context.dfg.get_numeric_constant(*lhs) else {
                        return;
                    };
                    let Some(rhs_constant) = context.dfg.get_numeric_constant(*rhs) else {
                        return;
                    };
                    if lhs_constant == rhs_constant {
                        current_block_reachability = Reachability::Unreachable;
                    }
                }
                Instruction::Binary(binary @ Binary { lhs, operator, rhs }) => {
                    if let Some(message) =
                        binary_operation_always_fails(*lhs, *operator, *rhs, context)
                    {
                        // Check if this operation is one that should only fail if the predicate is enabled.
                        let requires_acir_gen_predicate =
                            binary.requires_acir_gen_predicate(context.dfg);

                        let fails_under_predicate =
                            requires_acir_gen_predicate && !is_predicate_constant_one();

                        // Insert the instruction right away so we can add a constrain immediately after it
                        context.insert_current_instruction();

                        // Insert a constraint which makes it easy to see that this instruction will fail.
                        let guard = if fails_under_predicate {
                            side_effects_condition
                        } else {
                            context.dfg.make_constant(1_u128.into(), NumericType::bool())
                        };
                        let zero = context.dfg.make_constant(0_u128.into(), NumericType::bool());
                        let message = Some(ConstrainError::StaticString(message));
                        let instruction = Instruction::Constrain(zero, guard, message);
                        let call_stack =
                            context.dfg.get_instruction_call_stack_id(context.instruction_id);

                        context.dfg.insert_instruction_and_results(
                            instruction,
                            block_id,
                            None,
                            call_stack,
                        );

                        // Subsequent instructions can either be removed, of replaced by defaults until the next predicate.
                        current_block_reachability = if fails_under_predicate {
                            Reachability::UnreachableUnderPredicate
                        } else {
                            Reachability::Unreachable
                        }
                    }
                }
                Instruction::ArrayGet { array, index, offset }
                | Instruction::ArraySet { array, index, offset, .. }
                    if context.dfg.runtime().is_acir() =>
                {
                    let array_or_slice_type = context.dfg.type_of_value(*array);
                    let array_op_always_fails = match &array_or_slice_type {
                        Type::Slice(_) => false,
                        array_type @ Type::Array(_, len) => {
                            *len == 0
                                || context.dfg.get_numeric_constant(*index).is_some_and(|index| {
                                    (index.try_to_u32().unwrap() - offset.to_u32())
                                        >= (array_type.element_size() as u32 * len)
                                })
                        }

                        _ => unreachable!(
                            "Encountered non-array type during array read/write operation"
                        ),
                    };

                    if array_op_always_fails {
                        current_block_reachability = if is_predicate_constant_one() {
                            // If we have an array that contains references we no longer need to bother with resolution of those references.
                            // However, we want a trap to still be triggered by an OOB array access.
                            // Thus, we can replace our array with dummy numerics to avoid unnecessary allocations
                            // making there way further down the compilation pipeline (e.g. ACIR where references are not supported).
                            let (old_instruction, old_array, trap_array) = match array_or_slice_type
                            {
                                Type::Array(_, len) => {
                                    let dummy_array_typ = Type::Array(
                                        Arc::new(vec![Type::Numeric(NumericType::unsigned(1))]),
                                        len,
                                    );
                                    (
                                        instruction.clone(),
                                        *array,
                                        zeroed_value(
                                            context.dfg,
                                            func_id,
                                            block_id,
                                            &dummy_array_typ,
                                        ),
                                    )
                                }
                                _ => unreachable!("Expected an array type"),
                            };
                            let new_instruction = old_instruction.map_values(|value| {
                                if value == old_array { trap_array } else { value }
                            });
                            let stack =
                                context.dfg.get_instruction_call_stack_id(context.instruction_id);
                            context.dfg.insert_instruction_and_results(
                                new_instruction,
                                block_id,
                                Some(vec![Type::Numeric(NumericType::unsigned(1))]),
                                stack,
                            );
                            // Remove the old failing array access in favor of the dummy one
                            context.remove_current_instruction();

                            Reachability::Unreachable
                        } else {
                            Reachability::UnreachableUnderPredicate
                        };
                    }
                }
                // Intrinsic Slice operations in ACIR on empty arrays need to be replaced with a (conditional) constraint.
                // In Brillig they will be protected by an access constraint, which, if known to fail, will make the block unreachable.
                Instruction::Call { func, arguments } if context.dfg.runtime().is_acir() => {
                    if let Value::Intrinsic(Intrinsic::SlicePopBack | Intrinsic::SlicePopFront) =
                        &context.dfg[*func]
                    {
                        let length = arguments.iter().next().unwrap_or_else(|| {
                            unreachable!("slice operations have 2 arguments: [length, slice]")
                        });
                        let is_empty =
                            context.dfg.get_numeric_constant(*length).is_some_and(|v| v.is_zero());
                        // If the compiler knows the slice is empty, there is no point trying to pop from it, we know it will fail.
                        // Barretenberg doesn't handle memory operations with predicates, so we can't rely on those to disable the operation
                        // based on the current side effect variable. Instead we need to replace it with a conditional constraint.
                        if is_empty {
                            let always_fail = is_predicate_constant_one();

                            // We might think that if the predicate is constant 1, we can leave the pop as it will always fail.
                            // However by turning the block Unreachable, ACIR-gen would create empty bytecode and not fail the circuit.
                            insert_constraint(
                                context,
                                block_id,
                                side_effects_condition,
                                "Index out of bounds".to_string(),
                            );

                            current_block_reachability = if always_fail {
                                context.remove_current_instruction();
                                Reachability::Unreachable
                            } else {
                                // Here we could use the empty slice as the replacement of the return value,
                                // except that slice operations also return the removed element and the new length
                                // so it's easier to just use zeroed values here
                                remove_and_replace_with_defaults(context, func_id, block_id);
                                Reachability::UnreachableUnderPredicate
                            };
                        }
                    }
                }
                _ => (),
            };

            if current_block_reachability == Reachability::Unreachable {
                let terminator = context.dfg[block_id].take_terminator();
                let call_stack = terminator.call_stack();
                context.dfg[block_id]
                    .set_terminator(TerminatorInstruction::Unreachable { call_stack });
            }
        });
    }
}

fn binary_operation_always_fails(
    lhs: ValueId,
    operator: BinaryOp,
    rhs: ValueId,
    context: &SimpleOptimizationContext,
) -> Option<String> {
    // Unchecked operations can never fail
    match operator {
        BinaryOp::Add { unchecked } | BinaryOp::Sub { unchecked } | BinaryOp::Mul { unchecked } => {
            if unchecked {
                return None;
            }
        }
        BinaryOp::Div
        | BinaryOp::Mod
        | BinaryOp::Eq
        | BinaryOp::Lt
        | BinaryOp::And
        | BinaryOp::Or
        | BinaryOp::Xor
        | BinaryOp::Shl
        | BinaryOp::Shr => (),
    };

    let rhs_value = context.dfg.get_numeric_constant(rhs)?;

    if matches!(operator, BinaryOp::Div) && rhs_value.is_zero() {
        return Some("attempt to divide by zero".to_string());
    }

    if matches!(operator, BinaryOp::Mod) && rhs_value.is_zero() {
        return Some("attempt to calculate the remainder with a divisor of zero".to_string());
    }

    let Type::Numeric(numeric_type) = context.dfg.type_of_value(lhs) else {
        panic!("Expected numeric type for binary operation");
    };

    let lhs_value = context.dfg.get_numeric_constant(lhs)?;

    match eval_constant_binary_op(lhs_value, rhs_value, operator, numeric_type) {
        BinaryEvaluationResult::Failure(message) => Some(message),
        BinaryEvaluationResult::CouldNotEvaluate | BinaryEvaluationResult::Success(..) => None,
    }
}

fn zeroed_value(
    dfg: &mut DataFlowGraph,
    func_id: FunctionId,
    block_id: BasicBlockId,
    typ: &Type,
) -> ValueId {
    match typ {
        Type::Numeric(numeric_type) => dfg.make_constant(FieldElement::zero(), *numeric_type),
        Type::Array(element_types, len) => {
            let mut array = im::Vector::new();
            for _ in 0..*len {
                for typ in element_types.iter() {
                    array.push_back(zeroed_value(dfg, func_id, block_id, typ));
                }
            }
            let instruction = Instruction::MakeArray { elements: array, typ: typ.clone() };
            let stack = CallStackId::root();
            dfg.insert_instruction_and_results(instruction, block_id, None, stack).first()
        }
        Type::Slice(_) => {
            let array = im::Vector::new();
            let instruction = Instruction::MakeArray { elements: array, typ: typ.clone() };
            let stack = CallStackId::root();
            dfg.insert_instruction_and_results(instruction, block_id, None, stack).first()
        }
        Type::Reference(element_type) => {
            // The result of the instruction is a reference; Allocate creates a reference,
            // but if we tried to Load from it we would get an error, so follow it with a
            // Store of a default value.
            let instruction = Instruction::Allocate;
            let reference_type = Type::Reference(Arc::new((**element_type).clone()));

            let reference_id = dfg
                .insert_instruction_and_results(
                    instruction,
                    block_id,
                    Some(vec![reference_type]),
                    CallStackId::root(),
                )
                .first();

            let value = zeroed_value(dfg, func_id, block_id, element_type.as_ref());
            let instruction = Instruction::Store { address: reference_id, value };
            dfg.insert_instruction_and_results(instruction, block_id, None, CallStackId::root());

            reference_id
        }
        Type::Function => dfg.import_function(func_id),
    }
}

/// Remove the current instruction and replace it with default values.
fn remove_and_replace_with_defaults(
    context: &mut SimpleOptimizationContext<'_, '_>,
    func_id: FunctionId,
    block_id: BasicBlockId,
) {
    context.remove_current_instruction();

    let result_ids = context.dfg.instruction_results(context.instruction_id).to_vec();

    for result_id in result_ids {
        let typ = &context.dfg.type_of_value(result_id);
        let default_value = zeroed_value(context.dfg, func_id, block_id, typ);
        context.replace_value(result_id, default_value);
    }
}

/// Insert a `constrain 0 == <predicate>, "<msg>"` instruction.
fn insert_constraint(
    context: &mut SimpleOptimizationContext<'_, '_>,
    block_id: BasicBlockId,
    predicate: ValueId,
    msg: String,
) {
    let zero = context.dfg.make_constant(0_u128.into(), NumericType::bool());
    let message = Some(ConstrainError::StaticString(msg));
    let instruction = Instruction::Constrain(zero, predicate, message);
    let call_stack = context.dfg.get_instruction_call_stack_id(context.instruction_id);
    context.dfg.insert_instruction_and_results(instruction, block_id, None, call_stack);
}

#[cfg(test)]
mod test {
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::assert_ssa_does_not_change, ssa_gen::Ssa},
    };

    #[test]
    fn removes_unreachable_instructions_in_block_for_constrain_equal() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [] : [&mut u1; 0]
            constrain u1 0 == u1 1, "Index out of bounds"
            v4 = array_get v0, index u32 0 -> &mut u1
            v5 = load v4 -> u1
            return v5
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [] : [&mut u1; 0]
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#);
    }

    #[test]
    fn removes_unreachable_instructions_in_block_for_constrain_not_equal() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [] : [&mut u1; 0]
            constrain u1 0 != u1 0, "Index out of bounds"
            v4 = array_get v0, index u32 0 -> &mut u1
            v5 = load v4 -> u1
            return v5
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [] : [&mut u1; 0]
            constrain u1 0 != u1 0, "Index out of bounds"
            unreachable
        }
        "#);
    }

    #[test]
    fn removes_unreachable_instructions_in_block_for_sub_that_overflows() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = sub u32 0, u32 1
            v1 = add v0, u32 1
            return v1
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v2 = sub u32 0, u32 1
            constrain u1 0 == u1 1, "attempt to subtract with overflow"
            unreachable
        }
        "#);
    }

    #[test]
    fn removes_unreachable_instructions_in_block_for_division_by_zero() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = div u32 1, u32 0
            v1 = add v0, u32 1
            return v1
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v2 = div u32 1, u32 0
            constrain u1 0 == u1 1, "attempt to divide by zero"
            unreachable
        }
        "#);
    }

    #[test]
    fn does_not_replace_unchecked_sub_that_overflows() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = unchecked_sub u32 0, u32 1
            v1 = add v0, u32 1
            return v1
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::remove_unreachable_instructions);
    }

    #[test]
    fn replaces_sub_that_overflows_with_constraint_under_unknown_side_effects_condition() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v1 = sub u32 0, u32 1
            v2 = add v1, u32 1
            return v2
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        // The sub is followed by a constraint predicated upon the side effect variable.
        // The add is removed, because if the side effects are enabled the sub will fail,
        // and if the side effects are not enabled, then the sub will have its default
        // value and the add should not matter either.
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v3 = sub u32 0, u32 1
            constrain u1 0 == v0, "attempt to subtract with overflow"
            return u32 0
        }
        "#);
    }

    #[test]
    fn replaces_instructions_following_sub_that_overflows_under_side_effects_condition_with_defaults()
     {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v1 = sub u32 0, u32 1
            v2 = add v1, u32 1
            return v2
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        // The add is removed, because if we know that the sub is disabled and would fail, then we are dealing with
        // default values all the way.
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v3 = sub u32 0, u32 1
            constrain u1 0 == v0, "attempt to subtract with overflow"
            return u32 0
        }
        "#);
    }

    #[test]
    fn replaces_instructions_following_disabled_side_effects_with_defaults() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            enable_side_effects u1 0
            v0 = sub u32 0, u32 1
            v1 = add v0, u32 1
            return v1
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        // Both sub and add are removed because they cannot have side effects.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0():
            enable_side_effects u1 0
            return u32 0
        }
        ");
    }

    #[test]
    fn replaces_references_following_conditional_constraint_with_allocate_and_store_of_default() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            v1 = make_array [] : [&mut u1; 0]
            enable_side_effects v0
            v2 = mod u32 1, u32 0
            constrain u1 0 == v0, "Index out of bounds"
            v3 = array_get v1, index v2 -> &mut u1
            v4 = load v3 -> u1
            enable_side_effects u1 1
            return v4
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        // The `array_get` is replaced by `allocate+store`.
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            v1 = make_array [] : [&mut u1; 0]
            enable_side_effects v0
            v4 = mod u32 1, u32 0
            constrain u1 0 == v0, "attempt to calculate the remainder with a divisor of zero"
            constrain u1 0 == v0, "Index out of bounds"
            v6 = allocate -> &mut u1
            store u1 0 at v6
            v7 = load v6 -> u1
            enable_side_effects u1 1
            return v7
        }
        "#);
    }

    #[test]
    fn does_not_replace_instructions_following_sub_that_overflows_after_next_side_effects_condition()
     {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v1 = sub u32 0, u32 1
            enable_side_effects u1 1
            v2 = add v1, u32 1
            return v2
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v3 = sub u32 0, u32 1
            constrain u1 0 == v0, "attempt to subtract with overflow"
            enable_side_effects u1 1
            v6 = add v3, u32 1
            return v6
        }
        "#);
    }

    #[test]
    fn removes_unreachable_instructions_from_dominated_blocks_normal_order() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [] : [&mut u1; 0]
            constrain u1 0 == u1 1, "Index out of bounds"
            v4 = array_get v0, index u32 0 -> &mut u1
            v5 = load v4 -> u1
            jmp b1(v5)
          b1(v6: u1):
            v7 = add v6, u1 1
            jmp b2(v7)
          b2(v8: u1):
            v9 = add v8, u1 1
            return v9
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [] : [&mut u1; 0]
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#);
    }

    #[test]
    fn removes_unreachable_instructions_from_dominated_blocks_different_order() {
        // This is the same as `removes_unreachable_instructions_from_dominated_blocks_normal_order`
        // except that the blocks are in a different order.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [] : [&mut u1; 0]
            constrain u1 0 == u1 1, "Index out of bounds"
            v4 = array_get v0, index u32 0 -> &mut u1
            v5 = load v4 -> u1
            jmp b2(v5)
          b1(v8: u1):
            v9 = add v8, u1 1
            return v9
          b2(v6: u1):
            v7 = add v6, u1 1
            jmp b1(v7)
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [] : [&mut u1; 0]
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#);
    }

    #[test]
    fn removes_unreachable_instructions_from_dominated_blocks_transitively() {
        // This tests that if a block has an unreachable instruction,
        // all of its successors that are dominated by it are also unreachable,
        // and that is applied recursively.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b1()
          b1():
            v1 = add Field 1, Field 2
            jmpif u1 0 then: b2, else: b3
          b2():
            v2 = add Field 1, Field 2
            jmp b1()
          b3():
            v3 = add Field 1, Field 2
            return v3
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#);
    }

    #[test]
    fn removes_unreachable_instructions_following_block_with_no_instructions() {
        // This tests that if a block is determined to be unreachable,
        // a dominated block that has no instructions also gets its terminator zeroed out.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b1()
          b1():
            jmp b2()
          b2():
            v2 = add Field 1, Field 2
            return v2
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#);
    }

    #[test]
    fn does_not_remove_instructions_from_non_dominated_block_1() {
        // Here both b1 and b4 are successors of b3, but both are not dominated by it.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmp b1()
          b1():
            v2 = add Field 1, Field 2
            jmp b2(v2)
          b2():
            jmpif u1 0 then: b3, else: b4
          b3():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmpif u1 0 then: b4, else: b1
          b4():
            v1 = add Field 1, Field 2
            return v1
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmp b1()
          b1():
            v2 = add Field 1, Field 2
            jmp b2(v2)
          b2():
            jmpif u1 0 then: b3, else: b4
          b3():
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
          b4():
            v4 = add Field 1, Field 2
            return v4
        }
        "#);
    }

    #[test]
    fn does_not_remove_instructions_from_non_dominated_block_2() {
        // Here b3 is a successor of b2 but is not dominated by it.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmp b1()
          b1():
            jmpif u1 0 then: b2, else: b3
          b2():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b3()
          b3():
            v1 = add Field 1, Field 2
            return v1
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmp b1()
          b1():
            jmpif u1 0 then: b2, else: b3
          b2():
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
          b3():
            v3 = add Field 1, Field 2
            return v3
        }
        "#);
    }

    #[test]
    fn does_not_remove_instructions_from_non_dominated_block_3() {
        // Here b4 is a transitive successor of b2 but is not dominated by it.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmp b1()
          b1():
            jmpif u1 0 then: b2, else: b3
          b2():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b4()
          b3():
            jmp b4()
          b4():
            v1 = add Field 1, Field 2
            return v1
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmp b1()
          b1():
            jmpif u1 0 then: b2, else: b3
          b2():
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
          b3():
            jmp b4()
          b4():
            v3 = add Field 1, Field 2
            return v3
        }
        "#);
    }

    #[test]
    fn does_not_remove_instructions_from_non_dominated_block_4() {
        // Here b5 is a transitive successor of b2, but is not dominated by it
        // (it's a transitive successor of b1)
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmp b1()
          b1():
            jmpif u1 0 then: b2, else: b3
          b2():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b4()
          b3():
            jmp b4()
          b4():
            jmp b5()
          b5():
            v1 = add Field 1, Field 2
            return v1
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmp b1()
          b1():
            jmpif u1 0 then: b2, else: b3
          b2():
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
          b3():
            jmp b4()
          b4():
            jmp b5()
          b5():
            v3 = add Field 1, Field 2
            return v3
        }
        "#);
    }

    #[test]
    fn removes_block_that_is_unreachable_when_all_of_its_predecessors_are_unreachable() {
        // Here b4 won't be considered unreachable when we find that b2 or b3 are unreachable,
        // because neither dominate it, but it will still not show up in the final SSA
        // because no block will be able to reach it.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmpif u1 0 then: b1, else: b2
          b1():
            jmp b3()
          b2():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b4()
          b3():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b4()
          b4():
            return Field 1
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmpif u1 0 then: b1, else: b2
          b1():
            jmp b3()
          b2():
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
          b3():
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#);
    }

    #[test]
    fn transforms_failing_array_access_to_work_on_dummy_array() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = allocate -> &mut u8
            store u8 0 at v0
            v2 = make_array [u8 0, v0] : [(u8, &mut u8); 1]
            v4 = array_get v2, index u32 2 -> u8
            v6 = array_get v2, index u32 3 -> &mut u8
            return v4
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        // We expect the array containing references to no longer be in use,
        // for the failing array get to now be over a dummy array.
        // We expect the new assertion to also use the correct dummy type (u1) as to have a well formed SSA.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = allocate -> &mut u8
            store u8 0 at v0
            v2 = make_array [u8 0, v0] : [(u8, &mut u8); 1]
            v4 = make_array [u1 0] : [u1; 1]
            v6 = array_get v4, index u32 2 -> u1
            unreachable
        }
        ");
    }

    #[test]
    fn do_not_transform_failing_array_access_in_brillig() {
        let src = "
        brillig(inline) predicate_pure fn main f0 {
          b0():
            v0 = allocate -> &mut u8
            store u8 0 at v0
            v2 = make_array [u8 0, v0] : [(u8, &mut u8); 1]
            v4 = array_get v2, index u32 2 -> u8
            v6 = array_get v2, index u32 3 -> &mut u8
            return v4
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_unreachable_instructions);
    }

    #[test]
    fn transforms_failing_slice_pop_with_constraint_and_default() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            v1 = make_array [] : [u32]
            enable_side_effects v0
            v4, v5, v6 = call slice_pop_front(u32 0, v1) -> (u32, u32, [u32])
            enable_side_effects u1 1
            return u32 1
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            v1 = make_array [] : [u32]
            enable_side_effects v0
            constrain u1 0 == v0, "Index out of bounds"
            v3 = make_array [] : [u32]
            enable_side_effects u1 1
            return u32 1
        }
        "#);
    }

    #[test]
    fn transforms_failing_slice_pop_if_always_enabled() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            v1 = make_array [] : [u32]
            v4, v5, v6 = call slice_pop_front(u32 0, v1) -> (u32, u32, [u32])
            return v4
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            v1 = make_array [] : [u32]
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#);
    }

    #[test]
    fn simplifies_instructions_following_conditional_failure() {
        // In the following SSA we have:
        // 1. v1 is a divide-by-zero which turns into an conditional-fail constraint under v0
        // 2. v2 is replaced by its default value (because division is considered side effecting)
        // 3. v3 would be turned into a `truncate u64 0 to 32 bits` due to step 2, which is not expected to reach ACIR gen.
        // We expect 3 to disappear as it can be simplified out.
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v1 = div u64 1, u64 0
            v2 = div u64 1, u64 1
            v3 = truncate v2 to 32 bits, max_bit_size: 254
            enable_side_effects u1 1
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v3 = div u64 1, u64 0
            constrain u1 0 == v0, "attempt to divide by zero"
            enable_side_effects u1 1
            return
        }
        "#);
    }
}
