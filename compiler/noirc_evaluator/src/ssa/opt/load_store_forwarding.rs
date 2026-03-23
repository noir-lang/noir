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
//! Blocks that are part of loops are skipped, since loop-carried aliases can
//! invalidate per-block assumptions about which addresses alias each other.
//!
//! The pass conservatively clears known values when a reference is passed to a
//! function call, since the callee could read or modify the referenced memory.
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
    opt::Loops,
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
    fn load_store_forwarding(&mut self) {
        let loop_blocks = collect_loop_blocks(self);

        let mut inserter = FunctionInserter::new(self);
        let blocks = PostOrder::with_function(inserter.function).into_vec_reverse();

        // Single pass in RPO: forward loads/stores and remap instructions.
        // RPO guarantees predecessors are visited before successors (in acyclic
        // graphs), so value mappings from forwarded loads are always available
        // before blocks that use those values.
        for block in &blocks {
            let block = *block;
            if !loop_blocks.contains(&block) {
                let instructions_to_remove =
                    forward_loads_and_stores_in_block(&mut inserter, block);

                if !instructions_to_remove.is_empty() {
                    inserter.function.dfg[block]
                        .instructions_mut()
                        .retain(|id| !instructions_to_remove.contains(id));
                }
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

/// Collect all blocks that belong to any loop in the function.
fn collect_loop_blocks(function: &Function) -> HashSet<BasicBlockId> {
    use super::unrolling::LoopOrder;
    let loops = Loops::find_all(function, LoopOrder::InsideOut);
    let mut loop_blocks = HashSet::default();
    for loop_ in &loops.yet_to_unroll {
        loop_blocks.extend(&loop_.blocks);
    }
    loop_blocks
}

/// Perform load/store forwarding within a single block.
///
/// Returns the set of instructions to remove from the block.
///
/// ## Alias safety: clear-on-unknown-store
///
/// Instructions like `MakeArray`, `ArraySet`, `ArrayGet`, `IncrementRc`, etc. move
/// references around but don't modify pointed-to memory, so we keep known values
/// through them. Soundness is maintained by the Store handler: when a store writes
/// to an address that is neither a local allocation nor already in `known_values`,
/// it could be an alias (e.g. extracted via `array_get`), so we conservatively
/// clear all known values. Calls are handled separately since the callee could
/// dereference and write through any reference argument.
fn forward_loads_and_stores_in_block(
    inserter: &mut FunctionInserter,
    block: BasicBlockId,
) -> HashSet<InstructionId> {
    // Maps address → last stored value (after resolving through the inserter)
    let mut known_values: HashMap<ValueId, ValueId> = HashMap::default();
    // Maps address → last store instruction (candidate for dead store elimination)
    let mut last_stores: HashMap<ValueId, InstructionId> = HashMap::default();
    let mut instructions_to_remove: HashSet<InstructionId> = HashSet::default();
    // Track addresses from Allocate instructions in this block.
    // These are definitionally fresh and cannot alias anything else.
    let mut local_allocations: HashSet<ValueId> = HashSet::default();

    let instructions = inserter.function.dfg[block].instructions().to_vec();

    for instruction_id in instructions {
        let instruction = &inserter.function.dfg[instruction_id];
        match instruction {
            Instruction::Allocate => {
                let result = inserter.function.dfg.instruction_results(instruction_id)[0];
                local_allocations.insert(result);
            }
            Instruction::Store { address, value } => {
                let address = inserter.resolve(*address);
                let value = inserter.resolve(*value);

                if !known_values.contains_key(&address) && !local_allocations.contains(&address) {
                    // This address wasn't allocated locally and wasn't seen in a prior
                    // store — it could be an alias of an existing known reference
                    // (e.g. extracted via array_get). Conservatively clear all known
                    // reference values.
                    known_values.clear();
                    last_stores.clear();
                } else if let Some(prev_store) = last_stores.get(&address) {
                    // Previous store to this known/local address with no intervening
                    // load is dead.
                    instructions_to_remove.insert(*prev_store);
                }

                known_values.insert(address, value);
                last_stores.insert(address, instruction_id);
            }
            Instruction::Load { address } => {
                let address = inserter.resolve(*address);

                if let Some(value) = known_values.get(&address) {
                    // We know the value at this address — replace the load result.
                    let result = inserter.function.dfg.instruction_results(instruction_id)[0];
                    inserter.map_value(result, *value);
                    instructions_to_remove.insert(instruction_id);
                }

                // This address was loaded from, so the last store to it is not dead.
                last_stores.remove(&address);
            }
            Instruction::Call { .. } => {
                // A call could dereference and modify any reference argument.
                instruction.for_each_value(|value| {
                    let value = inserter.resolve(value);
                    if inserter.function.dfg.value_is_reference(value) {
                        known_values.remove(&value);
                        last_stores.remove(&value);
                    }
                });
            }
            _ => {
                // MakeArray, ArraySet, ArrayGet, IfElse, IncrementRc, DecrementRc, etc.
                // don't modify pointed-to memory — they just move references around.
                // Safe to keep known values. The Store handler's clear-on-unknown-store
                // ensures soundness if an alias is later written through.
            }
        }
    }

    // Any remaining entries in last_stores are stores with no subsequent load in
    // this block. We do NOT remove them here because they may be needed by successor
    // blocks or by later passes. Only truly dead stores (overwritten before being
    // read) are removed above.

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
}
