//! The purpose of this pass is to perform function specialization of Brillig functions based upon
//! a function's entry points. Function specialization is performed through duplication of functions.
//! Brillig entry points are defined as functions called directly by ACIR functions or are `main`.
//!
//! This pass is done due to how globals are initialized for Brillig generation.
//! We allow multiple Brillig entry points, and in order to avoid re-initializing globals
//! used in one entry point but not another, we set the globals initialization code based
//! upon the globals used in a given entry point. The ultimate goal is to optimize for runtime execution.
//!
//! However, doing the above on its own is insufficient as we allow entry points to be called from
//! other entry points and functions can be called across multiple entry points.
//! Without specialization, the following issues arise:
//! 1. Entry points calling the same function may conflict on global allocations.
//! 2. Entry points calling other entry points may cause overlapping global usage.
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
//! Here, `inner_func` is called by two different entry points. It is then not clear when generating the bytecode
//! for `inner_func` which global allocations map should be used, and any choice will lead to an incorrect program.
//! If `inner_func` used the map for `entry_point_one` the bytecode generated would use `M32837` to represent `THREE`.
//! However, when `inner_func` is called from `entry_point_two`, the address for `THREE` is `M32836`.
//!
//! This pass duplicates functions like `inner_func` so that each entry point gets its own specialized
//! version. The result is that bytecode can safely reference the correct globals without conflicts.
//!
//! The test module for this pass can be referenced to see how this function duplication looks in SSA.
//!
//! ## Post-conditions
//! - Each Brillig entry point has its own specialized set of functions. No non-entry Brillig
//!   function is reachable from more than one entry point.
//! - The single entry point restriction could be loosened if globals are not used at all or
//!   some Brillig functions do not use globals.
//!   However, Brillig generation attempts to hoist duplicated constants across functions
//!   to the global memory space so this restriction needs to be enforced.
use std::collections::{BTreeMap, BTreeSet};

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    Ssa,
    ir::{
        call_graph::CallGraph,
        function::{Function, FunctionId},
        instruction::{Instruction, InstructionId},
        value::{Value, ValueId},
    },
};

impl Ssa {
    pub(crate) fn brillig_entry_point_analysis(mut self) -> Ssa {
        let main = self.main();
        if main.runtime().is_brillig() {
            return self;
        }

        // Build a call graph from the SSA
        let call_graph = CallGraph::from_ssa(&self);

        // From the call graph find the Brillig entry points and set up
        // the functions needing specialization before performing the actual call site rewrites.
        let brillig_entry_points =
            get_brillig_entry_points_with_reachability(&self.functions, self.main_id, &call_graph);
        let functions_to_clone_map = build_functions_to_clone(&brillig_entry_points);
        let (calls_to_update, new_functions_map) =
            build_calls_to_update(&mut self, functions_to_clone_map, &brillig_entry_points);

        // Now we want to actually rewrite the appropriate call sites
        // First pass to rewrite the originally supplied call graph
        for CallToUpdate {
            entry_point,
            function_to_update,
            instruction,
            new_func_to_call: new_id,
            call_args: arguments,
        } in calls_to_update
        {
            // Fetch the caller function whose call site we wish to update
            let new_function_to_update = if entry_point == function_to_update {
                // Do not fetch entry points from the new functions map.
                // We leave resolving duplicated entry points to a later pass
                entry_point
            } else {
                new_functions_map
                    .get(&entry_point)
                    .and_then(|m| m.get(&function_to_update))
                    .copied()
                    .unwrap_or(function_to_update)
            };

            let function = self
                .functions
                .get_mut(&new_function_to_update)
                .expect("ICE: Function does not exist");
            let new_function_value_id = function.dfg.import_function(new_id);
            function.dfg[instruction] =
                Instruction::Call { func: new_function_value_id, arguments };
        }

        // Second pass to rewrite the calls sites in the cloned functions
        // The list of structs mapping call site updates in `calls_to_update` only includes the original function IDs
        // so we risk potentially not rewriting the call sites within the cloned functions themselves.
        // The cloned functions we are using are stored within the `new_functions_map`.
        for (_, new_functions_per_entry) in new_functions_map {
            for new_function in new_functions_per_entry.values() {
                let function =
                    self.functions.get_mut(new_function).expect("ICE: Function does not exist");
                resolve_cloned_function_call_sites(function, &new_functions_per_entry);
            }
        }

        #[cfg(debug_assertions)]
        brillig_specialization_post_check(&self);

        self
    }
}

/// Given that we have already rewritten all the call sites among the original SSA,
/// this function provides a helper for resolving the call sites within cloned functions.
/// This function will update a cloned function according to the supplied function mapping.
/// The function assumes that the supplied mapping is per entry point and handled
/// by the caller of this method.
fn resolve_cloned_function_call_sites(
    // Function that was cloned earlier in the pass to specialize functions
    // in the original SSA
    function: &mut Function,
    // Per entry point, maps (old function -> new function)
    new_functions_map: &HashMap<FunctionId, FunctionId>,
) {
    for block_id in function.reachable_blocks() {
        #[allow(clippy::unnecessary_to_owned)] // clippy is wrong here
        for instruction_id in function.dfg[block_id].instructions().to_vec() {
            let instruction = function.dfg[instruction_id].clone();
            let Instruction::Call { func: func_value_id, arguments } = instruction else {
                continue;
            };
            let func_value = &function.dfg[func_value_id];
            let Value::Function(func_id) = func_value else { continue };

            let Some(new_func_id) = new_functions_map.get(func_id) else {
                continue;
            };
            let new_function_value_id = function.dfg.import_function(*new_func_id);
            function.dfg[instruction_id] =
                Instruction::Call { func: new_function_value_id, arguments };
        }
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

// Per entry point context, maps the original function to its new specialized function
// (entry_point -> map(old_id, new_id))
type NewCallSitesMap = HashMap<FunctionId, HashMap<FunctionId, FunctionId>>;

/// Clones new functions and returns a mapping representing the calls to update.
///
/// Returns a set of [CallToUpdate] containing all information needed to rewrite
/// a call site and a [NewCallSitesMap]
fn build_calls_to_update(
    ssa: &mut Ssa,
    functions_to_clone_map: HashMap<FunctionId, Vec<FunctionId>>,
    brillig_entry_points: &BTreeMap<FunctionId, BTreeSet<FunctionId>>,
) -> (HashSet<CallToUpdate>, NewCallSitesMap) {
    // Clone new functions
    // Map of (entry point, callee function) -> new callee function id.
    // This will be used internally for determining whether a call site needs to be rewritten.
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

    // Maps a function to its new specialized function per entry point context
    // (entry_point -> map(old_id, new_id))
    let mut new_functions_map: NewCallSitesMap = HashMap::default();
    // Collect extra information about the call sites we want to rewrite.
    // We need to do this as the original calls to update were set up based upon
    // the original call sites, not the soon-to-be rewritten call sites.
    let mut new_calls_to_update = HashSet::default();
    for (entry_point, inner_calls) in brillig_entry_points {
        let function = ssa.functions.get(entry_point).expect("ICE: Function does not exist");
        new_calls_to_update.extend(collect_callsites_to_rewrite(
            function,
            *entry_point,
            &mut new_functions_map,
            &calls_to_update,
        ));
        for inner_call in inner_calls {
            let function = ssa.functions.get(inner_call).expect("ICE: Function does not exist");
            new_calls_to_update.extend(collect_callsites_to_rewrite(
                function,
                *entry_point,
                &mut new_functions_map,
                &calls_to_update,
            ));
        }
    }

    (new_calls_to_update, new_functions_map)
}

/// Stores the information necessary to appropriately update
/// the call sites across the Brillig entry point graph
///
/// This structure should be built by analyzing the unchanged SSA
/// and later used to perform updates.
#[derive(PartialEq, Eq, Hash)]
struct CallToUpdate {
    entry_point: FunctionId,
    function_to_update: FunctionId,
    instruction: InstructionId,
    new_func_to_call: FunctionId,
    call_args: Vec<ValueId>,
}

/// Go through the supplied function and based upon the call sites
/// set in the `calls_to_update` map build the set of call sites
/// that should be rewritten.
/// Upon finding call sites that should be rewritten this method will also
/// update the mapping of old functions to new functions in the supplied [NewCallSitesMap].
fn collect_callsites_to_rewrite(
    function: &Function,
    entry_point: FunctionId,
    // Maps (entry_point -> map(old_id, new_id))
    function_per_entry: &mut NewCallSitesMap,
    // Maps (entry_point, callee function) -> new callee function id
    calls_to_update: &HashMap<(FunctionId, FunctionId), FunctionId>,
) -> HashSet<CallToUpdate> {
    let mut new_calls_to_update = HashSet::default();
    for block_id in function.reachable_blocks() {
        #[allow(clippy::unnecessary_to_owned)] // clippy is wrong here
        for instruction_id in function.dfg[block_id].instructions().to_vec() {
            let instruction = function.dfg[instruction_id].clone();
            let Instruction::Call { func: func_value_id, arguments } = instruction else {
                continue;
            };

            let func_value = &function.dfg[func_value_id];
            let Value::Function(func_id) = func_value else { continue };
            let Some(new_id) = calls_to_update.get(&(entry_point, *func_id)) else {
                continue;
            };

            function_per_entry.entry(entry_point).or_default().insert(*func_id, *new_id);
            let new_call = CallToUpdate {
                entry_point,
                function_to_update: function.id(),
                instruction: instruction_id,
                new_func_to_call: *new_id,
                call_args: arguments,
            };
            new_calls_to_update.insert(new_call);
        }
    }
    new_calls_to_update
}

/// Returns the set of Brillig entry points
///
/// A Brillig entry point is defined as a Brillig function that is directly called
/// from at least one ACIR function, or is the `main` function itself if it is Brillig.
pub(crate) fn get_brillig_entry_points(
    functions: &BTreeMap<FunctionId, Function>,
    main_id: FunctionId,
    call_graph: &CallGraph,
) -> BTreeSet<FunctionId> {
    let mut entry_points = BTreeSet::new();

    // Only ACIR callers can introduce Brillig entry points
    let acir_callers = call_graph
        .callees()
        .into_iter()
        .filter(|(caller, _)| functions[caller].runtime().is_acir());

    for (_, callees) in acir_callers {
        // Filter only the Brillig callees. These are the Brillig entry points.
        entry_points
            .extend(callees.keys().filter(|callee| functions[callee].runtime().is_brillig()));
    }

    // If main has been marked as Brillig, it is itself an entry point.
    if functions[&main_id].runtime().is_brillig() {
        entry_points.insert(main_id);
    }

    entry_points
}

/// Returns a map of Brillig entry points to all reachable functions from that entry point.
///
/// A Brillig entry point is defined as a Brillig function that is directly called
/// from at least one ACIR function, or is the `main` function itself if it is Brillig.
///
/// The value set for each entry point includes all functions reachable
/// from the entry point (excluding the entry itself if it is non-recursive).
pub(crate) fn get_brillig_entry_points_with_reachability(
    functions: &BTreeMap<FunctionId, Function>,
    main_id: FunctionId,
    call_graph: &CallGraph,
) -> BTreeMap<FunctionId, BTreeSet<FunctionId>> {
    let recursive_functions = call_graph.get_recursive_functions();
    get_brillig_entry_points_with_recursive(functions, main_id, call_graph, &recursive_functions)
}

/// Like [get_brillig_entry_points_with_reachability], but uses a precomputed set of recursive functions
/// to avoid recomputing SCCs.
pub(crate) fn get_brillig_entry_points_with_recursive(
    functions: &BTreeMap<FunctionId, Function>,
    main_id: FunctionId,
    call_graph: &CallGraph,
    recursive_functions: &HashSet<FunctionId>,
) -> BTreeMap<FunctionId, BTreeSet<FunctionId>> {
    let mut brillig_entry_points = BTreeMap::default();

    for entry_point in get_brillig_entry_points(functions, main_id, call_graph) {
        brillig_entry_points
            .insert(entry_point, brillig_reachable(call_graph, recursive_functions, entry_point));
    }

    brillig_entry_points
}

/// Returns all functions reachable from the given Brillig entry point.
/// Includes the entry point itself if it is recursive, otherwise excludes it.
fn brillig_reachable(
    call_graph: &CallGraph,
    recursive_functions: &HashSet<FunctionId>,
    func: FunctionId,
) -> BTreeSet<FunctionId> {
    let mut reachable = call_graph.reachable_from([func]);
    if !recursive_functions.contains(&func) {
        reachable.remove(&func);
    }
    reachable.into_iter().collect()
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

/// Check post-execution properties of the Brillig specialization pass:
/// * No Brillig function should be reachable from more than one entry point
///   (to prevent global allocation conflicts).
#[cfg(debug_assertions)]
fn brillig_specialization_post_check(ssa: &Ssa) {
    let call_graph = CallGraph::from_ssa(ssa);
    let brillig_entry_points =
        get_brillig_entry_points_with_reachability(&ssa.functions, ssa.main_id, &call_graph);
    let inner_call_to_entry_point = build_inner_call_to_entry_points(&brillig_entry_points);

    for (func_id, entry_points) in inner_call_to_entry_point {
        if entry_points.len() > 1 {
            panic!(
                "Brillig specialization invariant violated: \
                 function {func_id} is reachable from multiple entry points: {entry_points:?}"
            );
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::assert_ssa_snapshot;

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
        assert_ssa_snapshot!(ssa, @r"
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
            call f4(v3, v4)
            return
        }
        brillig(inline) fn entry_point_two f2 {
          b0(v3: Field, v4: Field):
            v5 = add Field 2, v3
            v6 = add v5, v4
            constrain v6 == Field 3
            call f3(v3, v4)
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
        ");
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
        assert_ssa_snapshot!(ssa, @r"
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
            call f5(v2, v3)
            return
        }
        brillig(inline) fn entry_point_two f2 {
          b0(v2: Field, v3: Field):
            v4 = add Field 2, v2
            v5 = add v4, v3
            constrain v5 == Field 3
            call f3(v2, v3)
            return
        }
        brillig(inline) fn inner_func f3 {
          b0(v2: Field, v3: Field):
            v4 = add Field 2, v2
            v5 = add v4, v3
            constrain v5 == Field 3
            call f4(v2, v3)
            return
        }
        brillig(inline) fn nested_inner_func f4 {
          b0(v2: Field, v3: Field):
            v4 = add Field 3, v2
            v5 = add v4, v3
            constrain v5 == Field 4
            return
        }
        brillig(inline) fn inner_func f5 {
          b0(v2: Field, v3: Field):
            v4 = add Field 2, v2
            v5 = add v4, v3
            constrain v5 == Field 3
            call f6(v2, v3)
            return
        }
        brillig(inline) fn nested_inner_func f6 {
          b0(v2: Field, v3: Field):
            v4 = add Field 3, v2
            v5 = add v4, v3
            constrain v5 == Field 4
            return
        }
        ");
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
        assert_ssa_snapshot!(ssa, @r"
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
            call f6(v3, v4)
            call f5(v4, v3)
            return
        }
        brillig(inline) fn entry_point_one_diff_global f5 {
          b0(v3: Field, v4: Field):
            v5 = add Field 3, v3
            v6 = add v5, v4
            constrain v6 == Field 4
            return
        }
        brillig(inline) fn entry_point_one_global f6 {
          b0(v3: Field, v4: Field):
            v5 = add Field 2, v3
            v6 = add v5, v4
            constrain v6 == Field 3
            return
        }
        ");
    }

    #[test]
    fn duplicate_recursive_shared_entry_points() {
        // Check that we appropriately specialize functions when the entry point
        // is recursive.
        // f1 and f2 in the SSA below are recursive with themselves and another entry point.
        let src = "
        acir(inline) impure fn main f0 {
          b0():
            v3 = call f1(u1 1, u32 5) -> u1
            constrain v3 == u1 0
            v6 = call f2(u1 1, u32 5) -> u1
            constrain v6 == u1 0
            return
        }
        brillig(inline) impure fn func_1 f1 {
          b0(v0: u1, v1: u32):
            v4 = eq v1, u32 0
            jmpif v4 then: b1, else: b2
          b1():
            jmp b3(u1 0)
          b2():
            v6 = sub v1, u32 1
            v8 = call f2(v0, v6) -> u1
            v10 = call f1(v8, v6) -> u1
            jmp b3(v10)
          b3(v2: u1):
            return v2
        }
        brillig(inline) impure fn func_2 f2 {
          b0(v0: u1, v1: u32):
            v4 = eq v1, u32 0
            jmpif v4 then: b1, else: b2
          b1():
            jmp b3(u1 0)
          b2():
            v6 = sub v1, u32 1
            v8 = call f2(v0, v6) -> u1
            v10 = call f1(v8, v6) -> u1
            jmp b3(v10)
          b3(v2: u1):
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.brillig_entry_point_analysis();

        // We want no shared callees between entry points.
        // Each Brillig entry point (f1 and f2 called from f0) should have its own
        // specialized function call graph.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) impure fn main f0 {
          b0():
            v3 = call f1(u1 1, u32 5) -> u1
            constrain v3 == u1 0
            v6 = call f2(u1 1, u32 5) -> u1
            constrain v6 == u1 0
            return
        }
        brillig(inline) impure fn func_1 f1 {
          b0(v0: u1, v1: u32):
            v4 = eq v1, u32 0
            jmpif v4 then: b1, else: b2
          b1():
            jmp b3(u1 0)
          b2():
            v6 = sub v1, u32 1
            v8 = call f5(v0, v6) -> u1
            v10 = call f6(v8, v6) -> u1
            jmp b3(v10)
          b3(v2: u1):
            return v2
        }
        brillig(inline) impure fn func_2 f2 {
          b0(v0: u1, v1: u32):
            v4 = eq v1, u32 0
            jmpif v4 then: b1, else: b2
          b1():
            jmp b3(u1 0)
          b2():
            v6 = sub v1, u32 1
            v8 = call f3(v0, v6) -> u1
            v10 = call f4(v8, v6) -> u1
            jmp b3(v10)
          b3(v2: u1):
            return v2
        }
        brillig(inline) fn func_2 f3 {
          b0(v0: u1, v1: u32):
            v4 = eq v1, u32 0
            jmpif v4 then: b1, else: b2
          b1():
            jmp b3(u1 0)
          b2():
            v6 = sub v1, u32 1
            v8 = call f3(v0, v6) -> u1
            v10 = call f4(v8, v6) -> u1
            jmp b3(v10)
          b3(v2: u1):
            return v2
        }
        brillig(inline) fn func_1 f4 {
          b0(v0: u1, v1: u32):
            v4 = eq v1, u32 0
            jmpif v4 then: b1, else: b2
          b1():
            jmp b3(u1 0)
          b2():
            v6 = sub v1, u32 1
            v8 = call f3(v0, v6) -> u1
            v10 = call f4(v8, v6) -> u1
            jmp b3(v10)
          b3(v2: u1):
            return v2
        }
        brillig(inline) fn func_2 f5 {
          b0(v0: u1, v1: u32):
            v4 = eq v1, u32 0
            jmpif v4 then: b1, else: b2
          b1():
            jmp b3(u1 0)
          b2():
            v6 = sub v1, u32 1
            v8 = call f5(v0, v6) -> u1
            v10 = call f6(v8, v6) -> u1
            jmp b3(v10)
          b3(v2: u1):
            return v2
        }
        brillig(inline) fn func_1 f6 {
          b0(v0: u1, v1: u32):
            v4 = eq v1, u32 0
            jmpif v4 then: b1, else: b2
          b1():
            jmp b3(u1 0)
          b2():
            v6 = sub v1, u32 1
            v8 = call f5(v0, v6) -> u1
            v10 = call f6(v8, v6) -> u1
            jmp b3(v10)
          b3(v2: u1):
            return v2
        }
        ");
    }

    #[test]
    fn duplicate_recursive_shared_entry_points_indirect_recursion() {
        // This test is essentially identical to `duplicate_recursive_shared_entry_points`
        // except that the one recursive entry point does not recurse on itself directly.
        // f1 is recursive, but only through calling itself in f2.
        let src = "
        acir(inline) impure fn main f0 {
          b0():
            call f1(Field 1)
            call f2(Field 1)
            return
        }
        brillig(inline) impure fn foo f1 {
          b0(v0: Field):
            call f2(v0)
            return
        }
        brillig(inline) impure fn bar f2 {
          b0(v0: Field):
            call f1(Field 1)
            call f2(Field 1)
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.brillig_entry_point_analysis();

        // We want no shared callees between entry points.
        // Each Brillig entry point (f1 and f2 called from f0) should have its own
        // specialized function call graph.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) impure fn main f0 {
          b0():
            call f1(Field 1)
            call f2(Field 1)
            return
        }
        brillig(inline) impure fn foo f1 {
          b0(v0: Field):
            call f5(v0)
            return
        }
        brillig(inline) impure fn bar f2 {
          b0(v0: Field):
            call f4(Field 1)
            call f3(Field 1)
            return
        }
        brillig(inline) fn bar f3 {
          b0(v0: Field):
            call f4(Field 1)
            call f3(Field 1)
            return
        }
        brillig(inline) fn foo f4 {
          b0(v0: Field):
            call f3(v0)
            return
        }
        brillig(inline) fn bar f5 {
          b0(v0: Field):
            call f6(Field 1)
            call f5(Field 1)
            return
        }
        brillig(inline) fn foo f6 {
          b0(v0: Field):
            call f5(v0)
            return
        }
        ");
    }

    #[test]
    fn functions_reachable_from_single_entry_point_are_not_duplicated() {
        let src = "
        g0 = Field 1

        acir(inline) fn main f0 {
          b0(v1: Field):
            call f1(v1)
            return
        }
        brillig(inline) fn entry_point f1 {
          b0(v1: Field):
            call f2(v1)
            return
        }
        brillig(inline) fn helper_func f2 {
          b0(v1: Field):
            call f3(v1)
            return
        }
        brillig(inline) fn leaf_func f3 {
          b0(v1: Field):
            v2 = add g0, v1
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.brillig_entry_point_analysis();

        // f2 and f3 are reachable from only one entry point, so they are not duplicated
        assert_ssa_snapshot!(ssa, @r#"
        g0 = Field 1

        acir(inline) fn main f0 {
          b0(v1: Field):
            call f1(v1)
            return
        }
        brillig(inline) fn entry_point f1 {
          b0(v1: Field):
            call f2(v1)
            return
        }
        brillig(inline) fn helper_func f2 {
          b0(v1: Field):
            call f3(v1)
            return
        }
        brillig(inline) fn leaf_func f3 {
          b0(v1: Field):
            v2 = add Field 1, v1
            return
        }
        "#);
    }

    #[test]
    fn idempotency() {
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
        let mut first_ssa = ssa.brillig_entry_point_analysis().remove_unreachable_functions();

        // We expect `inner_func` to be duplicated
        assert_ssa_snapshot!(&mut first_ssa, @r"
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
            call f4(v3, v4)
            return
        }
        brillig(inline) fn entry_point_two f2 {
          b0(v3: Field, v4: Field):
            v5 = add Field 2, v3
            v6 = add v5, v4
            constrain v6 == Field 3
            call f3(v3, v4)
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
        ");

        let mut second_ssa =
            first_ssa.brillig_entry_point_analysis().remove_unreachable_functions();

        // We expect `inner_func` to be duplicated
        assert_ssa_snapshot!(&mut second_ssa, @r"
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
            call f4(v3, v4)
            return
        }
        brillig(inline) fn entry_point_two f2 {
          b0(v3: Field, v4: Field):
            v5 = add Field 2, v3
            v6 = add v5, v4
            constrain v6 == Field 3
            call f3(v3, v4)
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
        ");
    }
}
