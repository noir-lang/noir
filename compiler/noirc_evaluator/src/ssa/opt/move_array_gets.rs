//! Move Array Gets Pass
//!
//! This is a setup optimization for the mutable array set optimization.
//! Array sets can be made mutable if the array they're setting isn't used
//! again afterward. This pass moves all array gets to their earliest possible
//! location right after their creation. This pass is not expected to yield
//! any runtime difference without the mutable array pass.
//!
//! To move an array get:
//! - We must keep the enable_side_effects variable the same
//! - It cannot move past any instructions it depends on. This includes:
//!   - The array itself, the index to set, and the enable_side_effects variable.
//!
//! This optimization has two passes:
//! 1. Traverse the function to find each array_get and remember its dependencies,
//!    draining each instruction as we go.
//! 2. Traverse the function re-inserting each instruction. When all dependencies
//!    of an array_get have been inserted we can insert the corresponding array_get itself.
//!    Since it can be expensive checking this for every instruction, this pass makes
//!    the assumption that the last dependency will always have the highest ValueId.
use fxhash::FxHashMap as HashMap;

use crate::ssa::ir::basic_block::BasicBlockId;
use crate::ssa::ir::dfg::DataFlowGraph;
use crate::ssa::ir::instruction::InstructionId;
use crate::ssa::ir::types::Type;
use crate::ssa::ir::value::{Value, ValueId};
use crate::ssa::{ir::instruction::Instruction, ssa_gen::Ssa};

impl Ssa {
    pub(crate) fn move_array_gets(mut self) -> Self {
        for func in self.functions.values_mut() {
            if !func.runtime().is_entry_point() {
                let mut reachable_blocks = func.reachable_blocks();
                assert_eq!(reachable_blocks.len(), 1, "Expected there to be 1 block remaining in Acir function for array_set optimization");

                let block = reachable_blocks.pop_first().unwrap();
                let (state, instructions) = find_array_gets(&mut func.dfg, block);
                move_array_gets(state, &mut func.dfg, block, instructions);
            }
        }
        self
    }
}

#[derive(Default)]
struct State {
    array_gets: HashMap<ValueId, Vec<ArrayGet>>,

    /// These array gets only depend on constant values or function inputs so they'd
    /// never be inserted if we're going through each instruction's outputs only.
    /// They're separated out here so they can be inserted at the top of the function instead.
    independent_array_gets: Vec<InstructionId>,
}

struct ArrayGet {
    instruction: InstructionId,
    side_effects: ValueId,
}

fn find_array_gets(dfg: &mut DataFlowGraph, block: BasicBlockId) -> (State, Vec<InstructionId>) {
    let mut state = State::default();
    let instructions = dfg[block].take_instructions();
    let mut side_effects = dfg.make_constant(1u128.into(), Type::bool());

    for instruction in &instructions {
        match &dfg[*instruction] {
            Instruction::ArrayGet { array, index } => {
                let mut last_dependency = None;
                find_last_dependency(dfg, *array, &mut last_dependency);
                find_last_dependency(dfg, *index, &mut last_dependency);
                find_last_dependency(dfg, side_effects, &mut last_dependency);

                if let Some(last_dependency) = last_dependency {
                    // Assume largest non-constant ValueId came last in the program
                    state
                        .array_gets
                        .entry(last_dependency)
                        .or_default()
                        .push(ArrayGet { instruction: *instruction, side_effects });
                } else {
                    state.independent_array_gets.push(*instruction);
                }
            }
            Instruction::EnableSideEffects { condition } => {
                side_effects = *condition;
            }
            _ => (),
        }
    }

    (state, instructions)
}

fn find_last_dependency(dfg: &DataFlowGraph, value: ValueId, current_last: &mut Option<ValueId>) {
    let value = dfg.resolve(value);
    match &dfg[value] {
        Value::Instruction { .. } => {
            if let Some(last) = *current_last {
                *current_last = Some(last.max(value));
            } else {
                *current_last = Some(value);
            }
        }
        Value::Param { .. }
        | Value::NumericConstant { .. }
        | Value::Function(_)
        | Value::Intrinsic(_)
        | Value::ForeignFunction(_) => (),
        // Need to recursively search through arrays since they contain other ValueIds
        Value::Array { array, .. } => {
            for elem in array {
                find_last_dependency(dfg, *elem, current_last);
            }
        }
    }
}

fn is_instruction_result(dfg: &DataFlowGraph, value: ValueId) -> bool {
    match &dfg[value] {
        Value::Instruction { .. } => true,
        Value::Param { .. }
        | Value::NumericConstant { .. }
        | Value::Function(_)
        | Value::Intrinsic(_)
        | Value::ForeignFunction(_) => false,
        Value::Array { array, .. } => array.iter().any(|elem| is_instruction_result(dfg, *elem)),
    }
}

fn move_array_gets(
    mut state: State,
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    instructions: Vec<InstructionId>,
) {
    let mut side_effects = dfg.make_constant(1u128.into(), Type::bool());

    for array_set in state.independent_array_gets {
        dfg[block].instructions_mut().push(array_set);
    }

    for instruction_id in instructions {
        match &dfg[instruction_id] {
            // Skip, we'll re-insert these from `state.array_gets`
            Instruction::ArrayGet { .. } => (),
            Instruction::EnableSideEffects { condition } => {
                side_effects = *condition;
                dfg[block].instructions_mut().push(instruction_id);
            }
            _ => {
                dfg[block].instructions_mut().push(instruction_id);
            }
        }

        let results = dfg.instruction_results(instruction_id);
        let mut array_gets_to_insert = Vec::new();

        for result in results {
            if let Some(mut array_gets) = state.array_gets.remove(result) {
                array_gets_to_insert.append(&mut array_gets);
            }
        }

        for array_get in array_gets_to_insert {
            if array_get.side_effects != side_effects {
                insert_side_effects_enabled(dfg, block, array_get.side_effects);
            }
            dfg[block].instructions_mut().push(array_get.instruction);
            if array_get.side_effects != side_effects {
                insert_side_effects_enabled(dfg, block, side_effects);
            }
        }
    }
}

fn insert_side_effects_enabled(dfg: &mut DataFlowGraph, block: BasicBlockId, condition: ValueId) {
    let instruction = Instruction::EnableSideEffects { condition };
    let call_stack = dfg.get_value_call_stack(condition);
    dfg.insert_instruction_and_results(instruction, block, None, call_stack);
}
