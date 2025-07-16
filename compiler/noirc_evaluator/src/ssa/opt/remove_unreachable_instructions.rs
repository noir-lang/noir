//! This module defines an SSA pass to remove instructions that are unreachable.
//! For example, if an instruction in a block is `constrain u1 0 == u1 1`,
//! any subsequent instructions in that block will never be executed. This pass
//! then removes those subsequent instructions and replaces the block's terminator
//! with a special `unreachable` value.
//!
//! This pass might also add constrain checks after existing instructions,
//! for example binary operations that are guaranteed to overflow.
use std::sync::Arc;

use acvm::{AcirField, FieldElement};
use noirc_errors::call_stack::CallStackId;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        function::{Function, FunctionId},
        instruction::{
            Binary, BinaryOp, ConstrainError, Instruction, TerminatorInstruction,
            binary::{BinaryEvaluationResult, eval_constant_binary_op},
        },
        types::{NumericType, Type},
        value::ValueId,
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

        self.simple_reachable_blocks_optimization(|context| {
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
                if current_block_reachability == Reachability::UnreachableUnderPredicate {
                    current_block_reachability = Reachability::Reachable;
                }
                return;
            };

            if current_block_reachability == Reachability::UnreachableUnderPredicate {
                // Instructions that don't interact with the predicate should be left alone,
                // because the `remove_enable_side_effects` pass might have moved the boundaries around them.
                if !instruction.requires_acir_gen_predicate(context.dfg) {
                    return;
                }
                // Remove the current instruction and insert defaults for the results.
                context.remove_current_instruction();

                let result_ids = context.dfg.instruction_results(context.instruction_id).to_vec();

                for result_id in result_ids {
                    let typ = &context.dfg.type_of_value(result_id);
                    let default_value = zeroed_value(context.dfg, func_id, block_id, typ);
                    context.replace_value(result_id, default_value);
                }
                return;
            }

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

                        // Check if the current predicate is known to be enabled.
                        let is_predicate_constant_one =
                            match context.dfg.get_numeric_constant(side_effects_condition) {
                                Some(predicate) => predicate.is_one(),
                                None => false, // The predicate is a variable
                            };

                        let fails_under_predicate =
                            requires_acir_gen_predicate && !is_predicate_constant_one;

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
                | Instruction::ArraySet { array, index, offset, .. } => {
                    let array_or_slice_type = context.dfg.type_of_value(*array);
                    let array_op_always_fails = match array_or_slice_type {
                        Type::Slice(_) => false,
                        array_type @ Type::Array(_, len) => {
                            len == 0 || context.dfg.get_numeric_constant(*index).is_some_and(|index| {
                              (index.try_to_u32().unwrap() - offset.to_u32())
                                  >= array_type.flattened_size()})
                        }
                        
                        _ => unreachable!(
                            "Encountered non-array type during array read/write operation"
                        ),
                    };

                    if array_op_always_fails {
                      let is_predicate_constant_one =
                      match context.dfg.get_numeric_constant(side_effects_condition) {
                          Some(predicate) => predicate.is_one(),
                          None => false, // The predicate is a variable
                      };
                      current_block_reachability = if is_predicate_constant_one { Reachability::Unreachable } else { Reachability::UnreachableUnderPredicate };
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
            let instruction = Instruction::Allocate;
            let reference_type = Type::Reference(Arc::new((**element_type).clone()));

            dfg.insert_instruction_and_results(
                instruction,
                block_id,
                Some(vec![reference_type]),
                CallStackId::root(),
            )
            .first()
        }
        Type::Function => dfg.import_function(func_id),
    }
}

#[cfg(test)]
mod test {
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::assert_normalized_ssa_equals, ssa_gen::Ssa},
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
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn removes_unreachable_instructions_in_block_for_invalid_array_get() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [Field; 10], v1: u32):
            v2 = make_array [] : [Field; 0]
            jmpif u1 0 then: b1, else: b2
          b1():
            jmp b3()
          b2():
            v3 = array_get v2, index v1 -> Field
            jmp b4()
          b3():
            v4 = array_get v0, index u32 11 -> Field
            jmp b4()
          b4():
            return Field 1
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [Field; 10], v1: u32):
            v2 = make_array [] : [Field; 0]
            jmpif u1 0 then: b1, else: b2
          b1():
            jmp b3()
          b2():
            v4 = array_get v2, index v1 -> Field
            unreachable
          b3():
            v6 = array_get v0, index u32 11 -> Field
            unreachable
        }
        ");
    }

    #[test]
    fn replaces_sub_that_overflows_with_constraint_under_unknown_side_effects_condition()
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
    fn replaces_instructions_following_sub_that_overflows_under_false_side_effects_condition_with_defaults()
     {
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

        // No constraint will be added because we know that the sub is disabled, (it would be the trivial `0 == 0`).
        // The add is removed, because if we know that the sub is disabled and would fail, then we are dealing with
        // default values all the way.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0():
            enable_side_effects u1 0
            v3 = sub u32 0, u32 1
            return u32 0
        }
        ");
    }

    #[test]
    fn does_not_replace_instructions_following_sub_that_overflows_after_next_side_effects_condition()
     {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            enable_side_effects u1 0
            v0 = sub u32 0, u32 1
            enable_side_effects u1 0
            v1 = add v0, u32 1
            return v1
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_normalized_ssa_equals(ssa, src);
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
    fn does_not_removes_instructions_from_non_dominated_block_1() {
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
    fn does_not_removes_instructions_from_non_dominated_block_2() {
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
    fn does_not_removes_instructions_from_non_dominated_block_3() {
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
    fn does_not_removes_instructions_from_non_dominated_block_4() {
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
}
