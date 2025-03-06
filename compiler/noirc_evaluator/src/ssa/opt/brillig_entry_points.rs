//! The purpose of this pass is to perform function specialization of Brillig functions based upon
//! a function's entry points. Function specialization is performed through duplication of functions.
//!
//! This pass is done due to how globals are initialized for Brillig generation.
//! We allow multiple Brillig entry points (every call to Brillig from ACIR is an entry point),
//! and in order to avoid re-initializing globals used in one entry point but not another,
//! we set the globals initialization code based upon the globals used in a given entry point.
//! The ultimate goal is to optimize for runtime execution.
//!
//! However, doing the above on its own is insufficient as we allow entry points to be called from
//! other entry points and functions can be called across multiple entry points.
//! As all functions can potentially share entry points and use globals, the global allocations maps
//! generated for different entry points can conflict.
//!
//! To provide a more concrete example, let's take this program:
//! ```noir
//! global ONE: Field = 1;
//! global TWO: Field = 2;
//! global THREE: Field = 3;
//! fn main(x: Field, y: pub Field) {
//!     /// Safety: testing context
//!     unsafe {
//!         entry_point_one(x, y);
//!         entry_point_two(x, y);
//!     }
//! }
//! unconstrained fn entry_point_one(x: Field, y: Field) {
//!     let z = ONE + x + y;
//!     assert(z == 2);
//!     inner_func(x, y);
//! }
//! unconstrained fn entry_point_two(x: Field, y: Field) {
//!     let z = TWO + x + y;
//!     assert(z == 3);
//!     inner_func(x, y);
//! }
//! unconstrained fn inner_func(x: Field, y: Field) {
//!     let z = THREE + x + y;
//!     assert(z == 4);
//! }
//! ```
//! The two entry points will have different global allocation maps:
//! ```noir
//! GlobalInit(Id(1)):
//!   CONST M32835 = 1
//!   CONST M32836 = 2
//!   CONST M32837 = 3
//!   RETURN
//! GlobalInit(Id(2)):
//!   CONST M32835 = 2
//!   CONST M32836 = 3
//!   RETURN
//! ```
//! It is then not clear when generating the bytecode for `inner_func` which global allocations map should be used,
//! and any choice will lead to an incorrect program.
//! If `inner_func` used the map for `entry_point_one` the bytecode generated would use `M32837` to represent `THREE`.
//! However, when `inner_func` is called from `entry_point_two`, the address for `THREE` is `M32836`.
//!
//! This pass will duplicate `inner_func` so that different functions are called by the different entry points.
//! The test module for this pass can be referenced to see how this function duplication looks in SSA.

use std::collections::{BTreeMap, BTreeSet};

use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    Ssa,
    ir::{
        function::{Function, FunctionId},
        instruction::Instruction,
        value::Value,
    },
};

use super::inlining::called_functions_vec;

impl Ssa {
    pub(crate) fn brillig_entry_point_analysis(mut self) -> Ssa {
        if self.main().runtime().is_brillig() {
            return self;
        }

        let brillig_entry_points = get_brillig_entry_points(&self.functions, self.main_id);

        let functions_to_clone_map = build_functions_to_clone(&brillig_entry_points);

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

/// For every call site, we can determine the entry point for a given callee.
/// Once we know that we can determine which functions are in need of duplication.
/// We duplicate when the following occurs:
/// 1. A function is called from two different entry points
/// 2. An entry point function is called from another entry point
fn build_functions_to_clone(
    brillig_entry_points: &BTreeMap<FunctionId, BTreeSet<FunctionId>>,
) -> HashMap<FunctionId, Vec<FunctionId>> {
    let inner_call_to_entry_point = build_inner_call_to_entry_points(brillig_entry_points);
    let entry_points = brillig_entry_points.keys().copied().collect::<HashSet<_>>();

    let mut functions_to_clone_map: HashMap<FunctionId, Vec<FunctionId>> = HashMap::default();

    for (inner_call, inner_call_entry_points) in inner_call_to_entry_point {
        let should_clone = inner_call_entry_points.len() > 1 || entry_points.contains(&inner_call);
        if should_clone {
            for entry_point in inner_call_entry_points {
                functions_to_clone_map.entry(entry_point).or_default().push(inner_call);
            }
        }
    }

    functions_to_clone_map
}

/// Clones new functions and returns a mapping representing the calls to update.
///
///  Returns a map of (entry point, callee function) -> new callee function id.
fn build_calls_to_update(
    ssa: &mut Ssa,
    functions_to_clone_map: HashMap<FunctionId, Vec<FunctionId>>,
) -> HashMap<(FunctionId, FunctionId), FunctionId> {
    let mut calls_to_update: HashMap<(FunctionId, FunctionId), FunctionId> = HashMap::default();

    for (entry_point, functions_to_clone) in functions_to_clone_map {
        for old_id in functions_to_clone {
            let function = ssa.functions[&old_id].clone();
            ssa.add_fn(|id| {
                calls_to_update.insert((entry_point, old_id), id);
                Function::clone_with_id(id, &function)
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
            let Some(new_id) = calls_to_update.get(&(entry_point, *func_id)) else {
                continue;
            };

            new_functions_map.insert(*func_id, *new_id);
            let new_function_value_id = function.dfg.import_function(*new_id);
            function.dfg[instruction_id] =
                Instruction::Call { func: new_function_value_id, arguments };
        }
    }
}

/// Returns a map of Brillig entry points to all functions called in that entry point.
/// This includes any nested calls as well, as we want to be able to associate
/// any Brillig function with the appropriate global allocations.
pub(crate) fn get_brillig_entry_points(
    functions: &BTreeMap<FunctionId, Function>,
    main_id: FunctionId,
) -> BTreeMap<FunctionId, BTreeSet<FunctionId>> {
    let mut brillig_entry_points = BTreeMap::default();
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
                brillig_entry_points.insert(*func_id, BTreeSet::default());
                build_entry_points_map_recursive(
                    functions,
                    *func_id,
                    *func_id,
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
        brillig_entry_points.insert(main_id, BTreeSet::default());
        build_entry_points_map_recursive(
            functions,
            main_id,
            main_id,
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
    called_function: FunctionId,
    brillig_entry_points: &mut BTreeMap<FunctionId, BTreeSet<FunctionId>>,
    mut explored_functions: im::HashSet<FunctionId>,
) {
    if explored_functions.insert(called_function).is_some() {
        return;
    }

    let inner_calls: HashSet<FunctionId> =
        called_functions_vec(&functions[&called_function]).into_iter().collect();

    for inner_call in inner_calls {
        if let Some(inner_calls) = brillig_entry_points.get_mut(&entry_point) {
            inner_calls.insert(inner_call);
        }

        build_entry_points_map_recursive(
            functions,
            entry_point,
            inner_call,
            brillig_entry_points,
            explored_functions.clone(),
        );
    }
}

/// Builds a mapping from a [`FunctionId`] to the set of [`FunctionId`s][`FunctionId`] of all the brillig entrypoints
/// from which this function is reachable.
pub(crate) fn build_inner_call_to_entry_points(
    brillig_entry_points: &BTreeMap<FunctionId, BTreeSet<FunctionId>>,
) -> HashMap<FunctionId, BTreeSet<FunctionId>> {
    // Map for fetching the correct entry point globals when compiling any function
    let mut inner_call_to_entry_point: HashMap<FunctionId, BTreeSet<FunctionId>> =
        HashMap::default();

    // We only need to generate globals for entry points
    for (entry_point, entry_point_inner_calls) in brillig_entry_points.iter() {
        for inner_call in entry_point_inner_calls {
            inner_call_to_entry_point.entry(*inner_call).or_default().insert(*entry_point);
        }
    }

    inner_call_to_entry_point
}

#[cfg(test)]
mod tests {
    use crate::ssa::opt::assert_normalized_ssa_equals;

    use super::Ssa;

    #[test]
    fn duplicate_inner_call_with_multiple_entry_points() {
        let src = "
        g0 = Field 1
        g1 = Field 2
        g2 = Field 3
        
        acir(inline) fn main f0 {
          b0(v3: Field, v4: Field):
            call f1(v3, v4)
            call f2(v3, v4)
            return
        }
        brillig(inline) fn entry_point_one f1 {
          b0(v3: Field, v4: Field):
            v5 = add g0, v3
            v6 = add v5, v4
            constrain v6 == Field 2
            call f3(v3, v4)
            return
        }
        brillig(inline) fn entry_point_two f2 {
          b0(v3: Field, v4: Field):
            v5 = add g1, v3
            v6 = add v5, v4
            constrain v6 == Field 3
            call f3(v3, v4)
            return
        }
        brillig(inline) fn inner_func f3 {
          b0(v3: Field, v4: Field):
            v5 = add g2, v3
            v6 = add v5, v4
            constrain v6 == Field 4
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.brillig_entry_point_analysis();
        let ssa = ssa.remove_unreachable_functions();

        // We expect `inner_func` to be duplicated
        let expected = "
        g0 = Field 1
        g1 = Field 2
        g2 = Field 3
        
        acir(inline) fn main f0 {
          b0(v3: Field, v4: Field):
            call f1(v3, v4)
            call f2(v3, v4)
            return
        }
        brillig(inline) fn entry_point_one f1 {
          b0(v3: Field, v4: Field):
            v5 = add Field 1, v3
            v6 = add v5, v4
            constrain v6 == Field 2
            call f3(v3, v4)
            return
        }
        brillig(inline) fn entry_point_two f2 {
          b0(v3: Field, v4: Field):
            v5 = add Field 2, v3
            v6 = add v5, v4
            constrain v6 == Field 3
            call f4(v3, v4)
            return
        }
        brillig(inline) fn inner_func f3 {
          b0(v3: Field, v4: Field):
            v5 = add Field 3, v3
            v6 = add v5, v4
            constrain v6 == Field 4
            return
        }
        brillig(inline) fn inner_func f4 {
          b0(v3: Field, v4: Field):
            v5 = add Field 3, v3
            v6 = add v5, v4
            constrain v6 == Field 4
            return
        }
        ";
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn duplicate_inner_call_with_multiple_entry_points_nested() {
        let src = "
        g0 = Field 2
        g1 = Field 3
        
        acir(inline) fn main f0 {
          b0(v2: Field, v3: Field):
            call f1(v2, v3)
            call f2(v2, v3)
            return
        }
        brillig(inline) fn entry_point_one f1 {
          b0(v2: Field, v3: Field):
            v4 = add g0, v2
            v5 = add v4, v3
            constrain v5 == Field 3
            call f3(v2, v3)
            return
        }
        brillig(inline) fn entry_point_two f2 {
          b0(v2: Field, v3: Field):
            v4 = add g0, v2
            v5 = add v4, v3
            constrain v5 == Field 3
            call f3(v2, v3)
            return
        }
        brillig(inline) fn inner_func f3 {
          b0(v2: Field, v3: Field):
            v4 = add g0, v2
            v5 = add v4, v3
            constrain v5 == Field 3
            call f4(v2, v3)
            return
        }
        brillig(inline) fn nested_inner_func f4 {
          b0(v2: Field, v3: Field):
            v4 = add g1, v2
            v5 = add v4, v3
            constrain v5 == Field 4
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.brillig_entry_point_analysis();
        let ssa = ssa.remove_unreachable_functions();

        // We expect both `inner_func` and `nested_inner_func` to be duplicated
        let expected = "
        g0 = Field 2
        g1 = Field 3
        
        acir(inline) fn main f0 {
          b0(v2: Field, v3: Field):
            call f1(v2, v3)
            call f2(v2, v3)
            return
        }
        brillig(inline) fn entry_point_one f1 {
          b0(v2: Field, v3: Field):
            v4 = add Field 2, v2
            v5 = add v4, v3
            constrain v5 == Field 3
            call f4(v2, v3)
            return
        }
        brillig(inline) fn entry_point_two f2 {
          b0(v2: Field, v3: Field):
            v4 = add Field 2, v2
            v5 = add v4, v3
            constrain v5 == Field 3
            call f6(v2, v3)
            return
        }
        brillig(inline) fn nested_inner_func f3 {
          b0(v2: Field, v3: Field):
            v4 = add Field 3, v2
            v5 = add v4, v3
            constrain v5 == Field 4
            return
        }
        brillig(inline) fn inner_func f4 {
          b0(v2: Field, v3: Field):
            v4 = add Field 2, v2
            v5 = add v4, v3
            constrain v5 == Field 3
            call f3(v2, v3)
            return
        }
        brillig(inline) fn nested_inner_func f5 {
          b0(v2: Field, v3: Field):
            v4 = add Field 3, v2
            v5 = add v4, v3
            constrain v5 == Field 4
            return
        }
        brillig(inline) fn inner_func f6 {
          b0(v2: Field, v3: Field):
            v4 = add Field 2, v2
            v5 = add v4, v3
            constrain v5 == Field 3
            call f5(v2, v3)
            return
        }
        ";
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn duplicate_entry_point_called_from_entry_points() {
        // Check that we duplicate entry points that are also called from another entry point.
        // In this test the entry points used in other entry points are f2 and f3.
        // These functions are also called within the wrapper function f4, as we also want to make sure
        // that we duplicate entry points called from another entry point's inner calls.
        let src = "
        g0 = Field 2
        g1 = Field 3
        g2 = Field 1
        
        acir(inline) fn main f0 {
          b0(v3: Field, v4: Field):
            call f1(v3, v4)
            call f2(v3, v4)
            call f3(v3, v4)
            return
        }
        brillig(inline) fn entry_point_inner_func_globals f1 {
          b0(v3: Field, v4: Field):
            call f4(v3, v4)
            return
        }
        brillig(inline) fn entry_point_one_global f2 {
          b0(v3: Field, v4: Field):
            v5 = add g0, v3
            v6 = add v5, v4
            constrain v6 == Field 3
            return
        }
        brillig(inline) fn entry_point_one_diff_global f3 {
          b0(v3: Field, v4: Field):
            v5 = add g1, v3
            v6 = add v5, v4
            constrain v6 == Field 4
            return
        }
        brillig(inline) fn wrapper f4 {
          b0(v3: Field, v4: Field):
            v5 = add g2, v3
            v6 = add v5, v4
            constrain v6 == Field 2
            call f2(v3, v4)
            call f3(v4, v3)
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.brillig_entry_point_analysis();

        // We expect `entry_point_one_global` and `entry_point_one_diff_global` to be duplicated
        let expected = "
        g0 = Field 2
        g1 = Field 3
        g2 = Field 1
        
        acir(inline) fn main f0 {
          b0(v3: Field, v4: Field):
            call f1(v3, v4)
            call f2(v3, v4)
            call f3(v3, v4)
            return
        }
        brillig(inline) fn entry_point_inner_func_globals f1 {
          b0(v3: Field, v4: Field):
            call f4(v3, v4)
            return
        }
        brillig(inline) fn entry_point_one_global f2 {
          b0(v3: Field, v4: Field):
            v5 = add Field 2, v3
            v6 = add v5, v4
            constrain v6 == Field 3
            return
        }
        brillig(inline) fn entry_point_one_diff_global f3 {
          b0(v3: Field, v4: Field):
            v5 = add Field 3, v3
            v6 = add v5, v4
            constrain v6 == Field 4
            return
        }
        brillig(inline) fn wrapper f4 {
          b0(v3: Field, v4: Field):
            v5 = add Field 1, v3
            v6 = add v5, v4
            constrain v6 == Field 2
            call f5(v3, v4)
            call f6(v4, v3)
            return
        }
        brillig(inline) fn entry_point_one_global f5 {
          b0(v3: Field, v4: Field):
            v5 = add Field 2, v3
            v6 = add v5, v4
            constrain v6 == Field 3
            return
        }
        brillig(inline) fn entry_point_one_diff_global f6 {
          b0(v3: Field, v4: Field):
            v5 = add Field 3, v3
            v6 = add v5, v4
            constrain v6 == Field 4
            return
        }
        ";
        assert_normalized_ssa_equals(ssa, expected);
    }
}
