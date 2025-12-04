//! This module defines an SSA pass to remove instructions that are unreachable.
//! For example, if an instruction in a block is `constrain u1 0 == u1 1`,
//! any subsequent instructions in that block will never be executed. This pass
//! then removes those subsequent instructions and replaces the block's terminator
//! with a special `unreachable` value.
//!
//! This pass might also replace existing instructions with constrain checks,
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
//! enable_side_effects v0
//! v1 = make_array [] -> [&mut Field; 0]
//! v2 = array_get v1, index u32 0 -> &mut Field
//! v3 = load v2
//! v4 = add v3, Field 2
//! ```
//!
//! the array operation is replaced with a constrain failure and its results with
//! default values:
//!
//! ```ssa
//! enable_side_effects v0
//! v1 = make_array [] -> [&mut Field; 0]
//! constrain v0 == u1 0, "Index out of bounds"
//! v2 = allocate -> &mut Field
//! store Field 0 at v2
//! v3 <- load v2 -> Field
//! v4 = add v3, Field 2
//! ```
//!
//! For the `store` and the `load` to be resolved, this pass has to be followed up
//! with a `mem2reg` pass before any subsequent DIE pass would remove the `store`,
//! leaving the `load` with a reference that never gets stored at.
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
use im::HashSet;
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
            function.remove_unreachable_instructions(function.id() == self.main_id);
        }
        self
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
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
    fn remove_unreachable_instructions(&mut self, is_main: bool) {
        let func_id = self.id();

        // Keep the return instruction in main if the databus is used
        let replace_return = !(is_main && self.dfg.data_bus.return_data.is_some());

        // The current block we are currently processing
        let mut current_block_id = None;

        // Whether the current block instructions were determined to be unreachable
        // after an always failing one was found.
        let mut current_block_reachability = Reachability::Reachable;

        // In some cases a side effect variable can become effective again.
        let mut unreachable_predicates = HashSet::new();

        self.simple_optimization(|context| {
            let block_id = context.block_id;

            if current_block_id != Some(block_id) {
                current_block_id = Some(block_id);
                current_block_reachability = Reachability::Reachable;
                unreachable_predicates.clear();
            }

            if current_block_reachability == Reachability::Unreachable {
                if context.dfg.is_returned_in_databus(context.instruction_id) {
                    // We have to keep databus assignments at the end of the ACIR main function alive,
                    // otherwise we can't print the SSA, as it will crash trying to normalize values
                    // that no longer get created in the SSA.
                    // The reason it is enough to this only for unreachable blocks without worrying
                    // about their successors is that databus is only used in ACIR, and we only remove
                    // unreachable instructions after flattening, so there is only one block.
                    remove_and_replace_with_defaults(context, func_id, block_id);
                } else {
                    context.remove_current_instruction();
                }
                return;
            }

            let instruction = context.instruction();
            if let Instruction::EnableSideEffectsIf { condition } = instruction {
                current_block_reachability =
                    if let Some(predicate) = context.dfg.get_numeric_constant(*condition) {
                        // If side effects are turned off, we can replace side effecting instructions with defaults until the next predicate
                        if predicate.is_zero() {
                            Reachability::UnreachableUnderPredicate
                        } else {
                            Reachability::Reachable
                        }
                    } else {
                        // During loops a previous predicate variable can be restored.
                        if unreachable_predicates.contains(condition) {
                            Reachability::UnreachableUnderPredicate
                        } else {
                            Reachability::Reachable
                        }
                    };
                return;
            };

            if current_block_reachability == Reachability::UnreachableUnderPredicate {
                if should_replace_instruction_with_defaults(context) {
                    remove_and_replace_with_defaults(context, func_id, block_id);
                }
                return;
            }

            // Check if the current predicate is known to be enabled.
            let is_predicate_constant_one =
                match context.dfg.get_numeric_constant(context.enable_side_effects) {
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
                    let Some(message) =
                        binary_operation_always_fails(*lhs, *operator, *rhs, context)
                    else {
                        return;
                    };

                    // Check if this operation is one that should only fail if the predicate is enabled.
                    let requires_acir_gen_predicate =
                        binary.requires_acir_gen_predicate(context.dfg);

                    let fails_under_predicate =
                        requires_acir_gen_predicate && !is_predicate_constant_one;

                    insert_constraint(context, block_id, message);

                    // Subsequent instructions can either be removed, of replaced by defaults until the next predicate.
                    current_block_reachability = if fails_under_predicate {
                        remove_and_replace_with_defaults(context, func_id, block_id);
                        Reachability::UnreachableUnderPredicate
                    } else {
                        context.remove_current_instruction();
                        Reachability::Unreachable
                    }
                }
                Instruction::ArrayGet { array, index }
                | Instruction::ArraySet { array, index, .. }
                    if context.dfg.runtime().is_acir() =>
                {
                    let array_type = context.dfg.type_of_value(*array);
                    // We can only know a guaranteed out-of-bounds access for arrays,
                    // and slices which have been declared as a literal.
                    let len = match array_type {
                        Type::Array(_, len) => len,
                        Type::Slice(_) => {
                            let Some(Instruction::MakeArray { elements, typ }) =
                                context.dfg.get_local_or_global_instruction(*array)
                            else {
                                return;
                            };
                            // The index check expects `len` to be the logical length, like for arrays,
                            // not the flattened size, so we need to divide by the number of items.
                            (elements.len() / typ.element_size()) as u32
                        }
                        _ => return,
                    };

                    let array_op_always_fails = len == 0
                        || context.dfg.get_numeric_constant(*index).is_some_and(|index| {
                            (index.try_to_u32().unwrap())
                                >= (array_type.element_size() as u32 * len)
                        });
                    if !array_op_always_fails {
                        return;
                    }

                    let always_fail = is_predicate_constant_one;
                    // We could leave the array operation to trigger an OOB on the invalid access, however if the array contains and returns
                    // references, then the SSA passes still won't be able to deal with them as nothing ever stores to references which are
                    // never created. Instead, we can replace the result of the instruction with defaults, which will allocate and store
                    // defaults to those references, so subsequent SSA passes can complete without errors.
                    insert_constraint(context, block_id, "Index out of bounds".to_string());

                    current_block_reachability = if always_fail {
                        // If the block fails unconditionally, we don't even need a default for the results.
                        context.remove_current_instruction();
                        Reachability::Unreachable
                    } else {
                        // We will never use the results (the constraint fails if the side effects are enabled),
                        // but we need them to make the rest of the SSA valid even if the side effects are off.
                        remove_and_replace_with_defaults(context, func_id, block_id);
                        Reachability::UnreachableUnderPredicate
                    };
                }
                Instruction::Call { func, arguments } if context.dfg.runtime().is_acir() => {
                    // Intrinsic Slice operations in ACIR on empty arrays need to be replaced with a (conditional) constraint.
                    // In Brillig they will be protected by an access constraint, which, if known to fail, will make the block unreachable.
                    let Value::Intrinsic(Intrinsic::SlicePopBack | Intrinsic::SlicePopFront) =
                        &context.dfg[*func]
                    else {
                        return;
                    };

                    let length = arguments.first().unwrap_or_else(|| {
                        unreachable!("slice operations have 2 arguments: [length, slice]")
                    });
                    let is_empty =
                        context.dfg.get_numeric_constant(*length).is_some_and(|v| v.is_zero());
                    if !is_empty {
                        return;
                    }

                    // If the compiler knows the slice is empty, there is no point trying to pop from it, we know it will fail.
                    // Barretenberg doesn't handle memory operations with predicates, so we can't rely on those to disable the operation
                    // based on the current side effect variable. Instead we need to replace it with a conditional constraint.
                    let always_fail = is_predicate_constant_one;

                    // We might think that if the predicate is constant 1, we can leave the pop as it will always fail.
                    // However by turning the block Unreachable, ACIR-gen would create empty bytecode and not fail the circuit.
                    insert_constraint(context, block_id, "Index out of bounds".to_string());

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
                _ => (),
            };

            // Once we find an instruction that will always fail, replace the terminator with `unreachable`.
            // Subsequent instructions in this block will be removed.
            if current_block_reachability == Reachability::Unreachable && replace_return {
                let terminator = context.dfg[block_id].take_terminator();
                let call_stack = terminator.call_stack();
                context.dfg[block_id]
                    .set_terminator(TerminatorInstruction::Unreachable { call_stack });
            }
            if current_block_reachability == Reachability::UnreachableUnderPredicate {
                assert!(
                    !is_predicate_constant_one,
                    "predicate cannot be constant one in UnreachableUnderPredicate"
                );
                unreachable_predicates.insert(context.enable_side_effects);
            }
        });
    }
}

/// If a binary operation is guaranteed to fail, returns the error message. Otherwise returns None.
fn binary_operation_always_fails(
    lhs: ValueId,
    operator: BinaryOp,
    rhs: ValueId,
    context: &SimpleOptimizationContext,
) -> Option<String> {
    // Unchecked operations can never fail
    if binary_operator_is_unchecked(operator) {
        return None;
    }

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

fn binary_operator_is_unchecked(operator: BinaryOp) -> bool {
    match operator {
        BinaryOp::Add { unchecked } | BinaryOp::Sub { unchecked } | BinaryOp::Mul { unchecked } => {
            unchecked
        }
        BinaryOp::Div
        | BinaryOp::Mod
        | BinaryOp::Eq
        | BinaryOp::Lt
        | BinaryOp::And
        | BinaryOp::Or
        | BinaryOp::Xor
        | BinaryOp::Shl
        | BinaryOp::Shr => false,
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
    msg: String,
) {
    let zero = context.dfg.make_constant(0_u128.into(), NumericType::bool());
    let message = Some(ConstrainError::StaticString(msg));
    let instruction = Instruction::Constrain(zero, context.enable_side_effects, message);
    let call_stack = context.dfg.get_instruction_call_stack_id(context.instruction_id);
    context.dfg.insert_instruction_and_results(instruction, block_id, None, call_stack);
}

/// Check if an instruction should be replaced with default values if we are in the
/// `UnreachableUnderPredicate` mode.
///
/// These are generally the ones that require an ACIR predicate, except for `ArrayGet`,
/// which might appear safe after having its index replaced by a default zero value,
/// but by doing so we may have made the item and result types misaligned.
fn should_replace_instruction_with_defaults(context: &SimpleOptimizationContext) -> bool {
    let instruction = context.instruction();

    // ArrayGet needs special handling: if we replaced the index with a default value, it could be invalid.
    if let Instruction::ArrayGet { array, index } = instruction {
        // If we replaced the index with a default, it's going to be zero.
        let index_zero = context.dfg.get_numeric_constant(*index).is_some_and(|c| c.is_zero());

        // If it's zero, make sure that the type in the results
        if index_zero {
            let typ = match context.dfg.type_of_value(*array) {
                Type::Array(typ, _) | Type::Slice(typ) => typ,
                other => unreachable!("Array or Slice type expected; got {other:?}"),
            };
            let [result] = context.dfg.instruction_result(context.instruction_id);
            let result_type = context.dfg.type_of_value(result);
            // If the type doesn't agree then we should not use this any more,
            // as the type in the array will replace the type we wanted to get,
            // and cause problems further on.
            if typ[0] != result_type {
                return true;
            }
            // If the array contains a reference, then we should replace the results
            // with defaults because unloaded references also cause issues.
            if context.dfg.runtime().is_acir() && result_type.contains_reference() {
                return true;
            }
            // Note that it may be incorrect to replace a *safe* ArrayGet with defaults,
            // because `remove_enable_side_effects` may have moved the side effect
            // boundaries around them, and then `fold_constants_with_brillig` could
            // have replaced some with `enable_side_effect u1 0`. If we then replace
            // a *safe* ArrayGet with a default, that might be a result that would
            // really be enabled, had it not been skipped over by its original side
            // effect variable. Instructions which use its result would then get
            // incorrect zero, instead of whatever was in the array.
        }
    };

    // Instructions that don't interact with the predicate should be left alone,
    // because the `remove_enable_side_effects` pass might have moved the boundaries around them.
    instruction.requires_acir_gen_predicate(context.dfg)
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
            constrain u1 0 == v0, "attempt to calculate the remainder with a divisor of zero"
            constrain u1 0 == v0, "Index out of bounds"
            v3 = allocate -> &mut u1
            store u1 0 at v3
            v4 = load v3 -> u1
            enable_side_effects u1 1
            return v4
        }
        "#);
    }

    #[test]
    fn replaces_array_get_following_conditional_constraint_with_default_if_index_was_defaulted() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v1 = make_array [u8 1, u8 2] : [u8; 2]
            v2 = make_array [v1, u1 0] : [([u8; 2], u1); 1]
            v3 = mul u32 4294967295, u32 2          // overflow
            v4 = add v3, u32 1                      // after overflow, replaced by default
            enable_side_effects u1 1                // end of side effects mode
            enable_side_effects v0                  // restore side effects to what we know will fail
            v5 = array_get v1, index v4 -> u1       // if v4 is replaced by default, the item at 0 is not a u1
            v6 = unchecked_mul v0, v5               // v5 is no longer a u1, but [u8; 2]
            enable_side_effects u1 1
            return
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        // The `array_get` is should be replaced by `u1 0`
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v3 = make_array [u8 1, u8 2] : [u8; 2]
            v5 = make_array [v3, u1 0] : [([u8; 2], u1); 1]
            constrain u1 0 == v0, "attempt to multiply with overflow"
            enable_side_effects u1 1
            enable_side_effects v0
            enable_side_effects u1 1
            return
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
            constrain u1 0 == v0, "attempt to subtract with overflow"
            enable_side_effects u1 1
            return u32 1
        }
        "#);
    }

    #[test]
    fn removes_slice_literal_index_oob() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v1 = make_array [u1 1, u32 2, u64 3] : [(u1, u32, u64)]
            v2 = array_get v1, index u32 4 -> u32
            return v2
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v4 = make_array [u1 1, u32 2, u64 3] : [(u1, u32, u64)]
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
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
    fn removes_failing_array_access_when_predicate_is_one() {
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
        // and the failing array_get instructions be replaced by an always-fail constraint.
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = allocate -> &mut u8
            store u8 0 at v0
            v2 = make_array [u8 0, v0] : [(u8, &mut u8); 1]
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#);
    }

    #[test]
    fn removes_failing_array_access_when_predicate_is_variable() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v1 = make_array [] : [&mut u8; 0]
            v2 = array_get v1, index u32 0 -> &mut u8
            v3 = load v2 -> u8
            enable_side_effects u1 1
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        // We expect the empty array to no longer be in use,
        // and the result of the array_get be replaced by an allocation and store
        // that makes the following load valid, but technically unreachable because
        // of a conditionally failing constraint.
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v1 = make_array [] : [&mut u8; 0]
            constrain u1 0 == v0, "Index out of bounds"
            v3 = allocate -> &mut u8
            store u8 0 at v3
            v5 = load v3 -> u8
            enable_side_effects u1 1
            return
        }
        "#);
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
            constrain u1 0 == v0, "attempt to divide by zero"
            enable_side_effects u1 1
            return
        }
        "#);
    }

    #[test]
    fn replaces_databus_return_data_with_default_in_unreachable() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          return_data: v3
          b0(v0: u32):
            constrain u1 0 == u1 1
            v1 = sub v0, u32 10
            v2 = cast v1 as Field
            v3 = make_array [v2] : [Field; 1]
            return v3
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          return_data: v4
          b0(v0: u32):
            constrain u1 0 == u1 1
            v4 = make_array [Field 0] : [Field; 1]
            return v4
        }
        ");
    }
}
