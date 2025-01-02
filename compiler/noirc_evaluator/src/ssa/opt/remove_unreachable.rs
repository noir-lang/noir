use std::collections::BTreeSet;

use fxhash::FxHashSet as HashSet;

use crate::ssa::{
    ir::{
        function::{Function, FunctionId},
        instruction::Instruction,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Removes any unreachable functions from the code. These can result from
    /// optimizations making existing functions unreachable, e.g. `if false { foo() }`,
    /// or even from monomorphizing an unconstrained version of a constrained function
    /// where the original constrained version ends up never being used.
    ///
    /// This pass only checks functions that are called directly and thus depends on
    /// the defunctionalize pass to be called beforehand.
    pub(crate) fn remove_unreachable_functions(mut self) -> Self {
        let mut reachable_functions = HashSet::default();

        for function_id in self.functions.keys() {
            if self.is_entry_point(*function_id) {
                collect_reachable_functions(&self, *function_id, &mut reachable_functions);
            }
        }

        prune_unreachable_functions(&mut self);
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

    let func = &ssa.functions[&current_func_id];
    let called_functions = called_functions(func);

    for called_func_id in called_functions.iter() {
        collect_reachable_functions(ssa, *called_func_id, reachable_functions);
    }
}

fn prune_unreachable_functions(ssa: &mut Ssa) {
    let mut reachable_functions = HashSet::default();
    collect_reachable_functions(ssa, ssa.main_id, &mut reachable_functions);

    ssa.functions.retain(|id, _value| reachable_functions.contains(id));
}

// We only consider direct calls to functions since functions as values should have been resolved
fn called_functions_values(func: &Function) -> BTreeSet<ValueId> {
    let mut called_function_ids = BTreeSet::default();
    for block_id in func.reachable_blocks() {
        for instruction_id in func.dfg[block_id].instructions() {
            let Instruction::Call { func: called_value_id, .. } = &func.dfg[*instruction_id] else {
                continue;
            };

            if let Value::Function(_) = func.dfg[*called_value_id] {
                called_function_ids.insert(*called_value_id);
            }
        }
    }

    called_function_ids
}

fn called_functions(func: &Function) -> BTreeSet<FunctionId> {
    called_functions_values(func)
        .into_iter()
        .map(|value_id| {
            let Value::Function(func_id) = func.dfg[value_id] else {
                unreachable!("Value should be a function")
            };
            func_id
        })
        .collect()
}
