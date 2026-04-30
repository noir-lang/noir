//! Load/store forwarding pass, driven by the whole-program alias analysis.
//!
//! - **Load forwarding**: If a load reads from an address whose value is already known
//!   (from a prior store), replace the load with the known value.
//! - **Dead store elimination**: If two stores write to the same address with no
//!   intervening load, the first store is dead and can be removed.
//!
//! The pass is flow-sensitive *within* each block and starts each block with
//! empty state. Cross-block promotion is mem2reg's job; this pass only exploits
//! stores+loads that appear in the same basic block. Running it on multi-block
//! functions is sound because empty-state-at-block-entry never forwards across
//! block boundaries.
//!
//! ## Alias handling
//!
//! Alias queries go through the [`AliasAnalysis`] computed over the whole SSA.
//!
//! Calls are handled conservatively: simple reference arguments invalidate
//! that address and all its potential aliases; containers or nested
//! references clear all state.
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        function::Function,
        function_inserter::FunctionInserter,
        instruction::{Instruction, InstructionId},
        post_order::PostOrder,
        value::ValueId,
    },
    opt::alias_analysis::{AliasAnalysis, GlobalValueId},
    ssa_gen::Ssa,
};

impl Ssa {
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn load_store_forwarding(mut self) -> Ssa {
        let mut analysis = AliasAnalysis::analyze(&self);
        for function in self.functions.values_mut() {
            function.load_store_forwarding(&mut analysis);
        }
        self
    }
}

impl Function {
    pub(crate) fn load_store_forwarding(&mut self, analysis: &mut AliasAnalysis) {
        let mut inserter = FunctionInserter::new(self);
        let blocks = PostOrder::with_function(inserter.function).into_vec_reverse();

        for block in blocks {
            let instructions_to_remove =
                forward_loads_and_stores_in_block(&mut inserter, block, analysis);

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
        }
        inserter.map_data_bus_in_place();
    }
}

/// Perform load/store forwarding within a single block. State starts empty
/// at block entry and never crosses block boundaries — sound in the presence
/// of back-edges and joins, just imprecise across blocks.
///
/// Returns the set of instructions to remove from the block.
fn forward_loads_and_stores_in_block(
    inserter: &mut FunctionInserter,
    block: BasicBlockId,
    analysis: &mut AliasAnalysis,
) -> HashSet<InstructionId> {
    let mut known_values: HashMap<GlobalValueId, (GlobalValueId, ValueId)> = HashMap::default();
    let mut last_stores: HashMap<GlobalValueId, (GlobalValueId, InstructionId)> =
        HashMap::default();
    let mut instructions_to_remove: HashSet<InstructionId> = HashSet::default();

    let instructions = inserter.function.dfg[block].instructions().to_vec();

    for instruction_id in instructions {
        let instruction = &inserter.function.dfg[instruction_id];
        match instruction {
            Instruction::Store { address, value } => {
                let address = inserter.resolve(*address);
                let address = GlobalValueId::new(inserter.function, address);
                let value = inserter.resolve(*value);
                let key = analysis.get_trusted_allocation_site(address).unwrap_or(address);

                // Dead store elimination: a prior store under the same canonical key must-aliases this address
                // Kill any prior store at an address that must-alias the new one.
                if let Some((_, prev_store)) = last_stores.get(&key) {
                    instructions_to_remove.insert(*prev_store);
                }

                // Clear entries that may-alias the address.
                // We use the original address `a` (the first field of the map values) because of a potential
                // precision loss if the key `_k` happens to be on another function (see the comment inside `may_alias`)
                let function: &Function = inserter.function;
                known_values.retain(|_k, (a, _)| !analysis.may_alias(function, address, *a));
                last_stores.retain(|_k, (a, _)| !analysis.may_alias(function, address, *a));

                known_values.insert(key, (address, value));
                last_stores.insert(key, (address, instruction_id));
            }
            Instruction::Load { address } => {
                let address = inserter.resolve(*address);
                let address = GlobalValueId::new(inserter.function, address);
                let key = analysis.get_trusted_allocation_site(address).unwrap_or(address);
                let result = inserter.function.dfg.instruction_results(instruction_id)[0];
                let forward = known_values.get(&key).copied();

                if let Some((_, value)) = forward {
                    inserter.map_value(result, value);
                    instructions_to_remove.insert(instruction_id);
                } else {
                    known_values.insert(key, (address, result));
                }

                // Mark aliased stores as used (not dead).
                let function: &Function = inserter.function;
                last_stores.retain(|_k, (a, _)| !analysis.may_alias(function, address, *a));
            }
            Instruction::Call { .. } => {
                // If the call arguments can reference a known value, we invalidate it.
                let mut call_values: Vec<ValueId> = Vec::new();
                instruction.for_each_value(|v| call_values.push(v));
                for value in call_values {
                    let value = inserter.resolve(value);
                    if !inserter.function.dfg.type_of_value(value).contains_reference() {
                        continue;
                    }
                    let value = GlobalValueId::new(inserter.function, value);
                    // check against `a` for consistency with other checks, but here it does not matter.
                    known_values.retain(|_k, (a, _)| !analysis.may_reference(value, *a));
                    last_stores.retain(|_k, (a, _)| !analysis.may_reference(value, *a));
                }
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

        // The store to v2 (alias of v0) does not let stale `Field 1` be
        // forwarded. Pass-2 site propagation sets v2's allocation site to v0
        // (the array's pointee class has the singleton site `v0`), so
        // `must_alias(v0, v2)` fires: the first store is dead, the second
        // store updates the must-aliased entry, and the load forwards the
        // current value `Field 2`.
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = make_array [v0] : [&mut Field; 1]
            store Field 2 at v0
            return Field 2
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
    fn multi_block_within_block_forwarding() {
        // Within-block forwarding works on multi-block functions too — each
        // block is processed with fresh state so cross-block flow can't
        // introduce unsoundness. Here, `v2 = load v0` is forwarded to the
        // constant u32 10, which then folds the `add` and the `return`.
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
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            jmp b2()
          b1():
            return u32 11
          b2():
            v0 = allocate -> &mut u32
            store u32 10 at v0
            jmp b1()
        }
        ");
    }

    #[test]
    fn loop_within_block_forwarding() {
        // A local store+load inside a loop block still folds within that
        // block. The load forwards to Field 42 every iteration, and the
        // successor's `return v1` becomes `return Field 42`.
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
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u1):
            jmp b1()
          b1():
            v1 = allocate -> &mut Field
            store Field 42 at v1
            jmpif v0 then: b1(), else: b2()
          b2():
            return Field 42
        }
        ");
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
        // In each iteration of b1, `store Field 1 at v0` is overwritten by
        // `store Field 2 at v0` before any load of v0 observes it (the only
        // intervening load reads *v1, not *v0).
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
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: &mut Field):
            v1 = allocate -> &mut &mut Field
            store v0 at v1
            jmp b1()
          b1():
            v2 = load v1 -> &mut Field
            store Field 2 at v0
            v4 = load v2 -> Field
            store v2 at v1
            jmp b1()
        }
        ");
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
    fn regression_12234_if_else_over_params_does_not_alias_local_allocate() {
        // v5 is either v0 (function param) or v3 (block param fed by v0) —
        // both coming from the external caller. v2 is a local allocate, so
        // v5 cannot alias v2 at runtime (v2's address is fresh). The new
        // alias analysis recognizes this, so `store Field 1 at v2` is safely
        // DSE'd by the subsequent `store Field 2 at v2`.
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
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: &mut Field, v1: u1):
            v3 = allocate -> &mut Field
            store Field 0 at v3
            jmp b1(v0)
          b1(v2: &mut Field):
            v5 = not v1
            v6 = if v1 then v0 else (if v5) v2
            v7 = load v6 -> Field
            store Field 2 at v3
            return v7
        }
        ");
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

    #[test]
    fn call_with_inner_arg_does_not_invalidate_outer_known_value() {
        // The outer ref's cache (cached the inner ref it stores) must SURVIVE
        // a call that takes only the inner ref. The callee writes through the
        // inner — which changes Field memory, not the outer's stored ref.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 42 at v0
            v1 = allocate -> &mut &mut Field
            store v0 at v1                      // known_values[v1] = v0
            call f1(v0)
            v2 = load v1 -> &mut Field          // should forward to v0
            return v2
        }
        brillig(inline) fn f1 f1 {
          b0(v10: &mut Field):
            store Field 99 at v10
            return
        }
    ";
        // Known: known_values[v1] survives; load v1 forwards to v0.
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 42 at v0
            v2 = allocate -> &mut &mut Field
            store v0 at v2
            call f1(v0)
            return v0
        }
        brillig(inline) fn f1 f1 {
          b0(v0: &mut Field):
            store Field 99 at v0
            return
        }
    ");
    }

    #[test]
    fn dead_store_via_must_alias_block_param() {
        // The block parameter v1 inherits v0's allocation site (single-pred join
        // in track_allocations_from_predecessors). v0 and v1 are distinct SSA
        // values but must-alias. The store at v1 is then killed by the store at
        // v0 even though they are not the same SSA value.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            jmp b1(v0)
          b1(v1: &mut Field):
            store Field 1 at v1
            store Field 2 at v0
            v2 = load v0 -> Field
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v1 = allocate -> &mut Field
            jmp b1(v1)
          b1(v0: &mut Field):
            store Field 2 at v1
            return Field 2
        }
        ");
    }

    #[test]
    fn load_forward_via_must_alias_block_param() {
        // Symmetric to the dead-store case: a store at v0 is forwarded through
        // a load at v1, which is must-aliased to v0 via the block-param join.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            jmp b1(v0)
          b1(v1: &mut Field):
            store Field 42 at v0
            v2 = load v1 -> Field
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v1 = allocate -> &mut Field
            jmp b1(v1)
          b1(v0: &mut Field):
            store Field 42 at v1
            return Field 42
        }
        ");
    }

    /// load_store_forwarding incorrectly forwards a store across two call
    /// sites of a non-recursive callee. Each call to `f1` allocates a
    /// fresh `inner` cell; the store at `v1` writes to the first call's
    /// `inner`, and the load at `v4` reads through the second call's
    /// `inner`. Because pass 2 of alias_analysis assigns
    /// `Known(f1::inner)` to both `v1` and `v3` — and `is_trusted` does
    /// not account for multi-call-site amplification of a non-recursive
    /// callee — the forwarding pass keys both under the same trusted
    /// site and replaces `v4` with `Field 1`.
    ///
    /// Sound output: `v4 = load v3 -> Field` must remain (or fold to
    /// `Field 0`, the value `f1` stores into `inner` on every entry).
    /// It must NOT fold to `Field 1`.
    #[test]
    fn load_forward_unsound_across_multi_call_site_non_recursive_callee() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = call f1() -> &mut &mut Field
            v1 = load v0 -> &mut Field
            store Field 1 at v1
            v2 = call f1() -> &mut &mut Field
            v3 = load v2 -> &mut Field
            v4 = load v3 -> Field
            return v4
        }
        brillig(inline) fn f1 f1 {
          b0():
            v0 = allocate -> &mut Field
            store Field 0 at v0
            v1 = allocate -> &mut &mut Field
            store v0 at v1
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();
        // The load at `v4` reads from the *second* call's `inner` cell —
        // not the first's. The pass must not replace `v4` with a numeric
        // constant. (We accept either a remaining Load, or a fold to
        // `Field 0` — the value `f1` stores into `inner` on every entry —
        // but never `Field 1`.)
        let main = ssa.main();
        let returned = match main.dfg[main.entry_block()].terminator() {
            Some(crate::ssa::ir::instruction::TerminatorInstruction::Return {
                return_values, ..
            }) => return_values[0],
            _ => panic!("expected a Return terminator with one value"),
        };
        if let crate::ssa::ir::value::Value::NumericConstant { constant, .. } =
            &main.dfg[returned]
        {
            assert_ne!(
                format!("{constant:?}"),
                "1",
                "load through the second call's result was wrongly \
                 forwarded to `Field 1` (the value stored into the \
                 *first* call's `inner` cell). The two `inner` cells \
                 are distinct: must_alias is unsound across multiple \
                 call sites of a non-recursive callee."
            );
        }
    }

    #[test]
    fn dead_store_and_forward_via_must_alias_ifelse() {
        // v1 has site Some(v1); v2 (block-param) inherits Some(v1); IfElse
        // joining v1 and v2 produces v4 with site Some(v1). v4 must-aliases
        // v1 even though they are distinct SSA values, so the store at v1 is
        // dead and the load at v1 forwards from the store at v4.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            jmp b1(v1)
          b1(v2: &mut Field):
            v3 = not v0
            v4 = if v0 then v1 else (if v3) v2
            store Field 1 at v1
            store Field 2 at v4
            v5 = load v1 -> Field
            return v5
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1):
            v2 = allocate -> &mut Field
            jmp b1(v2)
          b1(v1: &mut Field):
            v3 = not v0
            v4 = if v0 then v2 else (if v3) v1
            store Field 2 at v4
            return Field 2
        }
        ");
    }
}
