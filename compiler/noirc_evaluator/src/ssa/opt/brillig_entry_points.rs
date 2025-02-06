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
        // dbg!(brillig_entry_points.clone());
        // dbg!(entry_points.clone());

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

        dbg!(inner_call_to_entry_point.clone());

        // Determine the number of times a function is called in different Brillig entry points
        // let entry_point_call_counts = brillig_entry_points
        //     .values()
        //     .flat_map(|set| set.iter())
        //     .fold(HashMap::default(), |mut counts, &function_id| {
        //         *counts.entry(function_id).or_insert(0) += 1;
        //         counts
        //     });
        // dbg!(entry_point_call_counts.clone());

        let mut calls_to_update_w_entry: HashMap<(FunctionId, FunctionId), FunctionId> =
            HashMap::default();
        let mut functions_to_clone_map: HashMap<FunctionId, Vec<(Function, FunctionId)>> =
            HashMap::default();

        for (inner_call, inner_call_entry_points) in inner_call_to_entry_point {
            if inner_call_entry_points.len() > 1 {
                for entry_point in inner_call_entry_points {
                    let cloned_function = self.functions[&inner_call].clone();
                    functions_to_clone_map
                        .entry(entry_point)
                        .or_default()
                        .push((cloned_function, inner_call));
                }
            } else if entry_points.contains(&inner_call) {
                // else {
                // dbg!(inner_call);
                let entry_point = inner_call_entry_points[0];
                // dbg!(entry_point);
                let cloned_function = self.functions[&inner_call].clone();
                functions_to_clone_map
                    .entry(entry_point)
                    .or_default()
                    .push((cloned_function, inner_call));
            }
        }

        for (entry_point, functions_to_clone) in functions_to_clone_map {
            for (mut cloned_function, old_id) in functions_to_clone {
                self.add_fn(|id| {
                    calls_to_update_w_entry.insert((entry_point, old_id), id);
                    // println!("entry point {entry_point}, old_id {old_id}, new_id {id}");
                    cloned_function.set_id(id);

                    cloned_function
                });
            }
        }

        // dbg!(calls_to_update_w_entry.clone());

        let mut new_functions_map = HashMap::default();
        for (entry_point, inner_calls) in brillig_entry_points {
            let new_entry_point =
                new_functions_map.get(&entry_point).copied().unwrap_or(entry_point);
            // println!("old entry point {entry_point}, new {new_entry_point}");
            let function =
                self.functions.get_mut(&new_entry_point).expect("ICE: Function does not exist");
            for block_id in function.reachable_blocks() {
                #[allow(clippy::unnecessary_to_owned)] // clippy is wrong here
                for instruction_id in function.dfg[block_id].instructions().to_vec() {
                    let instruction = function.dfg[instruction_id].clone();
                    let Instruction::Call { func: func_id, arguments } = instruction else {
                        continue;
                    };

                    let func_value = &function.dfg[func_id];
                    let Value::Function(func_id) = func_value else { continue };

                    // println!("call to {func_id} in entry {entry_point}");
                    if let Some(new_id) = calls_to_update_w_entry.get(&(entry_point, *func_id)) {
                        println!("inserting {new_id}, for old {func_id}, entry {entry_point}");
                        new_functions_map.insert(*func_id, *new_id);
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
            for inner_call in inner_calls {
                let new_inner_call =
                    new_functions_map.get(&inner_call).copied().unwrap_or(inner_call);
                // println!("old inner_call {inner_call}, new {new_inner_call}");
                let function =
                    self.functions.get_mut(&new_inner_call).expect("ICE: Function does not exist");

                for block_id in function.reachable_blocks() {
                    #[allow(clippy::unnecessary_to_owned)] // clippy is wrong here
                    for instruction_id in function.dfg[block_id].instructions().to_vec() {
                        let instruction = function.dfg[instruction_id].clone();
                        let Instruction::Call { func: func_id, arguments } = instruction else {
                            continue;
                        };

                        let func_value = &function.dfg[func_id];
                        let Value::Function(func_id) = func_value else { continue };
                        // println!("call to {func_id} in {inner_call} from entry {entry_point}");
                        if let Some(new_id) = calls_to_update_w_entry.get(&(entry_point, *func_id))
                        {
                            // println!("inserting {new_id}, for old {func_id}, entry {entry_point}");
                            new_functions_map.insert(*func_id, *new_id);
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
        }

        self
    }
}
