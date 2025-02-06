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
        if self.main().runtime().is_brillig() {
            return self;
        }

        let brillig_entry_points = get_brillig_entry_points(&self.functions);
        let entry_points = brillig_entry_points.keys().copied().collect::<HashSet<_>>();

        // Map for fetching the correct entry point globals when compiling any function
        let mut inner_call_to_entry_point: HashMap<FunctionId, Vec<FunctionId>> =
            HashMap::default();

        // We only need to generate globals for entry points
        for (entry_point, entry_point_inner_calls) in brillig_entry_points.iter() {
            let entry_point = *entry_point;
            for inner_call in entry_point_inner_calls {
                inner_call_to_entry_point.entry(*inner_call).or_default().push(entry_point);
            }
        }

        let mut functions_to_clone_map: HashMap<FunctionId, Vec<(Function, FunctionId)>> =
            HashMap::default();

        let mut add_function_to_clone = |entry_point: FunctionId, inner_call: FunctionId| {
            let cloned_function = Function::clone_no_id(&self.functions[&inner_call]);
            functions_to_clone_map
                .entry(entry_point)
                .or_default()
                .push((cloned_function, inner_call));
        };

        for (inner_call, inner_call_entry_points) in inner_call_to_entry_point {
            if inner_call_entry_points.len() > 1 {
                for entry_point in inner_call_entry_points {
                    add_function_to_clone(entry_point, inner_call);
                }
            } else if entry_points.contains(&inner_call) {
                add_function_to_clone(inner_call_entry_points[0], inner_call);
            }
        }

        // Maps (entry point, callee function) -> new callee function id
        let mut calls_to_update: HashMap<(FunctionId, FunctionId), FunctionId> = HashMap::default();
        for (entry_point, functions_to_clone) in functions_to_clone_map {
            for (mut cloned_function, old_id) in functions_to_clone {
                self.add_fn(|id| {
                    calls_to_update.insert((entry_point, old_id), id);
                    cloned_function.set_id(id);
                    cloned_function
                });
            }
        }

        let mut new_functions_map = HashMap::default();
        for (entry_point, inner_calls) in brillig_entry_points {
            let new_entry_point =
                new_functions_map.get(&entry_point).copied().unwrap_or(entry_point);
            let function =
                self.functions.get_mut(&new_entry_point).expect("ICE: Function does not exist");
            update_function_calls(function, entry_point, &mut new_functions_map, &calls_to_update);
            for inner_call in inner_calls {
                let new_inner_call =
                    new_functions_map.get(&inner_call).copied().unwrap_or(inner_call);
                let function =
                    self.functions.get_mut(&new_inner_call).expect("ICE: Function does not exist");
                update_function_calls(
                    function,
                    entry_point,
                    &mut new_functions_map,
                    &calls_to_update,
                );
            }
        }

        self
    }
}

fn update_function_calls(
    function: &mut Function,
    entry_point: FunctionId,
    new_functions_map: &mut HashMap<FunctionId, FunctionId>,
    // Maps (entry point, callee function) -> new callee function id
    calls_to_update: &HashMap<(FunctionId, FunctionId), FunctionId>,
) {
    for block_id in function.reachable_blocks() {
        #[allow(clippy::unnecessary_to_owned)] // clippy is wrong here
        for instruction_id in function.dfg[block_id].instructions().to_vec() {
            let instruction = function.dfg[instruction_id].clone();
            let Instruction::Call { func: func_id, arguments } = instruction else {
                continue;
            };

            let func_value = &function.dfg[func_id];
            let Value::Function(func_id) = func_value else { continue };

            if let Some(new_id) = calls_to_update.get(&(entry_point, *func_id)) {
                new_functions_map.insert(*func_id, *new_id);
                let new_function_value_id = function.dfg.import_function(*new_id);
                function.dfg[instruction_id] =
                    Instruction::Call { func: new_function_value_id, arguments: arguments.clone() };
            } else {
                continue;
            }
        }
    }
}
