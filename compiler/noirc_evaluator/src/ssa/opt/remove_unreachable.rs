use std::collections::BTreeSet;

use fxhash::FxHashSet as HashSet;

use crate::ssa::{
    ir::{
        function::{Function, FunctionId},
        instruction::Instruction,
        value::Value,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Removes any unreachable functions from the code. These can result from
    /// optimizations making existing functions unreachable, e.g. `if false { foo() }`,
    /// or even from monomorphizing an unconstrained version of a constrained function
    /// where the original constrained version ends up never being used.
    pub(crate) fn remove_unreachable_functions(mut self) -> Self {
        let mut used_functions = HashSet::default();

        for (id, function) in self.functions.iter() {
            // XXX: `self.is_entry_point(*id)` could leave Brillig functions that nobody calls in the SSA.
            let is_entry_point = function.id() == self.main_id
                || function.runtime().is_acir() && function.runtime().is_entry_point();

            if is_entry_point {
                collect_reachable_functions(&self, *id, &mut used_functions);
            }
        }

        self.functions.retain(|id, _| used_functions.contains(id));
        self
    }
}

fn collect_reachable_functions(
    ssa: &Ssa,
    current_func_id: FunctionId,
    reachable_functions: &mut HashSet<FunctionId>,
) {
    if reachable_functions.contains(&current_func_id) {
        return;
    }
    reachable_functions.insert(current_func_id);

    // If the debugger is used, its possible for function inlining
    // to remove functions that the debugger still references
    let Some(func) = ssa.functions.get(&current_func_id) else {
        return;
    };

    let used_functions = used_functions(func);

    for called_func_id in used_functions.iter() {
        collect_reachable_functions(ssa, *called_func_id, reachable_functions);
    }
}

fn used_functions(func: &Function) -> BTreeSet<FunctionId> {
    let mut used_function_ids = BTreeSet::default();

    let mut find_functions = |value| {
        if let Value::Function(function) = func.dfg[func.dfg.resolve(value)] {
            used_function_ids.insert(function);
        }
    };

    for block_id in func.reachable_blocks() {
        let block = &func.dfg[block_id];

        for instruction_id in block.instructions() {
            let instruction = &func.dfg[*instruction_id];

            if matches!(instruction, Instruction::Store { .. } | Instruction::Call { .. }) {
                instruction.for_each_value(&mut find_functions);
            }
        }

        block.unwrap_terminator().for_each_value(&mut find_functions);
    }

    used_function_ids
}
