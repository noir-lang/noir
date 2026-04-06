//! Per-block load/store forwarding pass.
//!
//! This pass performs simple, fast, per-block optimizations:
//! - **Load forwarding**: If a load reads from an address whose value is already known
//!   (from a prior store in the same block), replace the load with the known value.
//! - **Dead store elimination**: If two stores write to the same address with no
//!   intervening load, the first store is dead and can be removed.
//!
//! This pass does not track values across block boundaries (that is handled by
//! `mem2reg_simple` which promotes variables to block parameters). It is designed
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

            // Remap instructions and terminator immediately — all predecessor
            // mappings are already in the inserter thanks to RPO ordering.
            for instruction_id in inserter.function.dfg[block].instructions().to_vec() {
                inserter.map_instruction_in_place(instruction_id);
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
    let mut instructions_to_remove: HashSet<InstructionId> = HashSet::default();

    let instructions = inserter.function.dfg[block].instructions().to_vec();

    for instruction_id in instructions {
        let instruction = &inserter.function.dfg[instruction_id];
        match instruction {
            Instruction::Store { address, value } => {
                let is_loop_aliased = alias_analysis.is_loop_aliased(*address);
                let address = inserter.resolve(*address);
                let value = inserter.resolve(*value);

                if is_loop_aliased
                    || (!known_values.contains_key(&address)
                        && !alias_analysis.is_allocation(address))
                {
                    known_values.clear();
                    last_stores.clear();
                } else {
                    known_values
                        .retain(|k, _| *k == address || !alias_analysis.may_alias(address, *k));
                    if let Some(prev_store) = last_stores.get(&address) {
                        instructions_to_remove.insert(*prev_store);
                    }
                    last_stores
                        .retain(|k, _| *k == address || !alias_analysis.may_alias(address, *k));
                }

                known_values.insert(address, value);
                last_stores.insert(address, instruction_id);
            }
            Instruction::Load { address } => {
                let address = inserter.resolve(*address);

                if let Some(value) = known_values.get(&address) {
                    let result = inserter.function.dfg.instruction_results(instruction_id)[0];
                    inserter.map_value(result, *value);
                    instructions_to_remove.insert(instruction_id);
                }

                last_stores.remove(&address);
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
                            last_stores.remove(addr);
                        }
                    }
                    None => {
                        known_values.clear();
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
            v3 = add Field 1, Field 2
            return v3
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
            v4 = add Field 1, Field 2
            return v4
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
        // The store to v2 (alias of v0) must clear v0's known value,
        // so the load should NOT be forwarded.
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
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
            v3 = add u32 10, u32 1
            return v3
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
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
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
        assert_ssa_does_not_change(src, Ssa::load_store_forwarding);
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
}
