use std::collections::HashSet;

use crate::ssa::ir::{
    basic_block::BasicBlock,
    dfg::DataFlowGraph,
    function::Function,
    instruction::{Instruction, InstructionId, Intrinsic},
    value::{Value, ValueId},
};

/// Returns true if an instruction can be removed.
///
/// An instruction can be removed as long as it has no side-effects, and none of its result
/// values have been referenced.
pub(crate) fn is_unused(
    used_values: &HashSet<ValueId>,
    instruction_id: InstructionId,
    function: &Function,
) -> bool {
    let instruction = &function.dfg[instruction_id];

    if instruction.can_eliminate_if_unused(&function.dfg) {
        let results = function.dfg.instruction_results(instruction_id);
        results.iter().all(|result| !used_values.contains(result))
    } else if let Instruction::Call { func, arguments } = instruction {
        // TODO: make this more general for instructions which don't have results but have side effects "sometimes" like `Intrinsic::AsWitness`
        let as_witness_id = function.dfg.get_intrinsic(Intrinsic::AsWitness);
        as_witness_id == Some(func) && !used_values.contains(&arguments[0])
    } else {
        // If the instruction has side effects we should never remove it.
        false
    }
}

/// Adds values referenced by the terminator to the set of used values.
pub(crate) fn mark_terminator_values_as_used(
    used_values: &mut HashSet<ValueId>,
    function: &Function,
    block: &BasicBlock,
) {
    block.unwrap_terminator().for_each_value(|value| {
        mark_used_instruction_results(used_values, &function.dfg, value);
    });
}

/// Inspects a value recursively (as it could be an array) and marks all comprised instruction
/// results as used.
pub(crate) fn mark_used_instruction_results(
    used_values: &mut HashSet<ValueId>,
    dfg: &DataFlowGraph,
    value_id: ValueId,
) {
    let value_id = dfg.resolve(value_id);
    match &dfg[value_id] {
        Value::Instruction { .. } => {
            used_values.insert(value_id);
        }
        Value::Array { array, .. } => {
            for elem in array {
                mark_used_instruction_results(used_values, dfg, *elem);
            }
        }
        Value::Param { .. } => {
            used_values.insert(value_id);
        }
        _ => {
            // Does not comprise of any instruction results
        }
    }
}
