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
        for (entry_point, inner_calls) in brillig_entry_points {
            for inner_call in inner_calls {
                if entry_points.contains(&inner_call) {
                    if calls_to_update.get(&inner_call).is_some() {
                        functions_to_update.insert(entry_point);
                        continue;
                    }
                    // Must clone the function before `add_fn` as the method borrows `self` mutably
                    // while `clone_no_id` borrows a function immutably.
                    let mut cloned_function = Function::clone_no_id(&self.functions[&inner_call]);

                    self.add_fn(|id| {
                        cloned_function.set_id(id);

                        calls_to_update.insert(inner_call, id);
                        functions_to_update.insert(entry_point);

                        cloned_function
                    });
                }
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
