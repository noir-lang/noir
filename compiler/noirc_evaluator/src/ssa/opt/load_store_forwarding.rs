//! Per-block load/store forwarding pass.
//!
//! This pass performs simple, fast, per-block optimizations:
//! - **Load forwarding**: If a load reads from an address whose value is already known
//!   (from a prior store in the same block), replace the load with the known value.
//! - **Dead store elimination**: If two stores write to the same address with no
//!   intervening load, the first store is dead and can be removed.
//!
//! This pass does not track values across block boundaries (that is handled by
//! `mem2reg` which promotes variables to block parameters). It is designed
//! to be fast on large, single-block ACIR functions that result from inlining,
//! unrolling, and flattening.
//!
//! ## Alias handling
//!
//! All alias reasoning is delegated to [`AliasAnalysis`], which provides:
//! - `is_loop_aliased` / `is_allocation` / `may_alias` — alias queries for stores
//! - `addresses_modified_by_call` — which addresses a call may write through
//! - `get_known_at_entry` — cross-block known values to seed per-block forwarding
//!
//! See also: <https://github.com/noir-lang/noir/issues/12005>
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
    ssa_gen::Ssa,
};

use super::alias_analysis::AliasAnalysis;

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
        let alias_analysis = AliasAnalysis::new(self);

        let mut inserter = FunctionInserter::new(self);
        let blocks = PostOrder::with_function(inserter.function).into_vec_reverse();

        // Single pass in RPO: forward loads/stores and remap instructions.
        // RPO guarantees predecessors are visited before successors (in acyclic
        // graphs), so value mappings from forwarded loads are always available
        // before blocks that use those values.
        for block in &blocks {
            let block = *block;
            let initial_known: HashMap<ValueId, ValueId> = alias_analysis
                .get_known_at_entry(block)
                .map(|known| {
                    known
                        .iter()
                        .map(|(addr, val)| (inserter.resolve(*addr), inserter.resolve(*val)))
                        .collect()
                })
                .unwrap_or_default();

            let instructions_to_remove = forward_loads_and_stores_in_block(
                &mut inserter,
                block,
                &alias_analysis,
                initial_known,
            );

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
                if !instructions_to_remove.contains(instruction_id) {
                    inserter.push_instruction(*instruction_id, block, true);
                }
            }
            inserter.map_terminator_in_place(block);
        }
        inserter.map_data_bus_in_place();
    }
}

/// Perform load/store forwarding within a single block.
///
/// Returns the set of instructions to remove from the block.
fn forward_loads_and_stores_in_block(
    inserter: &mut FunctionInserter,
    block: BasicBlockId,
    alias_analysis: &AliasAnalysis,
    known_values: HashMap<ValueId, ValueId>,
) -> HashSet<InstructionId> {
    let mut known_values = known_values;
    let mut last_stores: HashMap<ValueId, InstructionId> = HashMap::default();
    // Maps address -> last load result (for load-to-load forwarding).
    // Kept separate from known_values so that load entries don't interfere
    // with the store handler's clear-on-unknown-store alias heuristic.
    let mut last_loads: HashMap<ValueId, ValueId> = HashMap::default();
    let mut instructions_to_remove: HashSet<InstructionId> = HashSet::default();

    let instructions = inserter.function.dfg[block].instructions().to_vec();

    for instruction_id in instructions {
        let instruction = &inserter.function.dfg[instruction_id];
        match instruction {
            Instruction::Store { address, value } => {
                let is_loop_aliased = alias_analysis.is_loop_aliased(*address);
                let address = inserter.resolve(*address);
                let value = inserter.resolve(*value);
                let dfg = &inserter.function.dfg;

                if is_loop_aliased {
                    // Loop-aliased addresses could alias any same-typed address
                    // across iterations. Use type discrimination to preserve
                    // entries with different types.
                    let addr_type = dfg.type_of_value(address);
                    known_values.retain(|k, _| dfg.type_of_value(*k) != addr_type);
                    last_loads.retain(|k, _| dfg.type_of_value(*k) != addr_type);
                    last_stores.retain(|k, _| dfg.type_of_value(*k) != addr_type);
                } else {
                    known_values.retain(|k, _| !alias_analysis.may_alias(address, *k, dfg));
                    last_loads.retain(|k, _| !alias_analysis.may_alias(address, *k, dfg));
                    if let Some(prev_store) = last_stores.get(&address) {
                        instructions_to_remove.insert(*prev_store);
                    }
                    last_stores.retain(|k, _| !alias_analysis.may_alias(address, *k, dfg));
                }

                // A store supersedes any prior load from this address.
                last_loads.remove(&address);
                known_values.insert(address, value);
                last_stores.insert(address, instruction_id);
            }
            Instruction::Load { address } => {
                let is_loop_aliased = alias_analysis.is_loop_aliased(*address);
                let address = inserter.resolve(*address);
                let dfg = &inserter.function.dfg;

                // If loading from a loop-aliased address and the result is a
                // reference, the loaded reference could alias any same-typed
                // address across iterations. Invalidate same-typed caches
                // (analogous to how the store handler treats loop-aliased stores).
                if is_loop_aliased {
                    let result = dfg.instruction_results(instruction_id)[0];
                    let result_type = dfg.type_of_value(result);
                    if result_type.contains_reference() {
                        known_values.retain(|k, _| dfg.type_of_value(*k) != result_type);
                        last_loads.retain(|k, _| dfg.type_of_value(*k) != result_type);
                        last_stores.retain(|k, _| dfg.type_of_value(*k) != result_type);
                    }
                }

                if let Some(value) = known_values.get(&address) {
                    // Store-to-load: we know the value from a prior store.
                    let result = inserter.function.dfg.instruction_results(instruction_id)[0];
                    inserter.map_value(result, *value);
                    instructions_to_remove.insert(instruction_id);
                } else if let Some(prev_result) = last_loads.get(&address) {
                    // Load-to-load: no store to this address since the last load,
                    // so we can reuse the previous load's result.
                    let result = inserter.function.dfg.instruction_results(instruction_id)[0];
                    inserter.map_value(result, *prev_result);
                    instructions_to_remove.insert(instruction_id);
                } else {
                    // No known value — record for future load-to-load forwarding.
                    let result = inserter.function.dfg.instruction_results(instruction_id)[0];
                    last_loads.insert(address, result);
                }

                // A load from address X "uses" any store to an address that
                // may_alias(X). Those stores must not be eliminated as dead.
                let dfg = &inserter.function.dfg;
                last_stores.retain(|k, _| !alias_analysis.may_alias(address, *k, dfg));
            }
            Instruction::Call { .. } => {
                match alias_analysis.addresses_modified_by_call(
                    instruction,
                    &inserter.function.dfg,
                    |v| inserter.resolve(v),
                ) {
                    Some(addrs) => {
                        for addr in &addrs {
                            known_values.remove(addr);
                            last_loads.remove(addr);
                            last_stores.remove(addr);
                        }
                    }
                    None => {
                        known_values.clear();
                        last_loads.clear();
                        last_stores.clear();
                    }
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
    fn cross_block_forwarded_through_single_predecessor() {
        // A store in b0 should forward to a load in b1 when b0 is b1's only predecessor.
        // The alias analysis propagates known values across block boundaries.
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
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 5 at v0
            jmp b1()
          b1():
            return Field 5
        }
        ");
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
    fn forwarding_in_later_block_remaps_earlier_block() {
        // Regression test: the SSA text format assigns block IDs in declaration
        // order, but RPO iteration follows the CFG. The block that creates the
        // forwarding mapping (b2) must be visited before the block that uses
        // the forwarded value (b1). RPO guarantees this since b2 is a
        // predecessor of b1 in the CFG: b0 -> b2 -> b1.
        //
        // b2 has allocate+store+load in the same block, so per-block forwarding
        // maps v2 -> u32 10. b1 uses v2 in an add.
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

        // v2 (load result from b2) must be forwarded to u32 10 in b1's add.
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
    fn forwarding_in_loop_block_without_aliases() {
        // A loop block with store+load to a local allocation and no reference
        // stores should benefit from forwarding (no loop-carried aliases).
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
    fn loop_call_modifying_reference_not_forwarded() {
        // A loop body calls a function that stores through a mutable reference.
        // The alias analysis must not forward the pre-loop value through the
        // loop exit, since the call modifies the reference.
        //
        // Reduced from test_programs/execution_success/uhashmap.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut u32
            store u32 0 at v0
            jmp b1(u32 0)
          b1(v1: u32):
            v4 = lt v1, u32 2
            jmpif v4 then: b2(), else: b3()
          b2():
            call f1(v0)
            v5 = unchecked_add v1, u32 1
            jmp b1(v5)
          b3():
            v6 = load v0 -> u32
            constrain v6 == u32 1
            return
        }
        brillig(inline) fn set_to_one f1 {
          b0(v0: &mut u32):
            store u32 1 at v0
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        // Verify the SSA interprets correctly before the pass
        let before = ssa.interpret(vec![]).expect("Before failed");

        let ssa = Ssa::from_str(src).unwrap();
        let after = ssa.load_store_forwarding();
        let result = after.interpret(vec![]).expect("After LSF failed");
        assert_eq!(before, result, "LSF changed program semantics");
    }

    #[test]
    fn loop_call_with_container_of_references_not_forwarded() {
        // A loop body calls a function with an array containing a reference.
        // The callee can extract the reference and store through it, so the
        // alias analysis must not forward the pre-loop value through the exit.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut u32
            store u32 0 at v0
            v1 = make_array [v0] : [&mut u32; 1]
            jmp b1(u32 0)
          b1(v2: u32):
            v5 = lt v2, u32 1
            jmpif v5 then: b2(), else: b3()
          b2():
            call f1(v1)
            v6 = unchecked_add v2, u32 1
            jmp b1(v6)
          b3():
            v7 = load v0 -> u32
            constrain v7 == u32 1
            return
        }
        brillig(inline) fn set_via_array f1 {
          b0(v0: [&mut u32; 1]):
            v1 = array_get v0, index u32 0 -> &mut u32
            store u32 1 at v1
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let before = ssa.interpret(vec![]).expect("Before failed");

        let ssa = Ssa::from_str(src).unwrap();
        let after = ssa.load_store_forwarding();
        let result = after.interpret(vec![]).expect("After LSF failed");
        assert_eq!(before, result, "LSF changed program semantics");
    }

    #[test]
    fn alias_through_load_not_forwarded_cross_block() {
        // Ported from old mem2reg: load_aliases_in_predecessor_block.
        // v3 and v4 both loaded from v2 (&mut &mut Field) — they alias.
        // A store through v4 in b1 must invalidate v3's known value.
        // The cross-block analysis must NOT forward Field 0 (from b0) to b1's load of v3.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 0 at v0
            v2 = allocate -> &mut &mut Field
            store v0 at v2
            v3 = load v2 -> &mut Field
            v4 = load v2 -> &mut Field
            jmp b1()
          b1():
            store Field 1 at v3
            store Field 2 at v4
            v7 = load v3 -> Field
            constrain v7 == Field 2
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let before = ssa.interpret(vec![]).expect("Before failed");

        let ssa = Ssa::from_str(src).unwrap();
        let after = ssa.load_store_forwarding();
        let result = after.interpret(vec![]).expect("After LSF failed");
        assert_eq!(before, result, "LSF changed program semantics");
    }

    #[test]
    fn call_with_nested_reference_clears_all_known_values() {
        // Regression: a &mut &mut Field argument lets the callee load the outer
        // reference to obtain the inner reference and store through it.
        // The inner ref's known value must be invalidated.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 42 at v0
            v1 = allocate -> &mut &mut Field
            store v0 at v1
            call f1(v1)
            v2 = load v0 -> Field
            constrain v2 == Field 99
            return
        }
        brillig(inline) fn modify_inner f1 {
          b0(v0: &mut &mut Field):
            v1 = load v0 -> &mut Field
            store Field 99 at v1
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let before = ssa.interpret(vec![]).expect("Before failed");

        let ssa = Ssa::from_str(src).unwrap();
        let after = ssa.load_store_forwarding();
        let result = after.interpret(vec![]).expect("After LSF failed");
        assert_eq!(before, result, "LSF changed program semantics");
    }

    #[test]
    fn loop_call_with_nested_reference_not_forwarded() {
        // A loop body calls a function with a nested reference (&mut &mut u32).
        // The callee loads the outer ref to reach the inner ref and stores through it.
        // The loop alias analysis must treat all allocations as potentially modified.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut u32
            store u32 0 at v0
            v1 = allocate -> &mut &mut u32
            store v0 at v1
            jmp b1(u32 0)
          b1(v2: u32):
            v5 = lt v2, u32 1
            jmpif v5 then: b2(), else: b3()
          b2():
            call f1(v1)
            v6 = unchecked_add v2, u32 1
            jmp b1(v6)
          b3():
            v7 = load v0 -> u32
            constrain v7 == u32 1
            return
        }
        brillig(inline) fn set_via_nested f1 {
          b0(v0: &mut &mut u32):
            v1 = load v0 -> &mut u32
            store u32 1 at v1
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let before = ssa.interpret(vec![]).expect("Before failed");

        let ssa = Ssa::from_str(src).unwrap();
        let after = ssa.load_store_forwarding();
        let result = after.interpret(vec![]).expect("After LSF failed");
        assert_eq!(before, result, "LSF changed program semantics");
    }

    // --- Parameter aliasing ---

    #[test]
    fn parameter_alias() {
        // Function parameters could alias each other. The load of v0 after
        // storing to v1 must not be forwarded because v0 and v1 might be
        // the same reference.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: &mut Field, v1: &mut Field):
            store Field 0 at v0
            store Field 0 at v1
            v3 = load v0 -> Field
            constrain v3 == Field 0
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn parameter_alias_nested_reference() {
        // Even when a third parameter is a nested reference, the aliasing
        // between v0 and v1 means loads can't be forwarded.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: &mut Field, v1: &mut Field, v2: &mut &mut Field):
            store Field 0 at v0
            store Field 0 at v1
            v3 = load v0 -> Field
            constrain v3 == Field 0
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    // --- Array element aliasing ---

    #[test]
    fn does_not_reuse_load_from_aliased_array_element() {
        // An array_get can extract a reference that aliases another parameter.
        // The second load of v0 must not reuse the first load's value since
        // the store through v6 (from array_get) may have modified v0.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: &mut Field, v1: &mut Field, v2: u32):
            v3 = make_array [v0] : [&mut Field; 1]
            v4 = array_set v3, index v2, value v1
            v5 = load v0 -> Field
            v6 = array_get v4, index v2 -> &mut Field
            store Field 0 at v6
            v7 = load v0 -> Field
            return v7
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();

        // The store through v6 (from array_get) clears known values, so the
        // second load of v0 is NOT forwarded. The array_get index simplifies
        // during re-insertion but the loads correctly remain.
        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn main f0 {
          b0(v0: &mut Field, v1: &mut Field, v2: u32):
            v3 = make_array [v0] : [&mut Field; 1]
            v4 = array_set v3, index v2, value v1
            v5 = load v0 -> Field
            constrain v2 == u32 0, "Index out of bounds"
            v7 = array_get v4, index u32 0 -> &mut Field
            store Field 0 at v7
            v9 = load v0 -> Field
            return v9
        }
        "#);
    }

    #[test]
    fn does_not_remove_store_from_aliased_array_element() {
        // A reference stored into an array and extracted via array_get could
        // alias the original allocation. The store through v6 modifies v1's
        // memory, so the final load must see Field 100.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v1 = allocate -> &mut Field
            store Field 0 at v1
            v3 = make_array [v1] : [&mut Field; 1]
            v5 = array_set v3, index v0, value v1
            v6 = array_get v5, index v0 -> &mut Field
            store Field 100 at v6
            v8 = load v1 -> Field
            constrain v8 == Field 100
            return v8
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.load_store_forwarding();

        // The store through v6 (alias of v1) clears v1's known value, so the
        // load and constrain are preserved. The array_get index simplifies
        // during re-insertion.
        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v1 = allocate -> &mut Field
            store Field 0 at v1
            v3 = make_array [v1] : [&mut Field; 1]
            v4 = array_set v3, index v0, value v1
            constrain v0 == u32 0, "Index out of bounds"
            v6 = array_get v4, index u32 0 -> &mut Field
            store Field 100 at v6
            v8 = load v1 -> Field
            constrain v8 == Field 100
            return v8
        }
        "#);
    }

    // --- Block parameter aliasing ---

    #[test]
    fn block_argument_is_alias_of_block_parameter_1() {
        // v0 is passed as a jmp argument to b1, creating block parameter v1.
        // v1 aliases v0, so storing through v1 must invalidate v0's known value.
        let src = "
        brillig(inline) impure fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 0 at v0
            jmp b1(v0)
          b1(v1: &mut Field):
            store Field 1 at v1
            v2 = load v0 -> Field
            return v2
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    #[test]
    fn block_argument_is_alias_of_block_parameter_2() {
        // Same aliasing as above, but the load is through v1 after storing to v0.
        let src = "
        brillig(inline) impure fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 0 at v0
            jmp b1(v0)
          b1(v1: &mut Field):
            store Field 1 at v1
            store Field 2 at v0
            v2 = load v1 -> Field
            return v2
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    // --- IfElse aliasing ---

    #[test]
    fn if_aliases_each_branch() {
        // The IfElse result v6 could be v1 or v3 depending on the condition.
        // Storing through v6 must invalidate both v1 and v3's known values.
        let src = "
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            store Field 0 at v1
            v3 = allocate -> &mut Field
            store Field 1 at v3
            v5 = not v0
            v6 = if v0 then v1 else (if v5) v3
            store Field 9 at v6
            v8 = load v1 -> Field
            constrain v8 == Field 9
            v9 = load v3 -> Field
            constrain v9 == Field 1
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    // --- Cross-block array_get aliasing ---

    #[test]
    fn store_to_reference_from_array_get_is_not_lost() {
        // A reference extracted from an array via array_get could alias the
        // original allocation. The store through v7 must not be lost.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1):
            v2 = allocate -> &mut Field
            store Field 0 at v2
            jmpif v0 then: b1(), else: b2()
          b1():
            v5 = make_array [v2] : [&mut Field; 1]
            jmp b3(v5)
          b2():
            v4 = make_array [v2] : [&mut Field; 1]
            jmp b3(v4)
          b3(v1: [&mut Field; 1]):
            v7 = array_get v1, index u32 0 -> &mut Field
            store Field 9 at v7
            v9 = array_get v1, index u32 0 -> &mut Field
            v10 = load v9 -> Field
            constrain v10 == Field 9
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    // --- Call return value aliasing ---

    #[test]
    fn call_return_aliases_allocation() {
        // A call that returns a reference can return one of its input references.
        // v2 (call return) aliases v1 (allocation). After storing Field 2 at v1,
        // loading v2 must see Field 2, not the stale Field 1.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v1 = allocate -> &mut Field
            store Field 0 at v1
            v2 = call f1(v1) -> &mut Field
            store Field 1 at v2
            store Field 2 at v1
            v8 = load v2 -> Field
            constrain v8 == Field 2
            return
        }
        brillig(inline) fn helper f1 {
          b0(v0: &mut Field):
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let before = ssa.interpret(vec![]).expect("Before failed");

        let ssa = Ssa::from_str(src).unwrap();
        let after = ssa.load_store_forwarding();
        let result = after.interpret(vec![]).expect("After LSF failed");
        assert_eq!(before, result, "LSF changed program semantics");
    }

    #[test]
    fn call_return_with_array_aliases_allocation() {
        // Similar to the above but the reference is passed/returned inside an array.
        // v5 (extracted from call return) aliases v1 (allocation).
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v1 = allocate -> &mut Field
            store Field 0 at v1
            v3 = make_array [v1] : [&mut Field; 1]
            v4 = call f1(v3) -> [&mut Field; 1]
            v5 = array_get v4, index u32 0 -> &mut Field
            store Field 1 at v5
            store Field 2 at v1
            v8 = load v5 -> Field
            constrain v8 == Field 2
            return
        }
        brillig(inline) fn helper f1 {
          b0(v0: [&mut Field; 1]):
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let before = ssa.interpret(vec![]).expect("Before failed");

        let ssa = Ssa::from_str(src).unwrap();
        let after = ssa.load_store_forwarding();
        let result = after.interpret(vec![]).expect("After LSF failed");
        assert_eq!(before, result, "LSF changed program semantics");
    }

    #[test]
    fn call_return_aliases_with_existing_param_aliases() {
        // v0 and v1 are parameters (potential aliases). v2 = call f1(v1) could
        // alias v1. Then store to v0 should invalidate v2.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: &mut Field, v1: &mut Field):
            store Field 0 at v1
            v2 = call f1(v1) -> &mut Field
            store Field 1 at v2
            store Field 2 at v0
            v8 = load v2 -> Field
            constrain v8 == Field 2
            return
        }
        brillig(inline) fn helper f1 {
          b0(v0: &mut Field):
            return v0
        }
        ";
        // Can't use interpreter (entry block has reference params).
        // This test verifies LSF doesn't change the SSA — store to v0
        // (unknown param) clears all known values including v2.
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    // --- Regression tests ---

    #[test]
    fn regression_10070_array_get_alias_of_allocation() {
        // v8 (from array_get on a block parameter) could alias v3 or v4
        // depending on the branch. After storing to v3/v4 (allocations),
        // loading v8 must not return the stale value.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v_dummy = allocate -> &mut Field
            v0 = make_array [v_dummy] : [&mut Field; 1]
            v3 = allocate -> &mut Field
            v4 = allocate -> &mut Field
            jmpif u1 1 then: b1(), else: b2()
          b1():
            v7 = array_set v0, index u32 0, value v3
            jmp b3(v7)
          b2():
            v6 = array_set v0, index u32 0, value v4
            jmp b3(v6)
          b3(v2: [&mut Field; 1]):
            v8 = array_get v2, index u32 0 -> &mut Field
            store Field 1 at v8
            store Field 2 at v3
            store Field 3 at v4
            v12 = load v8 -> Field
            return v12
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let before = ssa.interpret(vec![]).expect("Before failed");

        let ssa = Ssa::from_str(src).unwrap();
        let after = ssa.load_store_forwarding();
        let result = after.interpret(vec![]).expect("After LSF failed");
        assert_eq!(before, result, "LSF changed program semantics");
    }

    #[test]
    fn regression_10020_loop_array_get_alias() {
        // v9 (from array_get on v4 = [v1, v3]) could alias v1 depending on v0.
        // After storing to v1 (allocation), loading v9 must not return stale value.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v1 = allocate -> &mut Field
            store Field 0 at v1
            v3 = allocate -> &mut Field
            store Field 0 at v3
            v4 = make_array [v1, v3] : [&mut Field; 2]
            v5 = allocate -> &mut Field
            store Field 0 at v5
            jmp b1(u32 0)
          b1(v0: u32):
            v7 = eq v0, u32 0
            jmpif v7 then: b2(), else: b3()
          b2():
            v9 = array_get v4, index v0 -> &mut Field
            store Field 1 at v9
            store Field 2 at v1
            v12 = load v5 -> Field
            v13 = load v9 -> Field
            v14 = add v12, v13
            store v14 at v5
            v16 = unchecked_add v0, u32 1
            jmp b1(v16)
          b3():
            v8 = load v5 -> Field
            return v8
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let before = ssa.interpret(vec![]).expect("Before failed");

        let ssa = Ssa::from_str(src).unwrap();
        let after = ssa.load_store_forwarding();
        let result = after.interpret(vec![]).expect("After LSF failed");
        assert_eq!(before, result, "LSF changed program semantics");
    }

    // --- Load-to-load forwarding ---

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

    // === Issue reproduction tests (milestone 49 audit) ===

    // Issue #12217: carried aliases don't take call inputs into account
    #[test]
    fn issue_12217_loop_carried_alias_via_call() {
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

    // Issue #12219: carried aliases don't take call returned references into account
    #[test]
    fn issue_12219_loop_carried_alias_via_call_return() {
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

    // Issue #12220: carried aliases don't take array_get into account
    #[test]
    fn issue_12220_loop_carried_alias_via_array_get() {
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

    // Issue #12221: carried aliases don't take jmpif into account
    #[test]
    fn issue_12221_loop_carried_alias_via_jmpif() {
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

    // Issue #12222: carried aliases misses nested references in Form 1
    #[test]
    fn issue_12222_loop_carried_alias_array_containing_refs() {
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

    // Issue #12223: carried aliases misses nested references in Form 2
    #[test]
    fn issue_12223_loop_carried_alias_array_header_param() {
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

    // Issue #12225: load-to-load incorrectly deduplicated when array_get alias exists
    #[test]
    fn issue_12225_load_to_load_array_get_alias() {
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
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    // Issue #12230: load does not mark aliased stores as used
    #[test]
    fn issue_12230_aliased_load_prevents_dead_store() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: &mut Field, v1: &mut Field):
            store Field 1 at v0
            v2 = load v1 -> Field
            store Field 2 at v0
            return v2
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    // Issue #12231: local allocations aliases should be tracked
    #[test]
    fn issue_12231_local_allocation_alias() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: &mut Field, v1: u1):
            v2 = allocate -> &mut Field
            v3 = not v1
            v4 = if v1 then v0 else (if v3) v2
            store Field 0 at v4
            store Field 1 at v2
            v5 = load v4 -> Field
            return v5
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }

    // Issue #12232: Load ignore loop carried aliases (test 1: dead store)
    #[test]
    fn issue_12232_loop_aliased_load_dead_store() {
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

    // Issue #12232: Load ignore loop carried aliases (test 2: load forwarding)
    #[test]
    fn issue_12232_loop_aliased_load_forwarding() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: &mut Field):
            v1 = allocate -> &mut &mut Field
            store v0 at v1
            jmp b1()
          b1():
            store Field 1 at v0
            v3 = load v1 -> &mut Field
            v4 = load v3 -> Field
            store Field 2 at v0
            v5 = load v3 -> Field
            store v3 at v1
            jmp b1()
        }
        ";
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
    }
}
