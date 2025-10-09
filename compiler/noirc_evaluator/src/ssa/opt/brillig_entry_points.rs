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

use rustc_hash::FxHashSet as HashSet;

use crate::ssa::{
    ir::{
        call_graph::CallGraph,
        function::{Function, FunctionId},
    },
};

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
