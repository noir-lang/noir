use std::collections::BTreeMap;

use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    ir::{
        function::{Function, FunctionId},
        instruction::Instruction,
        value::Value,
    },
    Ssa,
};

use super::inlining::called_functions_vec;

impl Ssa {
    pub(crate) fn duplicate_reused_entry_points(mut self) -> Ssa {
        if self.main().runtime().is_brillig() {
            return self;
        }

        let brillig_entry_points = get_brillig_entry_points(&self.functions, self.main_id);
        let entry_points = brillig_entry_points.keys().copied().collect::<HashSet<_>>();

        let inner_call_to_entry_point = build_inner_call_to_entry_points(&brillig_entry_points);

        let functions_to_clone_map =
            build_functions_to_clone(&self.functions, inner_call_to_entry_point, entry_points);

        let calls_to_update = build_calls_to_update(&mut self, functions_to_clone_map);

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

fn build_functions_to_clone(
    functions: &BTreeMap<FunctionId, Function>,
    inner_call_to_entry_point: HashMap<FunctionId, Vec<FunctionId>>,
    entry_points: HashSet<FunctionId>,
) -> HashMap<FunctionId, Vec<(Function, FunctionId)>> {
    let mut functions_to_clone_map: HashMap<FunctionId, Vec<(Function, FunctionId)>> =
        HashMap::default();

    let mut add_function_to_clone = |entry_point: FunctionId, inner_call: FunctionId| {
        let cloned_function = Function::clone_no_id(&functions[&inner_call]);
        functions_to_clone_map.entry(entry_point).or_default().push((cloned_function, inner_call));
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

    functions_to_clone_map
}

// Clones new functions and returns a mapping representing the calls to update.
// Returns a map of (entry point, callee function) -> new callee function id.
fn build_calls_to_update(
    ssa: &mut Ssa,
    functions_to_clone_map: HashMap<FunctionId, Vec<(Function, FunctionId)>>,
) -> HashMap<(FunctionId, FunctionId), FunctionId> {
    let mut calls_to_update: HashMap<(FunctionId, FunctionId), FunctionId> = HashMap::default();

    for (entry_point, functions_to_clone) in functions_to_clone_map {
        for (mut cloned_function, old_id) in functions_to_clone {
            ssa.add_fn(|id| {
                calls_to_update.insert((entry_point, old_id), id);
                cloned_function.set_id(id);
                cloned_function
            });
        }
    }

    calls_to_update
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

/// Returns a map of Brillig entry points to all functions called in that entry point.
/// This includes any nested calls as well, as we want to be able to associate
/// any Brillig function with the appropriate global allocations.
pub(crate) fn get_brillig_entry_points(
    functions: &BTreeMap<FunctionId, Function>,
    main_id: FunctionId,
) -> HashMap<FunctionId, HashSet<FunctionId>> {
    let mut brillig_entry_points = HashMap::default();
    let acir_functions = functions.iter().filter(|(_, func)| func.runtime().is_acir());
    for (_, function) in acir_functions {
        for block_id in function.reachable_blocks() {
            for instruction_id in function.dfg[block_id].instructions() {
                let instruction = &function.dfg[*instruction_id];
                let Instruction::Call { func: func_id, arguments: _ } = instruction else {
                    continue;
                };

                let func_value = &function.dfg[*func_id];
                let Value::Function(func_id) = func_value else { continue };

                let called_function = &functions[func_id];
                if called_function.runtime().is_acir() {
                    continue;
                }

                // We have now found a Brillig entry point.
                brillig_entry_points.insert(*func_id, HashSet::default());
                build_entry_points_map_recursive(
                    functions,
                    *func_id,
                    called_function,
                    &mut brillig_entry_points,
                    im::HashSet::new(),
                );
            }
        }
    }

    // If main has been marked as Brillig, it is itself an entry point.
    // Run the same analysis from above on main.
    let main_func = &functions[&main_id];
    if main_func.runtime().is_brillig() {
        brillig_entry_points.insert(main_id, HashSet::default());
        build_entry_points_map_recursive(
            functions,
            main_id,
            &functions[&main_id],
            &mut brillig_entry_points,
            im::HashSet::new(),
        );
    }

    brillig_entry_points
}

/// Recursively mark any functions called in an entry point
fn build_entry_points_map_recursive(
    functions: &BTreeMap<FunctionId, Function>,
    entry_point: FunctionId,
    called_function: &Function,
    brillig_entry_points: &mut HashMap<FunctionId, HashSet<FunctionId>>,
    mut explored_functions: im::HashSet<FunctionId>,
) {
    if explored_functions.insert(called_function.id()).is_some() {
        return;
    }

    let inner_calls = called_functions_vec(called_function).into_iter().collect::<HashSet<_>>();

    for inner_call in inner_calls {
        if let Some(inner_calls) = brillig_entry_points.get_mut(&entry_point) {
            inner_calls.insert(inner_call);
        }

        build_entry_points_map_recursive(
            functions,
            entry_point,
            &functions[&inner_call],
            brillig_entry_points,
            explored_functions.clone(),
        );
    }
}

pub(crate) fn build_inner_call_to_entry_points(
    brillig_entry_points: &HashMap<FunctionId, HashSet<FunctionId>>,
) -> HashMap<FunctionId, Vec<FunctionId>> {
    // Map for fetching the correct entry point globals when compiling any function
    let mut inner_call_to_entry_point: HashMap<FunctionId, Vec<FunctionId>> = HashMap::default();

    // We only need to generate globals for entry points
    for (entry_point, entry_point_inner_calls) in brillig_entry_points.iter() {
        for inner_call in entry_point_inner_calls {
            inner_call_to_entry_point.entry(*inner_call).or_default().push(*entry_point);
        }
    }

    inner_call_to_entry_point
}
