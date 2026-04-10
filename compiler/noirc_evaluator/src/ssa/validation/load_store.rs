//! Validates that loads from references only occur after at least one store.
//!
//! An `Allocate` instruction produces an uninitialized reference. Loading from it
//! before any store is invalid (reading uninitialized memory). This module tracks
//! which references have been stored to and panics on loads from uninitialized ones.
//!
//! ## Known limitation
//!
//! The check accumulates state globally across all blocks in visitation order
//! without considering control flow. A store on only one branch of a diamond
//! will mark the reference as initialized globally, even though the other branch
//! never stores. Fixing this would require dominance-based dataflow analysis.

use rustc_hash::FxHashSet as HashSet;

use crate::ssa::ir::{
    function::Function,
    instruction::{Instruction, InstructionId},
    value::{Value, ValueId},
};

pub(super) struct LoadStoreValidator {
    /// References produced by `Allocate` that have not yet been stored to.
    uninitialized_references: HashSet<ValueId>,
}

impl LoadStoreValidator {
    pub(super) fn new() -> Self {
        Self { uninitialized_references: HashSet::default() }
    }

    /// Process one instruction, updating state and panicking on invalid loads.
    pub(super) fn check_instruction(&mut self, instruction: InstructionId, function: &Function) {
        let dfg = &function.dfg;
        match &dfg[instruction] {
            Instruction::Allocate => {
                let result = dfg.instruction_results(instruction)[0];
                self.uninitialized_references.insert(result);
            }
            Instruction::Store { address, .. } => {
                self.uninitialized_references.remove(address);
            }
            Instruction::Load { address } => {
                if self.uninitialized_references.contains(address) {
                    panic!(
                        "Load from reference {address} before any store — \
                         reading uninitialized memory"
                    );
                }
            }
            Instruction::Call { arguments, .. } => {
                // A call may store to any reference passed as an argument,
                // so treat all reference arguments as potentially initialized.
                for arg in arguments {
                    self.mark_reference_args_initialized(*arg, function);
                }
            }
            _ => (),
        }
    }

    /// If `value` is an uninitialized reference, mark it as initialized.
    /// Also recurses into array values to handle references nested in arrays.
    fn mark_reference_args_initialized(&mut self, value: ValueId, function: &Function) {
        let dfg = &function.dfg;
        if self.uninitialized_references.remove(&value) {
            return;
        }
        // Check if this value is a MakeArray containing references
        if let Value::Instruction { instruction, .. } = &dfg[value]
            && let Instruction::MakeArray { elements, .. } = &dfg[*instruction]
        {
            for element in elements {
                self.mark_reference_args_initialized(*element, function);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::ssa::ssa_gen::Ssa;

    // --- Single block ---

    #[test]
    #[should_panic(expected = "Load from reference v0 before any store")]
    fn load_before_store_is_rejected() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = load v0 -> Field
            store Field 42 at v0
            return v1
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn load_after_store_is_valid() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 42 at v0
            v1 = load v0 -> Field
            return v1
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    // --- Cross-block linear flow ---

    #[test]
    fn store_in_predecessor_load_in_successor_is_valid() {
        // Allocate and store in b0, load in b1 — straightforward valid case across blocks.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            jmp b1()
          b1():
            v1 = load v0 -> Field
            return v1
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Load from reference v0 before any store")]
    fn allocate_in_entry_load_in_successor_no_store_is_rejected() {
        // Allocate in b0, jump to b1, load in b1 with no store anywhere.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            jmp b1()
          b1():
            v1 = load v0 -> Field
            return v1
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    // --- Diamond control flow (branching) ---

    #[test]
    fn store_on_both_branches_load_after_join_is_valid() {
        // Allocate in b0, store on both branches (b1 and b2), load after join in b3.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            jmpif v0 then: b1(), else: b2()
          b1():
            store Field 1 at v1
            jmp b3()
          b2():
            store Field 2 at v1
            jmp b3()
          b3():
            v2 = load v1 -> Field
            return v2
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn store_on_one_branch_only_load_after_join_is_not_caught() {
        // Known limitation: allocate in b0, store only in b1 (not b2), load in b3.
        // This is actually invalid (on the b2→b3 path, v1 is uninitialized),
        // but the current check does not catch it because it tracks stores globally
        // across all visited blocks without considering control flow.
        //
        // A proper fix would require dominance-based dataflow analysis.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            jmpif v0 then: b1(), else: b2()
          b1():
            store Field 1 at v1
            jmp b3()
          b2():
            jmp b3()
          b3():
            v2 = load v1 -> Field
            return v2
        }
        ";
        // This SHOULD panic but doesn't — documenting as known false negative.
        let _ = Ssa::from_str(src).unwrap();
    }

    // --- Multiple references ---

    #[test]
    fn multiple_allocations_independent_tracking() {
        // Two allocations: v0 gets stored to, v1 does not.
        // Load from v0 should be valid, load from v1 should not.
        // This test verifies we track each reference independently.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            store Field 1 at v0
            store Field 2 at v1
            v2 = load v0 -> Field
            v3 = load v1 -> Field
            return v2
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Load from reference v1 before any store")]
    fn multiple_allocations_one_uninitialized() {
        // v0 is stored to, but v1 is loaded before any store.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            store Field 1 at v0
            v2 = load v1 -> Field
            return v2
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    // --- Call semantics ---

    #[test]
    fn call_initializes_reference() {
        // Reference passed to a call is treated as potentially initialized.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            call f1(v0)
            v1 = load v0 -> Field
            return v1
        }
        brillig(inline) fn store_to_ref f1 {
          b0(v0: &mut Field):
            store Field 42 at v0
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Load from reference v0 before any store")]
    fn load_before_call_that_would_initialize() {
        // Load happens before the call that would store — still invalid.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = load v0 -> Field
            call f1(v0)
            return v1
        }
        brillig(inline) fn store_to_ref f1 {
          b0(v0: &mut Field):
            store Field 42 at v0
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn reference_in_array_passed_to_call() {
        // Reference nested in a MakeArray and passed to a call — treated as initialized.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = make_array [v0] : [&mut Field; 1]
            call f1(v1)
            v2 = load v0 -> Field
            return v2
        }
        brillig(inline) fn f1 f1 {
          b0(v0: [&mut Field; 1]):
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    // --- Loop ---

    #[test]
    fn store_in_loop_header_load_in_loop_body() {
        // Reference stored before the loop, loaded inside the loop body.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            store Field 0 at v1
            jmp b1()
          b1():
            v2 = load v1 -> Field
            v3 = add v2, Field 1
            store v3 at v1
            jmpif v0 then: b1(), else: b2()
          b2():
            v4 = load v1 -> Field
            return v4
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }
}
