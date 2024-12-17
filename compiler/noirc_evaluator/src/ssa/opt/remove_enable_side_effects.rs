//! The goal of the "remove enable side effects" optimization pass is to delay any [Instruction::EnableSideEffectsIf]
//! instructions such that they cover the minimum number of instructions possible.
//!
//! The pass works as follows:
//! - Insert instructions until an [Instruction::EnableSideEffectsIf] is encountered, save this [InstructionId].
//! - Continue inserting instructions until either
//!     - Another [Instruction::EnableSideEffectsIf] is encountered, if so then drop the previous [InstructionId] in favour
//!       of this one.
//!     - An [Instruction] with side-effects is encountered, if so then insert the currently saved [Instruction::EnableSideEffectsIf]
//!       before the [Instruction]. Continue inserting instructions until the next [Instruction::EnableSideEffectsIf] is encountered.
use std::collections::HashSet;

use acvm::{acir::AcirField, FieldElement};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
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

        let mut context = Context::default();
        context.block_queue.push(self.entry_block());

        while let Some(block) = context.block_queue.pop() {
            if context.visited_blocks.contains(&block) {
                continue;
            }

            context.visited_blocks.insert(block);
            context.remove_enable_side_effects_in_block(self, block);
        }
    }
}

#[derive(Default)]
struct Context {
    visited_blocks: HashSet<BasicBlockId>,
    block_queue: Vec<BasicBlockId>,
}

impl Context {
    fn remove_enable_side_effects_in_block(
        &mut self,
        function: &mut Function,
        block: BasicBlockId,
    ) {
        let instructions = function.dfg[block].take_instructions();

        let one = FieldElement::one();
        let mut active_condition = function.dfg.make_constant(one, NumericType::bool());
        let mut last_side_effects_enabled_instruction = None;

        let mut new_instructions = Vec::with_capacity(instructions.len());
        for instruction_id in instructions {
            let instruction = &function.dfg[instruction_id];

            // If we run into another `Instruction::EnableSideEffectsIf` before encountering any
            // instructions with side effects then we can drop the instruction we're holding and
            // continue with the new `Instruction::EnableSideEffectsIf`.
            if let Instruction::EnableSideEffectsIf { condition } = instruction {
                // If this instruction isn't changing the currently active condition then we can ignore it.
                if active_condition == *condition {
                    continue;
                }

                // If we're seeing an `enable_side_effects u1 1` then we want to insert it immediately.
                // This is because we want to maximize the effect it will have.
                let condition_is_one = function
                    .dfg
                    .get_numeric_constant(*condition)
                    .map_or(false, |condition| condition.is_one());
                if condition_is_one {
                    new_instructions.push(instruction_id);
                    last_side_effects_enabled_instruction = None;
                    active_condition = *condition;
                    continue;
                }

                last_side_effects_enabled_instruction = Some(instruction_id);
                active_condition = *condition;
                continue;
            }

            // If we hit an instruction which is affected by the side effects var then we must insert the
            // `Instruction::EnableSideEffectsIf` before we insert this new instruction.
            if Self::responds_to_side_effects_var(&function.dfg, instruction) {
                if let Some(enable_side_effects_instruction_id) =
                    last_side_effects_enabled_instruction.take()
                {
                    new_instructions.push(enable_side_effects_instruction_id);
                }
            }
            new_instructions.push(instruction_id);
        }

        *function.dfg[block].instructions_mut() = new_instructions;

        self.block_queue.extend(function.dfg[block].successors());
    }

    fn responds_to_side_effects_var(dfg: &DataFlowGraph, instruction: &Instruction) -> bool {
        use Instruction::*;
        match instruction {
            Binary(binary) => match binary.operator {
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul => {
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
            | RangeCheck { .. }
            | IfElse { .. }
            | IncrementRc { .. }
            | DecrementRc { .. }
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
}

#[cfg(test)]
mod test {

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            instruction::{BinaryOp, Instruction},
            map::Id,
            types::{NumericType, Type},
        },
    };

    #[test]
    fn remove_chains_of_same_condition() {
        //  acir(inline) fn main f0 {
        //    b0(v0: Field):
        //      enable_side_effects u1 1
        //      v4 = mul v0, Field 2
        //      enable_side_effects u1 1
        //      v5 = mul v0, Field 2
        //      enable_side_effects u1 1
        //      v6 = mul v0, Field 2
        //      enable_side_effects u1 1
        //      v7 = mul v0, Field 2
        //      enable_side_effects u1 1
        //      (no terminator instruction)
        //  }
        //
        // After constructing this IR, we run constant folding which should replace the second cast
        // with a reference to the results to the first. This then allows us to optimize away
        // the constrain instruction as both inputs are known to be equal.
        //
        // The first cast instruction is retained and will be removed in the dead instruction elimination pass.
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::field());

        let two = builder.field_constant(2u128);

        let one = builder.numeric_constant(1u128, NumericType::bool());

        builder.insert_enable_side_effects_if(one);
        builder.insert_binary(v0, BinaryOp::Mul, two);
        builder.insert_enable_side_effects_if(one);
        builder.insert_binary(v0, BinaryOp::Mul, two);
        builder.insert_enable_side_effects_if(one);
        builder.insert_binary(v0, BinaryOp::Mul, two);
        builder.insert_enable_side_effects_if(one);
        builder.insert_binary(v0, BinaryOp::Mul, two);
        builder.insert_enable_side_effects_if(one);

        let ssa = builder.finish();

        println!("{ssa}");

        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 9);

        // Expected output:
        //
        // acir(inline) fn main f0 {
        //     b0(v0: Field):
        //       v3 = mul v0, Field 2
        //       v4 = mul v0, Field 2
        //       v5 = mul v0, Field 2
        //       v6 = mul v0, Field 2
        //       (no terminator instruction)
        //   }
        let ssa = ssa.remove_enable_side_effects();

        println!("{ssa}");

        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();

        assert_eq!(instructions.len(), 4);
        for instruction in instructions.iter().take(4) {
            assert_eq!(&main.dfg[*instruction], &Instruction::binary(BinaryOp::Mul, v0, two));
        }
    }
}
