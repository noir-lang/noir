use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
    brillig::brillig_gen::brillig_globals::get_brillig_entry_points,
    ssa::{
        ir::{
            function::{Function, FunctionId},
            instruction::Instruction,
            value::Value,
        },
        Ssa,
    },
};

impl Ssa {
    pub(crate) fn duplicate_reused_entry_points(mut self) -> Ssa {
        let brillig_entry_points = get_brillig_entry_points(&self.functions);
        let entry_points = brillig_entry_points.keys().copied().collect::<HashSet<_>>();

        let mut calls_to_update: HashMap<FunctionId, FunctionId> = HashMap::default();
        let mut functions_to_update: HashSet<FunctionId> = HashSet::default();

        let mut functions_to_clone_map: HashMap<FunctionId, Vec<(Function, FunctionId)>> =
            HashMap::default();
        for (entry_point, function) in self.functions.iter() {
            for block in function.reachable_blocks() {
                for &instruction_id in function.dfg[block].instructions() {
                    let instruction = &function.dfg[instruction_id];
                    let Instruction::Call { func: func_id, .. } = instruction else {
                        continue;
                    };

                    let func_value = &function.dfg[*func_id];
                    let Value::Function(called_func_id) = func_value else { continue };

                    if function.dfg.runtime().is_brillig()
                        && entry_points.contains(called_func_id)
                        && *called_func_id != function.id()
                    {
                        let cloned_function =
                            Function::clone_no_id(&self.functions[called_func_id]);
                        functions_to_clone_map
                            .entry(*entry_point)
                            .or_default()
                            .push((cloned_function, *called_func_id));
                    }
                }
            }
        }

        for (entry_point, functions_to_clone) in functions_to_clone_map {
            for (mut cloned_function, old_id) in functions_to_clone {
                if calls_to_update.get(&old_id).is_some() {
                    continue;
                }
                self.add_fn(|id| {
                    calls_to_update.insert(old_id, id);

                    cloned_function.set_id(id);

                    functions_to_update.insert(entry_point);

                    cloned_function
                });
            }
        }

        for func_id in functions_to_update {
            let function = self.functions.get_mut(&func_id).expect("ICE: Function does not exist");
            for block_id in function.reachable_blocks() {
                #[allow(clippy::unnecessary_to_owned)] // clippy is wrong here
                for instruction_id in function.dfg[block_id].instructions().to_vec() {
                    let instruction = function.dfg[instruction_id].clone();
                    let Instruction::Call { func: func_id, arguments } = instruction else {
                        continue;
                    };

                    let func_value = &function.dfg[func_id];
                    let Value::Function(func_id) = func_value else { continue };

                    if let Some(new_id) = calls_to_update.get(func_id) {
                        let new_function_value_id = function.dfg.import_function(*new_id);
                        function.dfg[instruction_id] = Instruction::Call {
                            func: new_function_value_id,
                            arguments: arguments.clone(),
                        };
                    } else {
                        continue;
                    }
                }
            }
        }

        self
    }
}
