//! Single-block load/store forwarding pass.
//!
//! This pass performs simple, fast optimizations on single-block functions:
//! - **Load forwarding**: If a load reads from an address whose value is already known
//!   (from a prior store), replace the load with the known value.
//! - **Dead store elimination**: If two stores write to the same address with no
//!   intervening load, the first store is dead and can be removed.
//!
//! This pass only runs on single-block functions. Multi-block functions are
//! handled by `mem2reg` which promotes variables to block parameters.
//! After inlining, unrolling, and flattening, ACIR functions are single-block.
//! Brillig functions with multiple blocks are skipped.
//!
//! ## Alias handling
//!
//! Alias reasoning uses allocation identity via [`may_alias`]:
//! - Two different `allocate` results never alias (each is a fresh, unique address).
//! - Different reference types never alias (`&mut Field` vs `&mut u32`).
//! - Any other pair of same-typed addresses may alias (conservative).
//!
//! This handles references extracted via `array_get` with dynamic indices:
//! the result is a new ValueId that could alias any same-typed allocation.
//! Constant-index `array_get` on reference arrays is simplified by the DFG
//! simplifier during re-insertion, resolving the alias.
//!
//! Calls are handled conservatively: simple reference arguments invalidate
//! that address and all its potential aliases; containers or nested
//! references clear all state.
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        function::Function,
        function_inserter::FunctionInserter,
        instruction::{Instruction, InstructionId},
        post_order::PostOrder,
        types::Type,
        value::ValueId,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn load_store_forwarding(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.load_store_forwarding();
        }
        self
    }
}

impl Function {
    pub(crate) fn load_store_forwarding(&mut self) {
        let mut inserter = FunctionInserter::new(self);
        let blocks = PostOrder::with_function(inserter.function).into_vec_reverse();

        // Only run on single-block functions. Multi-block functions rely on
        // mem2reg for cross-block promotion; Brillig natively supports loads/stores.
        if blocks.len() > 1 {
            return;
        }

        let allocations = collect_allocations(inserter.function, &blocks);

        let block = blocks[0];
        let instructions_to_remove =
            forward_loads_and_stores_in_block(&mut inserter, block, &allocations);

        if !instructions_to_remove.is_empty() {
            inserter.function.dfg[block]
                .instructions_mut()
                .retain(|id| !instructions_to_remove.contains(id));
        }

        // Re-insert instructions through the DFG simplify path. This resolves
        // value mappings from load forwarding AND triggers simplification
        // (e.g. `lt v2, u32 3` folds to a constant when v2 was forwarded).
        let instructions = inserter.function.dfg[block].take_instructions();
        for instruction_id in &instructions {
            inserter.push_instruction(*instruction_id, block, true);
        }
        inserter.map_terminator_in_place(block);
        inserter.map_data_bus_in_place();
    }
}

/// Collect all ValueIds produced by Allocate instructions.
fn collect_allocations(function: &Function, blocks: &[BasicBlockId]) -> HashSet<ValueId> {
    let mut allocations = HashSet::default();
    for block in blocks {
        for instruction_id in function.dfg[*block].instructions() {
            if let Instruction::Allocate = &function.dfg[*instruction_id] {
                let result = function.dfg.instruction_results(*instruction_id)[0];
                allocations.insert(result);
            }
        }
    }
    allocations
}

/// Returns true if two types are compatible for aliasing purposes.
///
/// `&mut T` and `&T` with the same inner type must be treated as potentially aliasing.
///
/// The comparison is recursive so that nested references (e.g. `&&mut T` vs
/// `&mut &T`) are also handled correctly.
fn same_type_for_aliasing(a: &Type, b: &Type) -> bool {
    match (a, b) {
        (Type::Reference(inner_a, _), Type::Reference(inner_b, _)) => {
            same_type_for_aliasing(inner_a, inner_b)
        }
        _ => a == b,
    }
}

/// Returns true if two addresses might refer to the same memory.
///
/// Conservative: returns true when uncertain.
/// - Same ValueId -> always alias.
/// - Incompatible reference types -> never alias.
/// - Both from different `allocate` instructions -> never alias.
/// - Otherwise -> may alias.
fn may_alias(a: ValueId, b: ValueId, allocations: &HashSet<ValueId>, dfg: &DataFlowGraph) -> bool {
    if a == b {
        return true;
    }
    if !same_type_for_aliasing(&dfg.type_of_value(a), &dfg.type_of_value(b)) {
        return false;
    }
    if allocations.contains(&a) && allocations.contains(&b) {
        return false;
    }
    true
}

/// Perform load/store forwarding within a single block.
///
/// Returns the set of instructions to remove from the block.
fn forward_loads_and_stores_in_block(
    inserter: &mut FunctionInserter,
    block: BasicBlockId,
    allocations: &HashSet<ValueId>,
) -> HashSet<InstructionId> {
    let mut known_values: HashMap<ValueId, ValueId> = HashMap::default();
    let mut last_stores: HashMap<ValueId, InstructionId> = HashMap::default();
    let mut instructions_to_remove: HashSet<InstructionId> = HashSet::default();

    let instructions = inserter.function.dfg[block].instructions().to_vec();

    for instruction_id in instructions {
        let instruction = &inserter.function.dfg[instruction_id];
        match instruction {
            Instruction::Store { address, value } => {
                let address = inserter.resolve(*address);
                let value = inserter.resolve(*value);
                let dfg = &inserter.function.dfg;

                // Dead store elimination: exact address match only.
                if let Some(prev_store) = last_stores.get(&address) {
                    instructions_to_remove.insert(*prev_store);
                }

                // Clear aliased entries (Y != address where may_alias).
                let aliases =
                    |k: &ValueId| *k != address && may_alias(address, *k, allocations, dfg);
                known_values.retain(|k, _| !aliases(k));
                last_stores.retain(|k, _| !aliases(k));

                known_values.insert(address, value);
                last_stores.insert(address, instruction_id);
            }
            Instruction::Load { address } => {
                let address = inserter.resolve(*address);

                let result = inserter.function.dfg.instruction_results(instruction_id)[0];
                if let Some(value) = known_values.get(&address) {
                    inserter.map_value(result, *value);
                    instructions_to_remove.insert(instruction_id);
                } else {
                    known_values.insert(address, result);
                }

                // Mark aliased stores as used (not dead).
                let dfg = &inserter.function.dfg;
                last_stores.retain(|k, _| !may_alias(address, *k, allocations, dfg));
            }
            Instruction::Call { .. } => {
                // Simple reference (`&mut T` where T has no refs): invalidate that
                // address and all its potential aliases.
                // Container or nested reference: clear all state.
                instruction.for_each_value(|value| {
                    let value = inserter.resolve(value);
                    let typ = inserter.function.dfg.type_of_value(value);
                    let is_simple_ref = matches!(typ.reference_element_type(), Some(inner) if !inner.contains_reference());
                    if is_simple_ref {
                        let dfg = &inserter.function.dfg;
                        known_values
                            .retain(|k, _| !may_alias(value, *k, allocations, dfg));
                        last_stores
                            .retain(|k, _| !may_alias(value, *k, allocations, dfg));
                    } else if typ.contains_reference() {
                        known_values.clear();
                        last_stores.clear();
                    }
                });
            }
            _ => {}
        }
    }

    instructions_to_remove
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::assert_ssa_does_not_change, ssa_gen::Ssa},
    };

    #[test]
    fn simple_store_then_load() {
        // A store followed by a load from the same address should forward the value
        // and remove both the load and the store (store becomes dead).
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 42 at v0
            v1 = load v0 -> Field
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 42 at v0
            return Field 42
        }
        ");
    }

    #[test]
    fn two_consecutive_stores_same_address() {
        // Two stores to the same address with no load in between — first store is dead.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            store Field 2 at v0
            v1 = load v0 -> Field
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 2 at v0
            return Field 2
        }
        ");
    }

    #[test]
    fn store_load_store() {
        // Store, load, then another store. The first store is NOT dead (it was loaded),
        // the load gets forwarded, and the second store survives.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            v1 = load v0 -> Field
            store Field 2 at v0
            v2 = load v0 -> Field
            v3 = add v1, v2
            return v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            store Field 2 at v0
            return Field 3
        }
        ");
    }

    #[test]
    fn call_with_reference_clears_known_value() {
        // A call that receives a reference should invalidate the known value.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 10 at v0
            call f1(v0)
            v1 = load v0 -> Field
            return v1
        }
        brillig(inline) fn f1 f1 {
          b0(v0: &mut Field):
            return
        }
        ";
        // The load should NOT be forwarded because the call could have modified the value.
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn multiple_addresses_tracked_independently() {
        // Two different allocations should be tracked independently.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            store Field 1 at v0
            store Field 2 at v1
            v2 = load v0 -> Field
            v3 = load v1 -> Field
            v4 = add v2, v3
            return v4
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            store Field 1 at v0
            store Field 2 at v1
            return Field 3
        }
        ");
    }

    #[test]
    fn works_on_brillig_functions() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 7 at v0
            v1 = load v0 -> Field
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 7 at v0
            return Field 7
        }
        ");
    }

    #[test]
    fn cross_block_not_forwarded() {
        // Load/store forwarding is per-block only. A store in b0 should not
        // forward to a load in b1.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 5 at v0
            jmp b1()
          b1():
            v1 = load v0 -> Field
            return v1
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn call_without_reference_does_not_clear() {
        // A call that does not pass a reference should not invalidate known values.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 10 at v0
            v1 = call f1(Field 0) -> Field
            v2 = load v0 -> Field
            v3 = add v1, v2
            return v3
        }
        brillig(inline) fn f1 f1 {
          b0(v0: Field):
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();

        // The load should be forwarded since the call doesn't take a reference
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 10 at v0
            v4 = call f1(Field 0) -> Field
            v5 = add v4, Field 10
            return v5
        }
        brillig(inline) fn f1 f1 {
          b0(v0: Field):
            return v0
        }
        ");
    }

    #[test]
    fn store_make_array_load_forwards() {
        // A reference stored to, then used in make_array, then loaded — should forward.
        // make_array doesn't modify pointed-to memory.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 42 at v0
            v1 = make_array [v0] : [&mut Field; 1]
            v2 = load v0 -> Field
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 42 at v0
            v2 = make_array [v0] : [&mut Field; 1]
            return Field 42
        }
        ");
    }

    #[test]
    fn store_through_alias_clears_known_values() {
        // If a reference is stored into an array, extracted via array_get as a new
        // ValueId, and then stored through, the original ref's known value must be
        // invalidated.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            v1 = make_array [v0] : [&mut Field; 1]
            v2 = array_get v1, index u32 0 -> &mut Field
            store Field 2 at v2
            v3 = load v0 -> Field
            return v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();

        // The store to v2 (alias of v0) clears v0's known value during forwarding,
        // so the load is NOT forwarded to stale Field 1. The array_get simplifies
        // to v0 during re-insertion, but the load correctly remains.
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            v2 = make_array [v0] : [&mut Field; 1]
            store Field 2 at v0
            v4 = load v0 -> Field
            return v4
        }
        ");
    }

    #[test]
    fn store_increment_rc_load_forwards() {
        // IncrementRc doesn't modify pointed-to memory — should forward.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut [Field; 2]
            v1 = make_array [Field 1, Field 2] : [Field; 2]
            store v1 at v0
            inc_rc v0
            v2 = load v0 -> [Field; 2]
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut [Field; 2]
            v3 = make_array [Field 1, Field 2] : [Field; 2]
            store v3 at v0
            return v3
        }
        ");
    }

    #[test]
    fn call_with_array_of_references_clears_known_values() {
        // A call that receives an array containing a reference should invalidate
        // known values. The callee can extract the reference via array_get and
        // modify the pointed-to memory. Regression test for regression_9398.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            v1 = make_array [v0] : [&mut Field; 1]
            call f1(v1)
            v2 = load v0 -> Field
            return v2
        }
        acir(inline) fn f1 f1 {
          b0(v0: [&mut Field; 1]):
            return
        }
        ";
        // The load should NOT be forwarded because the call receives an array
        // containing a reference, so the callee could modify the pointed-to memory.
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn multi_block_not_optimized() {
        // Multi-block functions are skipped — only single-block functions are optimized.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            jmp b2()
          b1():
            v3 = add v2, u32 1
            return v3
          b2():
            v0 = allocate -> &mut u32
            store u32 10 at v0
            v2 = load v0 -> u32
            jmp b1()
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn loop_not_optimized() {
        // Multi-block (loop) functions are skipped entirely.
        let src = "
        brillig(inline) fn main f0 {
          b0(v100: u1):
            jmp b1()
          b1():
            v0 = allocate -> &mut Field
            store Field 42 at v0
            v1 = load v0 -> Field
            jmpif v100 then: b1(), else: b2()
          b2():
            return v1
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn call_with_double_reference_clears_inner_known_value() {
        // Bug: When a `&mut &mut Field` is passed to a call, only the outer reference
        // is removed from known_values. The callee can load the inner reference and
        // store through it, but the inner ref's known value survives — causing a
        // subsequent load through the inner ref to forward a stale value.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 42 at v0
            v1 = allocate -> &mut &mut Field
            store v0 at v1
            call f1(v1)
            v2 = load v0 -> Field
            return v2
        }
        brillig(inline) fn f1 f1 {
          b0(v10: &mut &mut Field):
            v11 = load v10 -> &mut Field
            store Field 99 at v11
            return
        }
        ";
        // The callee writes 99 through the inner ref (v0). The load of v0 must NOT
        // be forwarded to the stale value 42.
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn call_returning_alias_of_local_allocation_prevents_forwarding() {
        // Bug: When a local allocation is passed to a call, it is removed from
        // known_values/last_stores but NOT from local_allocations. If the callee
        // returns an alias to the same memory, stores through the original address
        // skip the conservative clear (because it's still in local_allocations),
        // leaving stale entries for the alias.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            v1 = call f1(v0) -> &mut Field
            store Field 2 at v1
            store Field 3 at v0
            v2 = load v1 -> Field
            return v2
        }
        brillig(inline) fn f1 f1 {
          b0(v0: &mut Field):
            return v0
        }
        ";
        // v1 aliases v0. After `store 3 at v0`, loading v1 should see 3, not stale 2.
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn loop_carried_alias_prevents_incorrect_dead_store() {
        // Minimized from `test_programs/execution_success/loop_carried_aliases`.
        //
        // v2 holds a reference-to-reference; `store v3 at v2` inside the loop
        // creates a loop-carried alias: on the next iteration `load v2` returns
        // v3, so `load (load v2)` reads from v3 through the alias.
        //
        // Without loop-alias analysis, `store Field 0xdeadbeef at v3` looks like
        // a dead store (overwritten by `store v5 at v3`), and the load through
        // the alias incorrectly forwards a stale value.
        let src = "
        brillig(inline) fn bar f0 {
          b0(v0: &mut Field, v1: Field):
            v2 = allocate -> &mut &mut Field
            store v0 at v2
            v3 = allocate -> &mut Field
            store v1 at v3
            jmp b1()
          b1():
            store Field 3735928559 at v3
            v5 = load v2 -> &mut Field
            v6 = load v5 -> Field
            store v6 at v3
            store v3 at v2
            jmp b1()
        }
        ";
        // Verify the pass does not miscompile: the store of 0xdeadbeef at v3
        // must NOT be eliminated as a dead store, because `load v5` (where v5
        // is loaded from v2, which aliases v3 after `store v3 at v2`) reads
        // through the alias in the next iteration.
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn remove_redundant_loads_from_ref_params() {
        // Loads from reference parameters (not local allocations) should still be
        // forwarded load-to-load when no intervening store invalidates them.
        // After a store, subsequent loads should pick up the stored value.
        let src = "
        brillig(inline) impure fn push f0 {
          b0(v0: &mut [Field; 4], v1: &mut u32, v2: Field):
            v3 = load v0 -> [Field; 4]
            v4 = load v1 -> u32
            v5 = load v0 -> [Field; 4]
            v6 = load v1 -> u32
            v8 = lt v6, u32 4
            constrain v8 == u1 1
            v10 = array_set v3, index v6, value v2
            v12 = unchecked_add v6, u32 1
            store v10 at v0
            store v4 at v1
            v13 = load v0 -> [Field; 4]
            v14 = add v4, u32 1
            v15 = load v0 -> [Field; 4]
            store v15 at v0
            store v14 at v1
            return
        }
        brillig(inline) impure fn next_counter f1 {
          b0(v0: &mut [Field; 4], v1: &mut u32, v2: &mut Field):
            v3 = load v0 -> [Field; 4]
            v4 = load v1 -> u32
            v5 = load v2 -> Field
            v6 = load v0 -> [Field; 4]
            v7 = load v1 -> u32
            v8 = load v2 -> Field
            v10 = add v8, Field 1
            v11 = load v0 -> [Field; 4]
            v12 = load v1 -> u32
            v13 = load v2 -> Field
            store v11 at v0
            store v12 at v1
            store v10 at v2
            return v5
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();

        // In push: v5/v6 forward to v3/v4 (load-to-load before any stores).
        // v13 forwards to v10 (store-to-load: v0 kept through store to v1 since types differ),
        // v15 forwards to v10 (store-to-load), making store v4 at v1 a dead store.
        // In next_counter: v6/v7/v8 forward to v3/v4/v5 (load-to-load),
        // v11/v12/v13 forward to v3/v4/v5 (load-to-load, no intervening stores to v0/v1/v2
        // between the first loads and these).
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) impure fn push f0 {
          b0(v0: &mut [Field; 4], v1: &mut u32, v2: Field):
            v3 = load v0 -> [Field; 4]
            v4 = load v1 -> u32
            v6 = lt v4, u32 4
            constrain v6 == u1 1
            v8 = array_set v3, index v4, value v2
            v10 = unchecked_add v4, u32 1
            store v8 at v0
            v11 = add v4, u32 1
            store v8 at v0
            store v11 at v1
            return
        }
        brillig(inline) impure fn next_counter f1 {
          b0(v0: &mut [Field; 4], v1: &mut u32, v2: &mut Field):
            v3 = load v0 -> [Field; 4]
            v4 = load v1 -> u32
            v5 = load v2 -> Field
            v7 = add v5, Field 1
            store v3 at v0
            store v4 at v1
            store v7 at v2
            return v5
        }
        ");
    }

    #[test]
    fn load_to_load_does_not_bypass_alias_clear() {
        // Two reference params could alias. A load-to-load entry for v1 must not
        // prevent the store-to-v1 from clearing v0's known value.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: &mut Field, v1: &mut Field):
            v2 = load v0 -> Field
            store Field 5 at v0
            v3 = load v1 -> Field
            store Field 6 at v1
            v4 = load v0 -> Field
            return v4
        }
        ";
        // v0 and v1 could alias, so load v0 after store to v1 must NOT be forwarded.
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    // --- Regression tests for issues #12217-#12232 ---
    // Multi-block tests: the pass skips these entirely (single-block restriction).

    #[test]
    fn regression_12217_loop_alias_via_call_input() {
        // Loop-carried alias established via function call input. Multi-block -> skipped.
        let src = "
        brillig(inline) fn bar f0 {
          b0(v0: &mut Field, v1: Field):
            v2 = allocate -> &mut &mut Field
            store v0 at v2
            v3 = allocate -> &mut Field
            store v1 at v3
            jmp b1()
          b1():
            store Field 3735928559 at v3
            v5 = load v2 -> &mut Field
            v6 = load v5 -> Field
            store v6 at v3
            call f1(v3, v2)
            jmp b1()
        }
        brillig(inline) fn foo f1 {
          b0(v0: &mut Field, v1: &mut &mut Field):
            store v0 at v1
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn regression_12219_loop_alias_via_call_return() {
        // Loop-carried alias via returned reference from call. Multi-block -> skipped.
        let src = "
        brillig(inline) fn bar f0 {
          b0(v0: &mut Field, v1: Field):
            v2 = allocate -> &mut &mut Field
            store v0 at v2
            v3 = allocate -> &mut Field
            store v1 at v3
            jmp b1()
          b1():
            store Field 3735928559 at v3
            v5 = load v2 -> &mut Field
            v6 = load v5 -> Field
            store v6 at v3
            v7 = call f1(v3) -> &mut Field
            store v7 at v2
            jmp b1()
        }
        brillig(inline) fn f1 f1 {
          b0(v0: &mut Field):
            return v0
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn regression_12220_loop_alias_via_array_get() {
        // Loop-carried alias via array_get with variable index. Multi-block -> skipped.
        let src = "
        brillig(inline) fn bar f0 {
          b0(v0: &mut Field, v1: Field, v_idx: u32):
            v2 = allocate -> &mut &mut Field
            store v0 at v2
            v3 = allocate -> &mut Field
            store v1 at v3
            jmp b1()
          b1():
            store Field 3735928559 at v3
            v5 = load v2 -> &mut Field
            v6 = load v5 -> Field
            store v6 at v3
            v8 = make_array [v3, v0] : [&mut Field; 2]
            v9 = array_get v8, index v_idx -> &mut Field
            store v9 at v2
            jmp b1()
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn regression_12221_loop_alias_via_jmpif() {
        // Loop-carried alias via jmpif passing ref to non-header block. Multi-block -> skipped.
        let src = "
        brillig(inline) fn bar f0 {
          b0(v0: &mut Field, v1: Field, v_cond: u1):
            v2 = allocate -> &mut &mut Field
            store v0 at v2
            v3 = allocate -> &mut Field
            store v1 at v3
            jmp b1()
          b1():
            store Field 3735928559 at v3
            v5 = load v2 -> &mut Field
            v6 = load v5 -> Field
            store v6 at v3
            jmpif v_cond then: b2(v3), else: b3()
          b2(v7: &mut Field):
            store v7 at v2
            jmp b1()
          b3():
            jmp b1()
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn regression_12222_loop_nested_refs_form1() {
        // Array containing references stored in loop (Form 1 misses nested refs). Multi-block -> skipped.
        let src = "
        brillig(inline) fn bar f0 {
          b0(v0: &mut Field, v1: Field):
            v2 = allocate -> &mut [&mut Field; 2]
            v3 = allocate -> &mut Field
            store v1 at v3
            v4 = make_array [v0, v0] : [&mut Field; 2]
            store v4 at v2
            jmp b1()
          b1():
            store Field 3735928559 at v3
            v5 = load v2 -> [&mut Field; 2]
            v6 = array_get v5, index u32 0 -> &mut Field
            v7 = load v6 -> Field
            store v7 at v3
            v8 = make_array [v3, v0] : [&mut Field; 2]
            store v8 at v2
            jmp b1()
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn regression_12223_loop_nested_refs_form2() {
        // Loop header block param of array-of-refs type (Form 2 misses nested refs). Multi-block -> skipped.
        let src = "
        brillig(inline) fn bar f0 {
          b0(v0: &mut Field, v1: Field):
            v2 = allocate -> &mut Field
            store v1 at v2
            v3 = make_array [v0, v0] : [&mut Field; 2]
            jmp b1(v3)
          b1(v_arr: [&mut Field; 2]):
            store Field 3735928559 at v2
            v5 = array_get v_arr, index u32 0 -> &mut Field
            v6 = load v5 -> Field
            store v6 at v2
            v8 = make_array [v2, v0] : [&mut Field; 2]
            jmp b1(v8)
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn regression_12232_loop_load_ignores_carried_aliases() {
        // Loads at loop-carried alias addresses don't invalidate caches. Multi-block -> skipped.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: &mut Field):
            v1 = allocate -> &mut &mut Field
            store v0 at v1
            jmp b1()
          b1():
            store Field 1 at v0
            v3 = load v1 -> &mut Field
            store Field 2 at v0
            v4 = load v3 -> Field
            store v3 at v1
            jmp b1()
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    // --- Single-block regression tests: verify may_alias handles these correctly ---

    #[test]
    fn regression_12225_load_to_load_through_array_get_alias() {
        // array_get with dynamic index creates an alias. A store to the original
        // allocation must invalidate last_loads for the alias.
        let src = "
        brillig(inline) fn main f0 {
          b0(v_idx: u32):
            v1 = allocate -> &mut Field
            store Field 42 at v1
            v2 = allocate -> &mut Field
            store Field 7 at v2
            v3 = make_array [v1, v2] : [&mut Field; 2]
            v4 = array_get v3, index v_idx -> &mut Field
            v5 = load v4 -> Field
            store Field 99 at v1
            v6 = load v4 -> Field
            return v5, v6
        }
        ";
        // v4 may alias v1. After `store 99 at v1`, `load v4` must NOT forward
        // the stale load result from before the store.
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn regression_12230_load_does_not_mark_aliased_store_as_used() {
        // Two params could alias. Loading from v1 must mark the store to v0 as
        // used, preventing incorrect dead store elimination.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: &mut Field, v1: &mut Field):
            store Field 1 at v0
            v2 = load v1 -> Field
            store Field 2 at v0
            return v2
        }
        ";
        // If v0 == v1, eliminating `store Field 1 at v0` would be incorrect
        // because `load v1` reads it.
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn regression_12231_local_allocation_aliased_by_array_get() {
        // A local allocation can be aliased via array_get with a dynamic index.
        // Storing to the allocation must clear the alias's known value.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: &mut Field, v_idx: u32):
            v1 = allocate -> &mut Field
            v2 = make_array [v0, v1] : [&mut Field; 2]
            v3 = array_get v2, index v_idx -> &mut Field
            store Field 0 at v3
            store Field 1 at v1
            v4 = load v3 -> Field
            return v4
        }
        ";
        // v3 may alias v1. After `store 1 at v1`, `load v3` must NOT forward
        // the stale Field 0.
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn regression_12234_loop_alias_lost_through_if_else() {
        // v5 might alias v3, which is a loop alias, but IfElse does not
        // propagate the loop-alias property. The store to v2 before the
        // load must not be eliminated.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: &mut Field, v1: u1):
            v2 = allocate -> &mut Field
            store Field 0 at v2
            jmp b1(v0)
          b1(v3: &mut Field):
            v4 = not v1
            v5 = if v1 then v0 else (if v4) v3
            store Field 1 at v2
            v6 = load v5 -> Field
            store Field 2 at v2
            return v6
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn call_with_aliased_simple_ref_clears_aliases() {
        // A reference stored into an array via array_set, then extracted via
        // array_get as a new ValueId, then passed to a call. The callee can
        // write through that reference, so the original address's known value
        // must be invalidated. Regression test for #12317.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: &mut Field, v1: u32, v2: [&mut Field; 2]):
            store Field 0 at v0
            v3 = array_set v2, index v1, value v0
            v4 = array_get v3, index v1 -> &mut Field
            call f1(v4)
            v5 = load v0 -> Field
            return v5
        }

        brillig(inline) fn f1 f1 {
          b0(v0: &mut Field):
            store Field 1 at v0
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn does_not_remove_potentially_aliased_store_before_array_set() {
        // Regression test for #12316. After `array_set` stores v0 into v2, a
        // later `store at v1` may alias v0 (v1 could have been extracted from
        // v2). The intervening aliased store must invalidate last_stores[v0]
        // so that `store Field 2 at v0` does not treat `store Field 0 at v0`
        // as a redundant prior write and eliminate it.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: &mut Field, v1: &mut Field, v2: [&mut Field; 2]):
            store Field 0 at v0
            v3 = array_set v2, index u32 0, value v0
            store Field 1 at v1
            store Field 2 at v0
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn nested_mutable_and_immutable_reference_outer_are_aliases() {
        // same_type_for_aliasing must recurse through nested reference types.
        // v0 (&mut &mut Field) and v1 (&mut &Field) differ only in the inner
        // mutability flag; the outer types strip to the same underlying Field,
        // so they must be treated as potential aliases.
        //
        // Concretely: the store to v0 must invalidate the cached load of v1,
        // so the second load of v1 is NOT forwarded to v2.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: &mut &mut Field, v1: &mut &Field):
            v2 = load v1 -> &Field       // cache: last_loads[v1] = v2
            v3 = allocate -> &mut Field
            store v3 at v0              // must clear last_loads[v1] (v0 may alias v1)
            v4 = load v1 -> &Field      // must NOT forward to v2
            return v4
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn immutable_ref_load_prevents_dead_store_through_mutable_alias() {
        // A load from an immutable reference must protect a prior store to a
        // mutable reference of the same type, because the two may alias.
        // Without the fix, may_alias(&mut Field, &Field) returns false (type
        // mismatch), so the load does not mark the first store as used and it
        // gets incorrectly eliminated as a dead store.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: &mut Field, v1: &Field):
            store Field 1 at v0
            v2 = load v1 -> Field
            store Field 2 at v0
            return v2
        }
        ";
        // If v0 == v1, eliminating `store Field 1 at v0` is wrong because
        // `load v1` reads through it.
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn considers_mutable_reference_and_immutable_reference_to_be_aliases() {
        // v0 (&mut Field) and v1 (&Field) may point to the same memory even
        // though their types differ in mutability. The store to v0 must
        // therefore invalidate the cached load result for v1, so the second
        // load of v1 is NOT forwarded to v2.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: &mut Field, v1: &Field):
            v2 = load v1 -> Field    // cache: last_loads[v1] = v2
            store Field 0 at v0     // must clear last_loads[v1] (v0 may alias v1)
            v3 = load v1 -> Field   // must NOT forward to v2
            return v3
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }
}
