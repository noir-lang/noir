//! The goal of the "remove enable side effects" optimization pass is to delay any [Instruction::EnableSideEffectsIf]
//! instructions such that they cover the minimum number of instructions possible.
//!
//! The pass works as follows:
//! - Insert instructions until an [Instruction::EnableSideEffectsIf] is encountered, save this [InstructionId][crate::ssa::ir::instruction::InstructionId].
//! - Continue inserting instructions until either
//!     - Another [Instruction::EnableSideEffectsIf] is encountered, if so then drop the previous [InstructionId][crate::ssa::ir::instruction::InstructionId] in favour
//!       of this one.
//!     - An [Instruction] with side-effects is encountered, if so then insert the currently saved [Instruction::EnableSideEffectsIf]
//!       before the [Instruction]. Continue inserting instructions until the next [Instruction::EnableSideEffectsIf] is encountered.
//!

use acvm::{FieldElement, acir::AcirField};

use crate::ssa::{
    ir::{
        function::{Function, RuntimeType},
        instruction::Instruction,
        types::NumericType,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// See [`remove_enable_side_effects`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn remove_enable_side_effects(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.remove_enable_side_effects();
        }
        self
    }
}

impl Function {
    pub(crate) fn remove_enable_side_effects(&mut self) {
        if matches!(self.runtime(), RuntimeType::Brillig(_)) {
            // Brillig functions do not make use of the `EnableSideEffects` instruction so are unaffected by this pass.
            return;
        }

        // Make sure this optimization runs when there's only one block
        let block = self.entry_block();
        assert_eq!(self.dfg[block].successors().count(), 0);

        let one = self.dfg.make_constant(FieldElement::one(), NumericType::bool());
        let mut active_condition = one;
        let mut last_side_effects_enabled_instruction = None;

        let mut previous_block = None;

        self.simple_reachable_blocks_optimization(|context| {
            if Some(context.block_id) != previous_block {
                active_condition = one;
                last_side_effects_enabled_instruction = None;
            }
            previous_block = Some(context.block_id);

            let instruction_id = context.instruction_id;
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

                // If we're seeing an `enable_side_effects u1 1` then we want to insert it immediately.
                // This is because we want to maximize the effect it will have.
                let condition_is_one = context
                    .dfg
                    .get_numeric_constant(*condition)
                    .is_some_and(|condition| condition.is_one());
                if condition_is_one {
                    last_side_effects_enabled_instruction = None;
                    active_condition = *condition;
                    return;
                }

                last_side_effects_enabled_instruction = Some(instruction_id);
                active_condition = *condition;
                context.remove_current_instruction();
                return;
            }

            // If we hit an instruction which is affected by the side effects var then we must insert the
            // `Instruction::EnableSideEffectsIf` before we insert this new instruction.
            if instruction.has_side_effects(context.dfg) {
                if let Some(enable_side_effects_instruction_id) =
                    last_side_effects_enabled_instruction.take()
                {
                    context.insert_instruction(enable_side_effects_instruction_id);
                }
            }
        });
    }
}

#[cfg(test)]
mod test {
    use crate::{
        assert_ssa_snapshot,
        ssa::{
            ir::{
                function::Function,
                instruction::{Instruction, Intrinsic},
            },
            ssa_gen::Ssa,
        },
    };

    #[test]
    fn remove_chains_of_same_condition() {
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
    fn keeps_enable_side_effects_for_instructions_that_have_side_effects() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [u16; 3], v1: u32, v2: u32):
            enable_side_effects v1
            v3 = array_get v0, index v1 -> u16
            enable_side_effects v1
            v4 = array_get v0, index v1 -> u16
            enable_side_effects v2
            v5 = array_get v0, index v1 -> u16
            enable_side_effects u1 1
            v7 = array_get v0, index v1 -> u16
            return
        }";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.remove_enable_side_effects();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [u16; 3], v1: u32, v2: u32):
            enable_side_effects v1
            v3 = array_get v0, index v1 -> u16
            v4 = array_get v0, index v1 -> u16
            enable_side_effects v2
            v5 = array_get v0, index v1 -> u16
            enable_side_effects u1 1
            v7 = array_get v0, index v1 -> u16
            return
        }");
    }

    #[test]
    fn keep_enable_side_effects_for_safe_modulo() {
        // This is a simplification of `test_programs/execution_success/regression_8236`
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [u16; 3], v1: [u1; 1]):
            v4 = call f1(v0, u1 1) -> [u1; 1]
            v6 = array_get v0, index u32 0 -> u16
            v7 = cast v6 as u32
            v8 = array_get v4, index u32 0 -> u1
            enable_side_effects v8
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

        // We expect the SSA to be unchanged
        assert_ssa_snapshot!(ssa, @r#"
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
        "#);
    }

    #[test]
    fn keep_side_effects_for_safe_div() {
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
        // We expect the SSA to be unchanged
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [u32; 3], v1: u1):
            v3 = array_get v0, index u32 0 -> u32
            enable_side_effects v1
            v5 = div v3, u32 3
            return
        }
        "#);
    }

    #[test]
    fn remove_enable_side_effects_for_slice_push_back() {
        let src = get_slice_intrinsic_src(
            "v13, v14",
            &Intrinsic::SlicePushBack.to_string(),
            ", Field 5) -> (u32, [Field])",
        );
        verify_all_enable_side_effects_removed(&src);
    }

    #[test]
    fn remove_enable_side_effects_for_slice_push_front() {
        let src = get_slice_intrinsic_src(
            "v13, v14",
            &Intrinsic::SlicePushFront.to_string(),
            ", Field 5) -> (u32, [Field])",
        );
        verify_all_enable_side_effects_removed(&src);
    }

    #[test]
    fn remove_enable_side_effects_for_slice_pop_back() {
        let src = get_slice_intrinsic_src(
            "v13, v14, v15",
            &Intrinsic::SlicePopBack.to_string(),
            ") -> (u32, [Field], Field)",
        );
        verify_all_enable_side_effects_removed(&src);
    }

    #[test]
    fn remove_enable_side_effects_for_slice_pop_front() {
        let src = get_slice_intrinsic_src(
            "v13, v14, v15",
            &Intrinsic::SlicePopFront.to_string(),
            ") -> (Field, u32, [Field])",
        );
        verify_all_enable_side_effects_removed(&src);
    }

    #[test]
    fn keep_enable_side_effects_for_slice_insert() {
        let src = get_slice_intrinsic_src(
            "v13, v14",
            &Intrinsic::SliceInsert.to_string(),
            ", u32 1, Field 5) -> (u32, [Field])",
        );
        verify_ssa_unchanged(&src);
    }

    #[test]
    fn keep_enable_side_effects_for_slice_remove() {
        let src = get_slice_intrinsic_src(
            "v13, v14, v15",
            &Intrinsic::SliceRemove.to_string(),
            ", u32 1) -> (u32, [Field], Field)",
        );
        verify_ssa_unchanged(&src);
    }

    fn verify_all_enable_side_effects_removed(src: &str) {
        let ssa = Ssa::from_str(src).unwrap();
        let num_enable_side_effects = num_enable_side_effects_instructions(ssa.main());
        assert!(
            num_enable_side_effects >= 1,
            "Should have at least one EnableSideEffectsIf instruction"
        );
        let expected_total_instructions = ssa.main().num_instructions() - num_enable_side_effects;

        let ssa = ssa.remove_enable_side_effects();

        let num_enable_side_effects = num_enable_side_effects_instructions(ssa.main());
        assert_eq!(
            num_enable_side_effects, 0,
            "Should not have any EnableSideEffectsIf instructions"
        );
        assert_eq!(ssa.main().num_instructions(), expected_total_instructions);
    }

    fn verify_ssa_unchanged(src: &str) {
        let ssa = Ssa::from_str(src).unwrap();
        let num_enable_side_effects = num_enable_side_effects_instructions(ssa.main());
        assert!(
            num_enable_side_effects >= 1,
            "Should have at least one EnableSideEffectsIf instruction"
        );
        let expected_total_instructions = ssa.main().num_instructions();

        let ssa = ssa.remove_enable_side_effects();

        let ssa = ssa.remove_enable_side_effects();
        let got_num_enable_side_effects = num_enable_side_effects_instructions(ssa.main());
        assert_eq!(
            got_num_enable_side_effects, num_enable_side_effects,
            "Should not same number of EnableSideEffectsIf instructions"
        );
        assert_eq!(ssa.main().num_instructions(), expected_total_instructions);
    }

    // Helper method to set up the SSA for unit tests on slice intrinsics
    fn get_slice_intrinsic_src(
        return_values: &str,
        intrinsic_name: &str,
        args_and_return_type: &str,
    ) -> String {
        format!(
            "
        acir(inline) predicate_pure fn main f0 {{
          b0(v0: u32, v1: u1):
            v3 = array_get v0, index u32 0 -> u32
            v7 = make_array [Field 1, Field 2, Field 3] : [Field]
            v9 = array_set v7, index v0, value Field 4
            enable_side_effects v1
            {return_values} = call {intrinsic_name}(u32 3, v9{args_and_return_type}
            return
        }}
        "
        )
    }

    fn num_enable_side_effects_instructions(function: &Function) -> usize {
        function
            .reachable_blocks()
            .iter()
            .flat_map(|block| function.dfg[*block].instructions())
            .filter(|inst| matches!(function.dfg[**inst], Instruction::EnableSideEffectsIf { .. }))
            .count()
    }
}
