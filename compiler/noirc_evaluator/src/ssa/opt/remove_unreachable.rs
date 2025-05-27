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
//! - A function is reachable if it is used in a block terminator (e.g., returned from a function)
//!
//! The pass builds a call graph based upon the definition of reachability above.
//! It then identifies all entry points and uses the [CallGraph::reachable_from] utility
//! to mark all transitively reachable functions. It then discards the rest.
//!
//! This pass helps shrink the SSA before compilation stages like inlining and dead code elimination.

use std::collections::BTreeSet;

use crate::ssa::{
    ir::{
        call_graph::CallGraph,
        function::{Function, FunctionId},
        instruction::Instruction,
        value::Value,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// See [`remove_unreachable`][self] module for more information.
    pub(crate) fn remove_unreachable_functions(mut self) -> Self {
        // Identify entry points
        let entry_points = self.functions.iter().filter_map(|(&id, func)| {
            // Not using `Ssa::is_entry_point` because it could leave Brillig functions that nobody calls in the SSA,
            // because it considers every Brillig function as an entry point.
            let is_entry_point =
                id == self.main_id || func.runtime().is_acir() && func.runtime().is_entry_point();
            is_entry_point.then_some(id)
        });

        // Build call graph dependencies using this passes definition of reachability.
        let dependencies =
            self.functions.iter().map(|(&id, func)| (id, used_functions(func))).collect();
        let call_graph = CallGraph::from_deps(dependencies);

        // Traverse the call graph from all entry points
        let reachable_functions = call_graph.reachable_from(entry_points);

        // Discard all functions not marked as reachable
        self.functions.retain(|id, _| reachable_functions.contains(id));
        self
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

            if matches!(
                instruction,
                Instruction::Store { .. }
                    | Instruction::Call { .. }
                    | Instruction::MakeArray { .. }
            ) {
                instruction.for_each_value(&mut find_functions);
            }
        }

        block.unwrap_terminator().for_each_value(&mut find_functions);
    }

    used_function_ids
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::opt::assert_normalized_ssa_equals};

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

    #[test]
    fn keep_stored_function() {
        // Initial SSA from the `function_ref` integration test.
        let src = r#"
        acir(inline) fn main f0 {
            b0(v0: u1):
              v1 = allocate -> &mut function
              store f1 at v1
              jmpif v0 then: b1, else: b2
            b1():
              store f2 at v1
              jmp b2()
            b2():
              v4 = load v1 -> function
              v5 = call v4() -> [u8; 3]
              return v5
          }
          acir(inline) fn foo f1 {
            b0():
              v2 = make_array b"foo"
              return v2
          }
          acir(inline) fn bar f2 {
            b0():
              v3 = make_array b"bar"
              return v3
          }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_functions();

        // It should not remove anything.
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn keep_functions_used_in_array() {
        // f1 and f2 are used within an array. Thus, we do not want to remove them.
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v5 = make_array [f1, f2] : [function; 2]
            v7 = lt v0, u32 4
            constrain v7 == u1 1, "Index out of bounds"
            v9 = array_get v5, index v0 -> function
            call v9()
            return
        }
        acir(inline) fn lambda f1 {
          b0():
            return
        }
        acir(inline) fn lambda f2 {
          b0():
            return
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_functions();

        assert_normalized_ssa_equals(ssa, src);
    }
}
