//! The goal of the "remove enable side effects" optimization pass is to delay any
//! [Instruction::EnableSideEffectsIf] instructions in ACIR functions such that they cover
//! the minimum number of instructions possible.
//!
//! The pass works as follows:
//! - Insert instructions until an [Instruction::EnableSideEffectsIf] is encountered, save this
//!   [InstructionId][crate::ssa::ir::instruction::InstructionId].
//! - Continue inserting instructions until either
//!     - Another [Instruction::EnableSideEffectsIf] is encountered, if so then drop the previous
//!       [InstructionId][crate::ssa::ir::instruction::InstructionId] in favour of this one.
//!     - An [Instruction] that is affected by the side-effects variable is encountered, if so
//!       then insert the currently saved [Instruction::EnableSideEffectsIf] before the
//!       [Instruction]. Continue inserting instructions until the next
//!       [Instruction::EnableSideEffectsIf] is encountered.
//!
//! The pass will also remove redundant [Instruction::EnableSideEffectsIf] instructions,
//! for example if two consecutive [Instruction::EnableSideEffectsIf] instructions have the same
//! condition.
//!
//! This pass doesn't run in Brillig functions as [Instruction::EnableSideEffectsIf] is not allowed
//! in Brillig functions.
//!
//! ## Preconditions:
//! - This pass must run once ACIR functions only have one basic block.

use acvm::{FieldElement, acir::AcirField};

use crate::ssa::{
    ir::{dfg::DataFlowGraph, function::Function, instruction::Instruction, types::NumericType},
    ssa_gen::Ssa,
};

impl Ssa {
    /// See [`remove_enable_side_effects`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn remove_enable_side_effects(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            #[cfg(debug_assertions)]
            remove_enable_side_effects_pre_check(function);

            function.remove_enable_side_effects();
        }
        self
    }
}

impl Function {
    fn remove_enable_side_effects(&mut self) {
        if self.runtime().is_brillig() {
            // Brillig functions do not make use of the `EnableSideEffects` instruction so are unaffected by this pass.
            return;
        }

        let one = self.dfg.make_constant(FieldElement::one(), NumericType::bool());
        let mut active_condition = one;
        let mut last_side_effects_enabled_instruction = None;

        let mut previous_block = None;

        self.simple_optimization(|context| {
            if Some(context.block_id) != previous_block {
                active_condition = one;
                last_side_effects_enabled_instruction = None;
            }
            previous_block = Some(context.block_id);

            let instruction = context.instruction();

            // If we run into another `Instruction::EnableSideEffectsIf` before encountering any
            // instructions with side effects then we can drop the instruction we're holding and
            // continue with the new `Instruction::EnableSideEffectsIf`.
            if let Instruction::EnableSideEffectsIf { condition } = instruction {
                // If this instruction isn't changing the currently active condition then we can ignore it.
                if active_condition == *condition {
                    context.remove_current_instruction();
                    return;
                }
                active_condition = *condition;

                // If we're seeing an `enable_side_effects u1 1` then we want to insert it immediately.
                // This is because we want to maximize the effect it will have.
                let condition_is_one = context
                    .dfg
                    .get_numeric_constant(*condition)
                    .is_some_and(|condition| condition.is_one());

                if condition_is_one {
                    last_side_effects_enabled_instruction = None;
                    context.insert_current_instruction();
                } else {
                    last_side_effects_enabled_instruction = Some(context.instruction_id);
                    context.remove_current_instruction();
                }
                return;
            }

            if should_insert_side_effects_before_instruction(instruction, context.dfg) {
                if let Some(enable_side_effects_instruction_id) =
                    last_side_effects_enabled_instruction.take()
                {
                    context.insert_instruction_by_id(enable_side_effects_instruction_id);
                }
            }
        });
    }
}

/// Decide we should insert any pending side effect variable before we insert
/// a particular instruction into the SSA.
fn should_insert_side_effects_before_instruction(
    instruction: &Instruction,
    dfg: &DataFlowGraph,
) -> bool {
    // Constrain instructions don't need ACIR predicates, because the variables
    // they operate on have the side effects incorporated into them,
    // however they can later be turned into a ConstrainNotEqual instruction,
    // which _does_ need an ACIR predicate. If we don't require the side effect
    // variable for Constrain, then we might lose it and end up with a disabled
    // constrain, as it could inherit some unintended side effect.
    if matches!(instruction, Instruction::Constrain(..)) {
        return true;
    }
    // If we hit an instruction which is affected by the side effects var then we must insert the
    // `Instruction::EnableSideEffectsIf` before we insert this new instruction.
    instruction.requires_acir_gen_predicate(dfg)
}

/// Check that the CFG has been flattened.
#[cfg(debug_assertions)]
fn remove_enable_side_effects_pre_check(function: &Function) {
    if !function.runtime().is_acir() {
        return;
    }
    let block = function.entry_block();
    assert_eq!(function.dfg[block].successors().count(), 0);
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::assert_ssa_does_not_change, ssa_gen::Ssa},
    };

    #[test]
    fn remove_chains_of_same_condition_with_constant() {
        // Here all enable_side_effects u1 1 are removed because by default the active
        // side effects var is `u1 1` so all of these are redundant.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            enable_side_effects u1 1
            v3 = mul v0, Field 2
            enable_side_effects u1 1
            v4 = mul v0, Field 2
            enable_side_effects u1 1
            v5 = mul v0, Field 2
            enable_side_effects u1 1
            v6 = mul v0, Field 2
            enable_side_effects u1 1
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.remove_enable_side_effects();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2 = mul v0, Field 2
            v3 = mul v0, Field 2
            v4 = mul v0, Field 2
            v5 = mul v0, Field 2
            return
        }
        "
        );
    }

    #[test]
    fn remove_chains_of_same_condition_with_variable() {
        // Here only the first enable_side_effects is kept because the rest are redundant
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [u16; 3], v1: u32, v2: u1):
            enable_side_effects v2
            enable_side_effects v2
            v3 = array_get v0, index v1 -> u16
            enable_side_effects v2
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.remove_enable_side_effects();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [u16; 3], v1: u32, v2: u1):
            enable_side_effects v2
            v3 = array_get v0, index v1 -> u16
            return
        }
        "
        );
    }

    #[test]
    fn keeps_enable_side_effects_for_instructions_that_have_side_effects() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [u16; 3], v1: u32, v2: u1, v3: u1):
            enable_side_effects v2
            v4 = array_get v0, index v1 -> u16
            enable_side_effects v2
            v5 = array_get v0, index v1 -> u16
            enable_side_effects v3
            v6 = array_get v0, index v1 -> u16
            enable_side_effects u1 1
            v7 = array_get v0, index v1 -> u16
            return
        }";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.remove_enable_side_effects();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [u16; 3], v1: u32, v2: u1, v3: u1):
            enable_side_effects v2
            v4 = array_get v0, index v1 -> u16
            v5 = array_get v0, index v1 -> u16
            enable_side_effects v3
            v6 = array_get v0, index v1 -> u16
            enable_side_effects u1 1
            v8 = array_get v0, index v1 -> u16
            return
        }
        ");
    }

    #[test]
    fn keeps_enable_side_effects_for_unsafe_modulo() {
        // This is a simplification of `test_programs/execution_success/regression_8236`
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [u16; 3], v1: [u1; 1], v2: u32):
            v4 = call f1(v0, u1 1) -> [u1; 1]
            v6 = array_get v0, index u32 0 -> u16
            v7 = cast v6 as u32
            v8 = array_get v4, index u32 0 -> u1
            v9 = not v8
            enable_side_effects v9
            v11 = mod v7, v2
            v12 = array_get v0, index v11 -> u16
            enable_side_effects u1 1
            return v12
        }
        brillig(inline) predicate_pure fn func_1 f1 {
          b0(v0: [u16; 3], v1: u1):
            v3 = make_array [u1 0] : [u1; 1]
            return v3
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::remove_enable_side_effects);
    }

    #[test]
    fn does_not_keep_enable_side_effects_for_safe_modulo() {
        // This is a simplification of `test_programs/execution_success/regression_8236`
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [u16; 3], v1: [u1; 1]):
            v4 = call f1(v0, u1 1) -> [u1; 1]
            v6 = array_get v0, index u32 0 -> u16
            v7 = cast v6 as u32
            v8 = array_get v4, index u32 0 -> u1
            v9 = not v8
            enable_side_effects v9
            v11 = mod v7, u32 3
            v12 = array_get v0, index v11 -> u16
            enable_side_effects u1 1
            return v12
        }
        brillig(inline) predicate_pure fn func_1 f1 {
          b0(v0: [u16; 3], v1: u1):
            v3 = make_array [u1 0] : [u1; 1]
            return v3
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_enable_side_effects();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [u16; 3], v1: [u1; 1]):
            v4 = call f1(v0, u1 1) -> [u1; 1]
            v6 = array_get v0, index u32 0 -> u16
            v7 = cast v6 as u32
            v8 = array_get v4, index u32 0 -> u1
            v9 = not v8
            v11 = mod v7, u32 3
            enable_side_effects v9
            v12 = array_get v0, index v11 -> u16
            enable_side_effects u1 1
            return v12
        }
        brillig(inline) predicate_pure fn func_1 f1 {
          b0(v0: [u16; 3], v1: u1):
            v3 = make_array [u1 0] : [u1; 1]
            return v3
        }
        ");
    }

    #[test]
    fn keeps_side_effects_for_unsafe_div() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [u32; 3], v1: u1, v2: u32):
            v3 = array_get v0, index u32 0 -> u32
            enable_side_effects v1
            v5 = div v3, v2
            return
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::remove_enable_side_effects);
    }

    #[test]
    fn does_not_keep_side_effects_for_safe_div() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [u32; 3], v1: u1):
            v3 = array_get v0, index u32 0 -> u32
            enable_side_effects v1
            v5 = div v3, u32 3
            return
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_enable_side_effects();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [u32; 3], v1: u1):
            v3 = array_get v0, index u32 0 -> u32
            v5 = div v3, u32 3
            return
        }
        ");
    }

    #[test]
    fn remove_enable_side_effects_for_vector_push_back() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [u32; 3], v1: u1, v2: u32):
            v3 = array_get v0, index u32 0 -> u32
            v7 = make_array [Field 1, Field 2, Field 3] : [Field]
            v9 = array_set v7, index v2, value Field 4

            // this instruction should be removed
            enable_side_effects v1

            v13, v14 = call vector_push_back(u32 3, v9, Field 5) -> (u32, [Field])
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_enable_side_effects();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [u32; 3], v1: u1, v2: u32):
            v4 = array_get v0, index u32 0 -> u32
            v8 = make_array [Field 1, Field 2, Field 3] : [Field]
            v10 = array_set v8, index v2, value Field 4
            v14, v15 = call vector_push_back(u32 3, v10, Field 5) -> (u32, [Field])
            return
        }
        ");
    }

    #[test]
    fn remove_enable_side_effects_for_vector_push_front() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [u32; 3], v1: u1, v2: u32):
            v3 = array_get v0, index u32 0 -> u32
            v7 = make_array [Field 1, Field 2, Field 3] : [Field]
            v9 = array_set v7, index v2, value Field 4

            // this instruction should be removed
            enable_side_effects v1

            v13, v14 = call vector_push_front(u32 3, v9, Field 5) -> (u32, [Field])
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_enable_side_effects();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [u32; 3], v1: u1, v2: u32):
            v4 = array_get v0, index u32 0 -> u32
            v8 = make_array [Field 1, Field 2, Field 3] : [Field]
            v10 = array_set v8, index v2, value Field 4
            v14, v15 = call vector_push_front(u32 3, v10, Field 5) -> (u32, [Field])
            return
        }
        ");
    }

    #[test]
    fn keep_enable_side_effects_for_vector_pop_back() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [u32; 3], v1: u1, v2: u32):
            v3 = array_get v0, index u32 0 -> u32
            v7 = make_array [Field 1, Field 2, Field 3] : [Field]
            v9 = array_set v7, index v2, value Field 4
            enable_side_effects v1
            v13, v14, v15 = call vector_pop_back(u32 3, v9) -> (u32, [Field], Field)
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_enable_side_effects);
    }

    #[test]
    fn keep_enable_side_effects_for_vector_pop_front() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [u32; 3], v1: u1, v2: u32):
            v3 = array_get v0, index u32 0 -> u32
            v7 = make_array [Field 1, Field 2, Field 3] : [Field]
            v9 = array_set v7, index v2, value Field 4
            enable_side_effects v1
            v13, v14, v15 = call vector_pop_front(u32 3, v9) -> (Field, u32, [Field])
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_enable_side_effects);
    }

    #[test]
    fn keep_enable_side_effects_for_vector_insert() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [u32; 3], v1: u1, v2: u32):
            v3 = array_get v0, index u32 0 -> u32
            v7 = make_array [Field 1, Field 2, Field 3] : [Field]
            v9 = array_set v7, index v2, value Field 4
            enable_side_effects v1
            v13, v14 = call vector_insert(u32 3, v9, u32 1, Field 5) -> (u32, [Field])
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_enable_side_effects);
    }

    #[test]
    fn keep_enable_side_effects_for_vector_remove() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [u32; 3], v1: u1, v2: u32):
            v3 = array_get v0, index u32 0 -> u32
            v7 = make_array [Field 1, Field 2, Field 3] : [Field]
            v9 = array_set v7, index v2, value Field 4
            enable_side_effects v1
            v13, v14, v15 = call vector_remove(u32 3, v9, u32 1) -> (u32, [Field], Field)
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_enable_side_effects);
    }

    #[test]
    fn moves_enable_side_effects_just_before_instruction_affected_by_it() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [u16; 3], v1: u32, v2: u1):
            // This instruction should be moved right before the `add` instruction,
            // as it's the first instruction affected by the side effects var.
            enable_side_effects v2

            v4 = allocate -> &mut Field
            v5 = allocate -> &mut Field
            v6 = add v1, u32 1
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.remove_enable_side_effects();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [u16; 3], v1: u32, v2: u1):
            v3 = allocate -> &mut Field
            v4 = allocate -> &mut Field
            enable_side_effects v2
            v6 = add v1, u32 1
            return
        }
        "
        );
    }

    #[test]
    fn remove_enable_side_effects_that_has_no_effect() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [u16; 3], v1: u32, v2: u1, v3: u1):

            // This instruction isn't affected by any instruction so it should be removed.
            enable_side_effects v2

            v4 = allocate -> &mut Field

            // This one should still be removed to just before the `add` instruction.
            enable_side_effects v3

            v5 = allocate -> &mut Field
            v6 = add v1, u32 1
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.remove_enable_side_effects();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [u16; 3], v1: u32, v2: u1, v3: u1):
            v4 = allocate -> &mut Field
            v5 = allocate -> &mut Field
            enable_side_effects v3
            v7 = add v1, u32 1
            return
        }
        "
        );
    }

    #[test]
    fn inserts_always_true_enable_side_effects_right_away() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [u16; 3], v1: u32, v2: u1, v3: u1):
            // This instruction isn't affected by any instruction so it should be removed.
            enable_side_effects v2

            v4 = allocate -> &mut Field

            // This one should remain and in this place to maximize its effect
            enable_side_effects u1 1

            v5 = allocate -> &mut Field
            v6 = add v1, u32 1
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.remove_enable_side_effects();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [u16; 3], v1: u32, v2: u1, v3: u1):
            v4 = allocate -> &mut Field
            enable_side_effects u1 1
            v6 = allocate -> &mut Field
            v8 = add v1, u32 1
            return
        }
        "
        );
    }

    #[test]
    fn inserts_insert_side_effects_before_constrain() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: u1, v2: u32, v3: u32):
            enable_side_effects v0
            v4 = mul v2, v3
            enable_side_effects v1
            v5 = add v2, v3
            enable_side_effects v0
            constrain v4 == u32 0
            enable_side_effects u1 1
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.remove_enable_side_effects();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: u1, v2: u32, v3: u32):
            enable_side_effects v0
            v4 = mul v2, v3
            enable_side_effects v1
            v5 = add v2, v3
            enable_side_effects v0
            constrain v4 == u32 0
            enable_side_effects u1 1
            return
        }
        "
        );
    }
}
