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
        dfg::DataFlowGraph,
        function::{Function, RuntimeType},
        instruction::{BinaryOp, Hint, Instruction, Intrinsic},
        types::NumericType,
        value::Value,
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
            if responds_to_side_effects_var(context.dfg, instruction) {
                if let Some(enable_side_effects_instruction_id) =
                    last_side_effects_enabled_instruction.take()
                {
                    context.dfg[context.block_id]
                        .insert_instruction(enable_side_effects_instruction_id);
                }
            }
        });
    }
}

fn responds_to_side_effects_var(dfg: &DataFlowGraph, instruction: &Instruction) -> bool {
    use Instruction::*;
    match instruction {
        Binary(binary) => match binary.operator {
            BinaryOp::Add { .. } | BinaryOp::Sub { .. } | BinaryOp::Mul { .. } => {
                dfg.type_of_value(binary.lhs).is_unsigned()
            }
            BinaryOp::Div | BinaryOp::Mod => {
                if let Some(rhs) = dfg.get_numeric_constant(binary.rhs) {
                    rhs == FieldElement::zero()
                } else {
                    true
                }
            }
            _ => false,
        },

        Cast(_, _)
        | Not(_)
        | Truncate { .. }
        | Constrain(..)
        | ConstrainNotEqual(..)
        | RangeCheck { .. }
        | IfElse { .. }
        | IncrementRc { .. }
        | DecrementRc { .. }
        | Noop
        | MakeArray { .. } => false,

        EnableSideEffectsIf { .. }
        | ArrayGet { .. }
        | ArraySet { .. }
        | Allocate
        | Store { .. }
        | Load { .. } => true,

        // Some `Intrinsic`s have side effects so we must check what kind of `Call` this is.
        Call { func, .. } => match dfg[*func] {
            Value::Intrinsic(intrinsic) => match intrinsic {
                Intrinsic::SlicePushBack
                | Intrinsic::SlicePushFront
                | Intrinsic::SlicePopBack
                | Intrinsic::SlicePopFront
                | Intrinsic::SliceInsert
                | Intrinsic::SliceRemove => true,

                Intrinsic::ArrayLen
                | Intrinsic::ArrayAsStrUnchecked
                | Intrinsic::AssertConstant
                | Intrinsic::StaticAssert
                | Intrinsic::ApplyRangeConstraint
                | Intrinsic::StrAsBytes
                | Intrinsic::ToBits(_)
                | Intrinsic::ToRadix(_)
                | Intrinsic::BlackBox(_)
                | Intrinsic::Hint(Hint::BlackBox)
                | Intrinsic::AsSlice
                | Intrinsic::AsWitness
                | Intrinsic::IsUnconstrained
                | Intrinsic::DerivePedersenGenerators
                | Intrinsic::ArrayRefCount
                | Intrinsic::SliceRefCount
                | Intrinsic::FieldLessThan => false,
            },

            // We must assume that functions contain a side effect as we cannot inspect more deeply.
            Value::Function(_) => true,

            _ => false,
        },
    }
}

#[cfg(test)]
mod test {
    use crate::{assert_ssa_snapshot, ssa::ssa_gen::Ssa};

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
          b0(v0: Field, v1: Field):
            enable_side_effects v0
            v3 = allocate -> &mut Field
            enable_side_effects v0
            v4 = allocate -> &mut Field
            enable_side_effects v1
            v5 = allocate -> &mut Field
            enable_side_effects u1 1
            v6 = allocate -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.remove_enable_side_effects();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            enable_side_effects v0
            v2 = allocate -> &mut Field
            v3 = allocate -> &mut Field
            enable_side_effects v1
            v4 = allocate -> &mut Field
            enable_side_effects u1 1
            v6 = allocate -> &mut Field
            return
        }
        "
        );
    }
}
