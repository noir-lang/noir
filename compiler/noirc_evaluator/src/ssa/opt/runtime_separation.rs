use std::collections::BTreeSet;

use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    ir::{
        function::{Function, FunctionId, RuntimeType},
        instruction::Instruction,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// This optimization step separates the runtime of the functions in the SSA.
    /// After this step, all functions with runtime `Acir` will be converted to Acir and
    /// the functions with runtime `Brillig` will be converted to Brillig.
    /// It does so by cloning all ACIR functions called from a Brillig context
    /// and changing the runtime of the cloned functions to Brillig.
    /// This pass needs to run after functions as values have been resolved (defunctionalization).
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn separate_runtime(mut self) -> Self {
        RuntimeSeparatorContext::separate_runtime(&mut self);

        self
    }
}

#[derive(Debug, Default)]
struct RuntimeSeparatorContext {
    // Original functions to clone to brillig
    acir_functions_called_from_brillig: BTreeSet<FunctionId>,
    // Tracks the original => cloned version
    mapped_functions: HashMap<FunctionId, FunctionId>,
}

impl RuntimeSeparatorContext {
    pub(crate) fn separate_runtime(ssa: &mut Ssa) {
        let mut runtime_separator = RuntimeSeparatorContext::default();

        // We first collect all the acir functions called from a brillig context by exploring the SSA recursively
        let mut processed_functions = HashSet::default();
        runtime_separator.collect_acir_functions_called_from_brillig(
            ssa,
            ssa.main_id,
            false,
            &mut processed_functions,
        );

        // Now we clone the relevant acir functions and change their runtime to brillig
        runtime_separator.convert_acir_functions_called_from_brillig_to_brillig(ssa);

        // Now we update any calls within a brillig context to the mapped functions
        runtime_separator.replace_calls_to_mapped_functions(ssa);

        // Some functions might be unreachable now (for example an acir function only called from brillig)
        prune_unreachable_functions(ssa);
    }

    fn collect_acir_functions_called_from_brillig(
        &mut self,
        ssa: &Ssa,
        current_func_id: FunctionId,
        mut within_brillig: bool,
        processed_functions: &mut HashSet<(/* within_brillig */ bool, FunctionId)>,
    ) {
        // Processed functions needs the within brillig flag, since it is possible to call the same function from both brillig and acir
        if processed_functions.contains(&(within_brillig, current_func_id)) {
            return;
        }
        processed_functions.insert((within_brillig, current_func_id));

        let func = ssa.functions.get(&current_func_id).expect("Function should exist in SSA");
        if func.runtime() == RuntimeType::Brillig {
            within_brillig = true;
        }

        let called_functions = called_functions(func);

        if within_brillig {
            for called_func_id in called_functions.iter() {
                let called_func =
                    ssa.functions.get(called_func_id).expect("Function should exist in SSA");
                if matches!(called_func.runtime(), RuntimeType::Acir(_)) {
                    self.acir_functions_called_from_brillig.insert(*called_func_id);
                }
            }
        }

        for called_func_id in called_functions.into_iter() {
            self.collect_acir_functions_called_from_brillig(
                ssa,
                called_func_id,
                within_brillig,
                processed_functions,
            );
        }
    }

    fn convert_acir_functions_called_from_brillig_to_brillig(&mut self, ssa: &mut Ssa) {
        for acir_func_id in self.acir_functions_called_from_brillig.iter() {
            let cloned_id = ssa.clone_fn(*acir_func_id);
            let new_func =
                ssa.functions.get_mut(&cloned_id).expect("Cloned function should exist in SSA");
            new_func.set_runtime(RuntimeType::Brillig);
            self.mapped_functions.insert(*acir_func_id, cloned_id);
        }
    }

    fn replace_calls_to_mapped_functions(&mut self, ssa: &mut Ssa) {
        for (_function_id, func) in ssa.functions.iter_mut() {
            if func.runtime() == RuntimeType::Brillig {
                for called_func_value_id in called_functions_values(func).iter() {
                    let Value::Function(called_func_id) = &func.dfg[*called_func_value_id] else {
                        unreachable!("Value should be a function")
                    };
                    if let Some(mapped_func_id) = self.mapped_functions.get(called_func_id) {
                        let new_target_value = Value::Function(*mapped_func_id);
                        let mapped_value_id = func.dfg.make_value(new_target_value);
                        func.dfg.set_value_from_id(*called_func_value_id, mapped_value_id);
                    }
                }
            }
        }
    }
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

fn collect_reachable_functions(
    ssa: &Ssa,
    current_func_id: FunctionId,
    reachable_functions: &mut HashSet<FunctionId>,
) {
    if reachable_functions.contains(&current_func_id) {
        return;
    }
    reachable_functions.insert(current_func_id);

    let func = ssa.functions.get(&current_func_id).expect("Function should exist in SSA");
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

#[cfg(test)]
mod test {
    use noirc_frontend::monomorphization::ast::InlineType;

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{function::RuntimeType, map::Id, types::Type},
    };

    #[test]
    fn basic_runtime_separation() {
        // brillig fn foo {
        //   b0():
        //     v0 = call bar()
        //     return v0
        // }
        // acir fn bar {
        //   b0():
        //     return 72
        // }
        let foo_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("foo".into(), foo_id);
        builder.current_function.set_runtime(RuntimeType::Brillig);

        let bar_id = Id::test_new(1);
        let bar = builder.import_function(bar_id);
        let results = builder.insert_call(bar, Vec::new(), vec![Type::field()]).to_vec();
        builder.terminate_with_return(results);

        builder.new_function("bar".into(), bar_id, InlineType::default());
        let expected_return = 72u128;
        let seventy_two = builder.field_constant(expected_return);
        builder.terminate_with_return(vec![seventy_two]);

        let ssa = builder.finish();
        assert_eq!(ssa.functions.len(), 2);

        let separated = ssa.separate_runtime();

        // The original bar function must have been pruned
        assert_eq!(separated.functions.len(), 2);

        // All functions should be brillig now
        for func in separated.functions.values() {
            assert_eq!(func.runtime(), RuntimeType::Brillig);
        }
    }
}
