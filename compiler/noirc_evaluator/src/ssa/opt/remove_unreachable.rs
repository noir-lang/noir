//! Removes any unreachable functions from the code. These can result from
//! optimizations making existing functions unreachable, e.g. `if false { foo() }`,
//! or even from monomorphizing an unconstrained version of a constrained function
//! where the original constrained version ends up never being used.
//!
//! This pass identifies all unreachable functions and prunes them from the
//! function set. Reachability is defined as:
//! - A function is reachable if it is an entry point (e.g., `main`)
//! - A function is reachable if it is called from another reachable function
//! - A function is reachable if it is stored in a reference (e.g., in a `Store` instruction) from another reachable function.
//!   Even if not immediately called, it may later be dynamically loaded and invoked.
//!   This marking is conservative but ensures correctness. We should instead rely on [mem2reg][crate::ssa::opt::mem2reg]
//!   for resolving loads/stores.
//!
//! The pass performs a recursive traversal starting from all entry points and marks
//! any transitively reachable functions. It then discards the rest.
//!
//! This pass helps shrink the SSA before compilation stages like inlining and dead code elimination.

use std::collections::BTreeSet;

use fxhash::FxHashSet as HashSet;

use crate::ssa::{
    ir::{
        function::{Function, FunctionId},
        instruction::Instruction,
        value::Value,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// See [`remove_unreachable`][self] module for more information.
    pub(crate) fn remove_unreachable_functions(mut self) -> Self {
        let mut reachable_functions = HashSet::default();

        // Go through all the functions, and if we have an entry point, extend the set of all
        // functions which are reachable.
        for (id, function) in self.functions.iter() {
            // XXX: `self.is_entry_point(*id)` could leave Brillig functions that nobody calls in the SSA.
            let is_entry_point = function.id() == self.main_id
                || function.runtime().is_acir() && function.runtime().is_entry_point();

            if is_entry_point {
                collect_reachable_functions(&self, *id, &mut reachable_functions);
            }
        }

        // Discard all functions not marked as reachable
        self.functions.retain(|id, _| reachable_functions.contains(id));
        self
    }
}

/// Recursively determine the reachable functions from a given function.
/// This function is only intended to be called on functions that are already known
/// to be entry points or transitively reachable from one.
///
/// # Arguments
/// - `ssa`: The full [Ssa] structure containing all functions.
/// - `current_func_id`: The [FunctionId] from which to begin a traversal.
/// - `reachable_functions`: A mutable set used to collect all reachable functions.
///   It serves both as the final output of this traversal and as a visited set
///   to prevent cycles and redundant recursion.
fn collect_reachable_functions(
    ssa: &Ssa,
    current_func_id: FunctionId,
    reachable_functions: &mut HashSet<FunctionId>,
) {
    // If this function has already been determine as reachable, then we have already
    // processed the given function and we can simply return.
    if reachable_functions.contains(&current_func_id) {
        return;
    }
    // Mark the given function as reachable
    reachable_functions.insert(current_func_id);

    // If the debugger is used, its possible for function inlining
    // to remove functions that the debugger still references
    let Some(func) = ssa.functions.get(&current_func_id) else {
        // TODO: when does this trigger??
        return;
    };

    // Get the set of reachable functions from the given function
    let used_functions = used_functions(func);

    // For each reachable function within the given function recursively collect
    // any more reachable functions.
    for called_func_id in used_functions.iter() {
        collect_reachable_functions(ssa, *called_func_id, reachable_functions);
    }
}

/// Identifies all reachable function IDs within a given function.
/// This includes:
/// - Function calls (functions used via `Call` instructions)
/// - Function references (functions stored via `Store` instructions)
///
/// # Arguments
/// - `func`: The [Function] to analyze for usage
///
/// # Returns
/// A sorted set of [`FunctionId`]s that are reachable from the function.
fn used_functions(func: &Function) -> BTreeSet<FunctionId> {
    let mut used_function_ids = BTreeSet::default();

    let mut find_functions = |value| {
        if let Value::Function(function) = func.dfg[value] {
            used_function_ids.insert(function);
        }
    };

    for block_id in func.reachable_blocks() {
        let block = &func.dfg[block_id];

        for instruction_id in block.instructions() {
            let instruction = &func.dfg[*instruction_id];

            if matches!(instruction, Instruction::Store { .. } | Instruction::Call { .. }) {
                instruction.for_each_value(&mut find_functions);
            }
        }

        block.unwrap_terminator().for_each_value(&mut find_functions);
    }

    used_function_ids
}

#[cfg(test)]
mod tests {
    use crate::assert_ssa_snapshot;

    use super::Ssa;

    #[test]
    fn remove_unused_brillig() {
        let src = "
          brillig(inline) fn main f0 {
            b0(v0: u32):
              v2 = call f1(v0) -> u32
              v4 = add v0, u32 1
              v5 = eq v2, v4
              constrain v2 == v4
              return
          }
          brillig(inline) fn increment f1 {
            b0(v0: u32):
              v2 = add v0, u32 1
              return v2
          }
          brillig(inline) fn increment_acir f2 {
            b0(v0: u32):
              v2 = add v0, u32 1
              return v2
          }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_functions();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v2 = call f1(v0) -> u32
            v4 = add v0, u32 1
            v5 = eq v2, v4
            constrain v2 == v4
            return
        }
        brillig(inline) fn increment f1 {
          b0(v0: u32):
            v2 = add v0, u32 1
            return v2
        }
        ");
    }
}
