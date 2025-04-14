use std::collections::VecDeque;

use acvm::{AcirField, FieldElement};
use fxhash::FxHashSet as HashSet;
use iter_extended::vecmap;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::{DataFlowGraph, InsertInstructionResult, simplify::SimplifyResult},
        function::Function,
        instruction::{
            Binary, BinaryOp, Instruction, InstructionId, binary::eval_constant_binary_op,
        },
        types::NumericType,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Map arrays with the last instruction that uses it
    /// For this we simply process all the instructions in execution order
    /// and update the map whenever there is a match
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn arithmetic_optimization(mut self) -> Self {
        for func in self.functions.values_mut() {
            func.arithmetic_optimization();
        }
        self
    }
}

impl Function {
    pub(crate) fn arithmetic_optimization(&mut self) {
        let mut context = Context::new();
        context.block_queue.push_back(self.entry_block());

        while let Some(block) = context.block_queue.pop_front() {
            if context.visited_blocks.contains(&block) {
                continue;
            }

            context.visited_blocks.insert(block);
            context.optimize_arithmetic_in_block(self, block);
        }
    }
}

struct Context {
    /// Maps pre-folded ValueIds to the new ValueIds obtained by re-inserting the instruction.
    visited_blocks: HashSet<BasicBlockId>,
    block_queue: VecDeque<BasicBlockId>,
}

impl Context {
    fn new() -> Self {
        Self { visited_blocks: Default::default(), block_queue: Default::default() }
    }

    fn optimize_arithmetic_in_block(&mut self, function: &mut Function, block_id: BasicBlockId) {
        let instructions = function.dfg[block_id].take_instructions();

        for instruction_id in instructions {
            self.fold_constants_into_instruction(function, block_id, instruction_id);
        }

        // Map a terminator in place, replacing any ValueId in the terminator with the
        // resolved version of that value id from the simplification cache's internal value mapping.
        let mut terminator = function.dfg[block_id].take_terminator();
        terminator.map_values_mut(|value| function.dfg.resolve(value));
        function.dfg[block_id].set_terminator(terminator);

        self.block_queue.extend(function.dfg[block_id].successors());
    }

    fn fold_constants_into_instruction(
        &mut self,
        function: &mut Function,
        block: BasicBlockId,
        id: InstructionId,
    ) {
        let instruction = function.dfg[id].clone();
        let old_results = function.dfg.instruction_results(id);
        let ctrl_typevars = instruction
            .requires_ctrl_typevars()
            .then(|| vecmap(old_results, |result| function.dfg.type_of_value(*result)));

        let new_instruction = match instruction {
            Instruction::Binary(binary) => {
                let binary = simplify_binary(binary.clone(), &mut function.dfg);
                match simplify_using_previous_instruction(&binary, &mut function.dfg) {
                    SimplifyResult::SimplifiedToInstruction(instruction) => instruction,
                    SimplifyResult::None => Instruction::Binary(binary),
                    _ => unreachable!("we're doing bad things"),
                }
            }
            _ => instruction,
        };

        let call_stack = function.dfg.get_instruction_call_stack_id(id);

        let new_results = match function.dfg.insert_instruction_and_results_if_simplified(
            new_instruction,
            block,
            ctrl_typevars,
            call_stack,
            Some(id),
        ) {
            InsertInstructionResult::SimplifiedTo(new_result) => vec![new_result],
            InsertInstructionResult::SimplifiedToMultiple(new_results) => new_results,
            InsertInstructionResult::Results(_, new_results) => new_results.to_vec(),
            InsertInstructionResult::InstructionRemoved => vec![],
        };
        // Optimizations while inserting the instruction should not change the number of results.
        assert_eq!(new_results.len(), 1);
    }
}

fn simplify_binary(binary: Binary, dfg: &mut DataFlowGraph) -> Binary {
    let Binary { lhs, rhs, operator } = binary;

    if operator == BinaryOp::Div {
        if let Some((rhs_value, NumericType::NativeField)) = dfg.get_numeric_constant_with_type(rhs)
        {
            let rhs = dfg.make_constant(FieldElement::one() / rhs_value, NumericType::NativeField);
            Binary { lhs, rhs, operator: BinaryOp::Mul { unchecked: false } }
        } else {
            binary
        }
    } else {
        binary
    }
}

/// This method inspects the precursor instruction for binary instructions with a constant argument,
/// where possible it will then combine the constants within the two instructions in order to flatten both operations.
///
/// # Example
///
/// Consider a program consisting of the instruction
///
/// ```md
/// v1 = add v0, u32 1
/// ```
///
/// If we insert the instruction defined as
///
/// ```md
/// v2 = lt v1, u32 9
/// ```
///
/// this can be automatically simplified to instead be
///
/// ```md
/// v2 = lt v0, u32 8
/// ```
fn simplify_using_previous_instruction(binary: &Binary, dfg: &mut DataFlowGraph) -> SimplifyResult {
    // We make some assumptions about the shape of binary instructions for simplicity, namely that any constant arguments are in the `rhs` term.
    // This allows us to define the following structure for the pair of binary instructions.
    let ((inner_lhs, inner_rhs, inner_operator), outer_rhs, outer_operator): (
        (ValueId, FieldElement, BinaryOp),
        FieldElement,
        BinaryOp,
    ) = match (&dfg[binary.lhs], &dfg[binary.rhs]) {
        (
            Value::Instruction { instruction, .. },
            Value::NumericConstant { constant: outer_constant, .. },
        ) => {
            let Instruction::Binary(Binary { lhs, rhs, operator }) = dfg[*instruction].clone()
            else {
                return SimplifyResult::None;
            };

            let Value::NumericConstant { constant: inner_constant, .. } = dfg[rhs].clone() else {
                return SimplifyResult::None;
            };

            ((lhs, inner_constant, operator), *outer_constant, binary.operator)
        }

        _ => return SimplifyResult::None,
    };

    let typ = dfg.type_of_value(inner_lhs).unwrap_numeric();

    match outer_operator {
        BinaryOp::Lt => {
            if !matches!(inner_operator, BinaryOp::Add { .. }) {
                return SimplifyResult::None;
            }

            if outer_rhs < inner_rhs {
                // Skip if performing subtraction would result in an underflow.
                return SimplifyResult::None;
            }

            let Some((new_const, new_typ)) = eval_constant_binary_op(
                outer_rhs,
                inner_rhs,
                BinaryOp::Sub { unchecked: false },
                typ,
            ) else {
                return SimplifyResult::None;
            };
            assert_eq!(typ, new_typ, "ICE: instruction type changed");

            let new_const = dfg.make_constant(new_const, typ);
            SimplifyResult::SimplifiedToInstruction(Instruction::binary(
                BinaryOp::Lt,
                inner_lhs,
                new_const,
            ))
        }

        // We can implement more of these optimizations however we only do this for a subset currently
        _ => SimplifyResult::None,
    }
}

#[cfg(test)]
mod test {
    use crate::{assert_ssa_snapshot, ssa::ssa_gen::Ssa};

    #[test]
    fn remove_constant_divisions() {
        // We want to replace any field divisions by constants with an equivalent multiplication so that we
        // perform the field inversion at compile time.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v1 = div v0, Field 2
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        assert_ssa_snapshot!(ssa.arithmetic_optimization(), @r"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2 = mul v0, Field 10944121435919637611123202872628637544274182200208017171849102093287904247809
            return
        }
        ");
    }

    #[test]
    fn cross_instruction_lt_optimization() {
        // We want to test that the calculation of `v2` is rewritten to not depend on `v1` as we can combine the
        // two constants into a new constant.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            v1 = add v0, u32 1
            v2 = lt v1, u32 9
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        // We preserve `v2` as this should be removed by the DIE optimization pass.
        assert_ssa_snapshot!(ssa.arithmetic_optimization(), @r"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2 = add v0, u32 1
            v4 = lt v0, u32 8
            return
        }
        ");
    }
}
