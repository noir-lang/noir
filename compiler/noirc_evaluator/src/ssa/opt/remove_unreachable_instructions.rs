//! This module defines an SSA pass to remove instructions that are unreachable.
//! For example, if an instruction in a block is `constrain u1 0 == u1 1`,
//! any subsequent instructions in that block will never be executed. This pass
//! then removes those subsequent instructions and replaces the block's terminator
//! with a special `unreachable` value.
//!
//! This pass might also add constrain checks after existing instructions,
//! for example binary operations that are guaranteed to overflow.
use acvm::AcirField;

use crate::ssa::{
    ir::{
        function::Function,
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

impl Function {
    fn remove_unreachable_instructions(&mut self) {
        // The current block we are currently processing
        let mut current_block_id = None;

        // Whether the current block instructions were determined to be unreachable
        // after an always failing one was found.
        let mut current_block_instructions_are_unreachable = false;

        let one = self.dfg.make_constant(1_u32.into(), NumericType::bool());
        let mut side_effects_condition = one;

        self.simple_reachable_blocks_optimization(|context| {
            let block_id = context.block_id;

            if current_block_id != Some(block_id) {
                current_block_id = Some(block_id);
                current_block_instructions_are_unreachable = false;
                side_effects_condition = one;
            }

            if current_block_instructions_are_unreachable {
                context.remove_current_instruction();
                return;
            }

            let instruction = context.instruction();
            if let Instruction::EnableSideEffectsIf { condition } = instruction {
                side_effects_condition = *condition;
                return;
            };

            let always_fails = match instruction {
                Instruction::Constrain(lhs, rhs, _) => {
                    let Some(lhs_constant) = context.dfg.get_numeric_constant(*lhs) else {
                        return;
                    };
                    let Some(rhs_constant) = context.dfg.get_numeric_constant(*rhs) else {
                        return;
                    };
                    lhs_constant != rhs_constant
                }
                Instruction::ConstrainNotEqual(lhs, rhs, _) => {
                    let Some(lhs_constant) = context.dfg.get_numeric_constant(*lhs) else {
                        return;
                    };
                    let Some(rhs_constant) = context.dfg.get_numeric_constant(*rhs) else {
                        return;
                    };
                    lhs_constant == rhs_constant
                }
                Instruction::Binary(binary @ Binary { lhs, operator, rhs }) => {
                    let requires_acir_gen_predicate =
                        binary.requires_acir_gen_predicate(context.dfg);
                    if requires_acir_gen_predicate {
                        // If performing the binary operation depends on the side effects condition, then
                        // we can only simplify it if the condition is true: not when it's zero, and not when it's a variable.
                        let predicate = context.dfg.get_numeric_constant(side_effects_condition);
                        match predicate {
                            Some(predicate) => {
                                if predicate.is_zero() {
                                    // The predicate is zero
                                    return;
                                }
                            }
                            None => {
                                // The predicate is a variable
                                return;
                            }
                        }
                    }

                    if let Some(message) =
                        binary_operation_always_fails(*lhs, *operator, *rhs, context)
                    {
                        // Insert the instruction right away so we can add a constrain immediately after it
                        context.insert_current_instruction();

                        let zero = context.dfg.make_constant(0_u128.into(), NumericType::bool());
                        let one = context.dfg.make_constant(1_u128.into(), NumericType::bool());
                        let message = Some(ConstrainError::StaticString(message));
                        let instruction = Instruction::Constrain(zero, one, message);
                        let call_stack =
                            context.dfg.get_instruction_call_stack_id(context.instruction_id);
                        context.dfg.insert_instruction_and_results(
                            instruction,
                            block_id,
                            None,
                            call_stack,
                        );
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if always_fails {
                current_block_instructions_are_unreachable = true;

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
    context: &mut SimpleOptimizationContext,
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
    fn does_not_replace_sub_that_overflows_but_is_disabled_because_of_unknown_side_effects_condition()
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
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn does_not_replace_sub_that_overflows_but_is_disabled_because_of_false_side_effects_condition()
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
