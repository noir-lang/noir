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

use rustc_hash::FxHashSet as HashSet;

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
        let reachable_functions = reachable_functions(&self);

        // Discard all functions not marked as reachable
        self.functions.retain(|id, _| reachable_functions.contains(id));

        #[cfg(debug_assertions)]
        remove_unreachable_functions_post_check(&self);

        self
    }
}

/// Post-check condition for [Ssa::remove_unreachable_functions].
///
/// Succeeds if:
///   - `ssa` contains no unreachable functions, i.e., all functions are reachable from the entry points.
///
/// Otherwise panics.
///
/// ## Note
/// We reuse the same logic for checking reachability as in the main pass, so this function will not
/// catch any bugs in the pass itself, but it will ensure that the pass is idempotent.
#[cfg(debug_assertions)]
fn remove_unreachable_functions_post_check(ssa: &Ssa) {
    let reachable_functions = reachable_functions(ssa);

    let has_unreachable_functions =
        ssa.functions.keys().any(|id| !reachable_functions.contains(id));

    assert!(!has_unreachable_functions, "SSA contains unreachable functions");
}

/// Identifies all reachable function IDs within the provided [Ssa].
/// This includes:
/// - Function calls (functions used via `Call` instructions)
/// - Function references (functions stored via `Store` instructions)
///
/// # Arguments
/// - `ssa`: The [Ssa] to analyze for usage
///
/// # Returns
/// A sorted set of [`FunctionId`]s that are reachable from the entry points of the SSA.
fn reachable_functions(ssa: &Ssa) -> HashSet<FunctionId> {
    // Identify entry points
    let entry_points =
        ssa.functions.iter().filter_map(|(&id, _)| ssa.is_entry_point(id).then_some(id));

    // Build call graph dependencies using this passes definition of reachability.
    let dependencies = ssa.functions.iter().map(|(&id, func)| (id, used_functions(func))).collect();
    let call_graph = CallGraph::from_deps(dependencies);

    // Traverse the call graph from all entry points
    call_graph.reachable_from(entry_points)
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
                    | Instruction::ArraySet { .. }
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
    use crate::{assert_ssa_snapshot, ssa::opt::assert_ssa_does_not_change};

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
        assert_ssa_does_not_change(src, Ssa::remove_unreachable_functions);
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
        assert_ssa_does_not_change(src, Ssa::remove_unreachable_functions);
    }

    #[test]
    fn keep_functions_used_in_array_set() {
        // Regression test for issue "V-NSCA-VUL-003: Missing ArraySet case in Removing Unreachable Functions pass"
        // found in Veridise Audit. https://github.com/noir-lang/noir/issues/8890

        // f2 is written to an array using an `array_set` instruction. Thus, we do not want to remove it.
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v2 = make_array [f1] : [function; 1]
            v3 = array_set v2, index u32 0, value f2
            v4 = array_get v3, index u32 0 -> function
            v5 = call v4(v0) -> Field
            return
        }
        
        acir(inline) fn my_fun f1 {
          b0(v0: Field):
            v2 = add v0, Field 1
            return v2
        }
        
        acir(inline) fn my_fun2 f2 {
          b0(v0: Field):
            v2 = add v0, Field 2
            return v2
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::remove_unreachable_functions);
    }
}
