//! Block-local cleanup that runs immediately after [`Ssa::remove_unreachable_instructions`].
//!
//! [`Ssa::remove_unreachable_instructions`] terminates a block with `Unreachable`
//! whenever it discovers an always-failing constraint, but it only deletes
//! instructions *after* that constraint. Reference plumbing produced *before*
//! the constraint — typically `Allocate`/`Store` chains and the `MakeArray`s
//! they flow into — stays behind. Those `Allocate`s eventually reach ACIR
//! codegen and trip [`RuntimeError::UnknownReference`] in
//! `compiler/noirc_evaluator/src/acir/mod.rs`.
//!
//! ## Why this isn't handled by mem2reg
//!
//! [`Ssa::mem2reg`] only optimizes references whose only uses are `Store`/
//! `Load` against the address — see
//! `collect_eligible_variables_and_def_sites` in `mem2reg`. The triggering
//! pattern is a nested `&mut [&mut &mut T; N]` bound to a `let mut`, whose
//! inner allocations escape into a `make_array`. That escape flips the
//! address into a first-class use and mem2reg excludes it, so the
//! `Allocate` survives every subsequent pass. This pass fires only on the
//! special case the forward sweep just created (a block now ending in
//! `Unreachable`) and reasons about observability rather than alias
//! analysis.
//!
//! ## What this pass drops
//!
//! Walking each `Unreachable`-terminated block backwards, an instruction is
//! removed when both:
//!
//! 1. It is one of `Allocate`/`Store`/`MakeArray`/`IncrementRc`/`DecrementRc`
//!    — the only opcodes whose sole purpose is to materialize references,
//!    arrays, or RC bookkeeping.
//! 2. Its results (or, for `Store`, its address) are not consumed by any
//!    retained instruction in the block or by the function's databus.
//!
//! Constraints, range checks, calls, arithmetic, casts, and any other
//! constraint-contributing instruction are deliberately retained, so the
//! set and order of constraints emitted to ACIR is unchanged.
//!
//! ## Aliasing
//!
//! `Call`, `ArrayGet`, `ArraySet`, and `Load` can all return values whose
//! type contains a reference, and that reference can alias something this
//! pass cannot track on its own. To keep aliasing-sensitive stores live
//! through the backward sweep, every kept (non-droppable) instruction's
//! reference-typed results are seeded into the live set up front. A
//! subsequent `Store` whose address is such a returned reference then
//! satisfies the `store_addr_used` check and is preserved.
//!
//! `make_array` of references is only dangerous through its consumers, so
//! the `make_array` itself is safe to drop when its result is unused —
//! the seeding above fires on the consumer (`array_get`/`array_set`/etc.)
//! that actually surfaces the aliased reference.
//!
//! ## Fail-secure invariants
//!
//! `flatten_cfg`, `Remove IfElse`, and the major inlining passes have all
//! run before [`Ssa::remove_unreachable_instructions`] (and therefore
//! before this pass). `Instruction::IfElse` reaching here would mean the
//! pipeline-ordering invariant was violated and aliasing through the
//! merged branches would be invisible to the seeding above, so it ICEs
//! before any removal happens.
use im::HashSet;

use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Instruction, InstructionId, TerminatorInstruction},
        value::ValueId,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    pub(crate) fn remove_dead_refs_in_unreachable_blocks(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.remove_dead_refs_in_unreachable_blocks();
        }
        self
    }
}

impl Function {
    fn remove_dead_refs_in_unreachable_blocks(&mut self) {
        let mut databus_used: HashSet<ValueId> = HashSet::new();
        let _ = self.dfg.data_bus.map_values(|v| {
            databus_used.insert(v);
            v
        });

        for block_id in self.reachable_blocks() {
            let is_unreachable = matches!(
                self.dfg[block_id].terminator(),
                Some(TerminatorInstruction::Unreachable { .. })
            );
            if !is_unreachable {
                continue;
            }

            let instruction_ids = self.dfg[block_id].take_instructions();

            // Fail-secure: `IfElse` must already have been lowered before this
            // pass; encountering one here means the pipeline-ordering
            // invariant was violated.
            for &instruction_id in &instruction_ids {
                if matches!(self.dfg[instruction_id], Instruction::IfElse { .. }) {
                    panic!(
                        "remove_dead_refs_in_unreachable_blocks: encountered `IfElse` in a block \
                         whose terminator is `Unreachable`. The `Remove IfElse` pass is expected \
                         to run before this one; please ensure the SSA pipeline order is preserved."
                    );
                }
            }

            // Aliasing: reference-typed results of non-droppable instructions
            // (`Call`/`ArrayGet`/`ArraySet`/`Load`/…) can alias something
            // outside this pass's view. Seed those into the live set so any
            // `Store` that targets one survives the backward sweep below.
            let mut used: HashSet<ValueId> = databus_used.clone();
            for &instruction_id in &instruction_ids {
                if is_droppable(&self.dfg[instruction_id]) {
                    continue;
                }
                for &result in self.dfg.instruction_results(instruction_id) {
                    if self.dfg.type_of_value(result).contains_reference() {
                        used.insert(result);
                    }
                }
            }

            let mut kept_rev: Vec<InstructionId> = Vec::new();
            for instruction_id in instruction_ids.into_iter().rev() {
                let instruction = &self.dfg[instruction_id];

                let droppable = is_droppable(instruction);

                let store_addr_used = if let Instruction::Store { address, .. } = instruction {
                    used.contains(address)
                } else {
                    false
                };

                let has_used_result = self
                    .dfg
                    .instruction_results(instruction_id)
                    .iter()
                    .any(|r| used.contains(r));

                if !droppable || has_used_result || store_addr_used {
                    instruction.for_each_value(|v| {
                        used.insert(v);
                    });
                    kept_rev.push(instruction_id);
                }
            }

            kept_rev.reverse();
            *self.dfg[block_id].instructions_mut() = kept_rev;
        }
    }
}

/// Instructions whose only effect is to produce a reference, an array
/// value, or RC bookkeeping. These are the only candidates for removal in
/// an `Unreachable`-terminated block — everything else is retained so the
/// set of constraints emitted to ACIR is unchanged.
fn is_droppable(instruction: &Instruction) -> bool {
    matches!(
        instruction,
        Instruction::Allocate
            | Instruction::Store { .. }
            | Instruction::MakeArray { .. }
            | Instruction::IncrementRc { .. }
            | Instruction::DecrementRc { .. }
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::assert_ssa_does_not_change, ssa_gen::Ssa},
    };

    /// Regression: dead `Allocate`/`Store` pairs preceding an always-failing
    /// `constrain` used to survive every SSA pass and reach ACIR codegen,
    /// where `Instruction::Allocate` raises `RuntimeError::UnknownReference`.
    /// Reproduced via the AST fuzzer smoke test with seed
    /// `0x95b8eab400100000`; the minimal triggering Noir source is roughly:
    ///
    /// ```text
    /// fn main() -> pub Field {
    ///     let mut q: &mut [&mut &mut u16; 1] = &mut [&mut &mut 1_u16];
    ///     let r: [Field] = &[];
    ///     let i: u32 = 1;
    ///     r[i]
    /// }
    /// ```
    #[test]
    fn removes_dead_allocate_before_unreachable_terminator() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = allocate -> &mut u16
            store u16 1 at v0
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_dead_refs_in_unreachable_blocks();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#);
    }

    #[test]
    fn removes_unused_make_array_before_unreachable_terminator() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [] : [&mut u1; 0]
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_dead_refs_in_unreachable_blocks();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#);
    }

    #[test]
    fn removes_nested_allocate_store_chain_before_unreachable_terminator() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = allocate -> &mut u8
            store u8 0 at v0
            v2 = make_array [u8 0, v0] : [(u8, &mut u8); 1]
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_dead_refs_in_unreachable_blocks();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#);
    }

    #[test]
    fn preserves_constraints_in_unreachable_block() {
        // This pass must keep every `Constrain` so the set and order of
        // constraints emitted to ACIR is unchanged.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            constrain v0 == u32 5, "first check"
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::remove_dead_refs_in_unreachable_blocks);
    }

    #[test]
    fn leaves_reachable_blocks_untouched() {
        // Blocks that terminate normally must not be touched.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = allocate -> &mut u8
            store u8 7 at v0
            v1 = load v0 -> u8
            return v1
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::remove_dead_refs_in_unreachable_blocks);
    }

    #[test]
    #[should_panic(expected = "encountered `IfElse` in a block whose terminator is `Unreachable`")]
    fn rejects_if_else_in_unreachable_block() {
        // `Remove IfElse` runs before this pass; an `IfElse` reaching here
        // would mean the pipeline ordering invariant was violated, so we
        // ICE rather than silently mishandle aliasing through the merged
        // result.
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: [Field; 1], v2: [Field; 1]):
            v3 = not v0
            v4 = if v0 then v1 else (if v3) v2
            constrain u1 0 == u1 1, \"Index out of bounds\"
            unreachable
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let _ = ssa.remove_dead_refs_in_unreachable_blocks();
    }

    #[test]
    fn preserves_store_through_call_returned_reference() {
        // A `Call` returning a `&mut T` produces a reference that can alias
        // caller-visible memory. A subsequent `Store` through that returned
        // reference must survive the backward sweep so the aliasing effect
        // is not silently dropped.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = call f1() -> &mut u8
            store u8 1 at v0
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        acir(inline) predicate_pure fn helper f1 {
          b0():
            v0 = allocate -> &mut u8
            store u8 0 at v0
            return v0
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::remove_dead_refs_in_unreachable_blocks);
    }

    #[test]
    fn preserves_store_through_array_get_returned_reference() {
        // An `ArrayGet` whose result is `&mut T` aliases the reference
        // stored into the source array. The `Store` through that returned
        // reference must be kept; consequently the whole producing chain
        // (allocate / store / make_array / array_get) is kept too.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = allocate -> &mut u8
            store u8 0 at v0
            v2 = make_array [v0] : [&mut u8; 1]
            v3 = array_get v2, index u32 0 -> &mut u8
            store u8 1 at v3
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::remove_dead_refs_in_unreachable_blocks);
    }

    #[test]
    fn preserves_store_through_array_set_returned_array_of_references() {
        // An `ArraySet` on an array of references produces a new array
        // that still holds aliasing references. The new array's elements
        // are seeded as live; the chain producing them must survive.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = allocate -> &mut u8
            store u8 0 at v0
            v2 = allocate -> &mut u8
            store u8 0 at v2
            v4 = make_array [v0, v2] : [&mut u8; 2]
            v5 = array_set v4, index u32 0, value v2
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::remove_dead_refs_in_unreachable_blocks);
    }

    #[test]
    fn preserves_store_through_load_returned_reference() {
        // `load v_outer -> &mut u8` hands out a reference that aliases
        // whatever was last stored to `v_outer`. The subsequent `Store`
        // through that loaded reference must survive — together with the
        // outer/inner allocate+store chain that produced it.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = allocate -> &mut u8
            store u8 0 at v0
            v1 = allocate -> &mut &mut u8
            store v0 at v1
            v2 = load v1 -> &mut u8
            store u8 1 at v2
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::remove_dead_refs_in_unreachable_blocks);
    }

    #[test]
    fn accepts_call_with_non_reference_result_in_unreachable_block() {
        // Sanity check: a `Call` returning a non-reference value flows
        // through the pass unchanged.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = call f1() -> u8
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        acir(inline) predicate_pure fn helper f1 {
          b0():
            return u8 0
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::remove_dead_refs_in_unreachable_blocks);
    }

    #[test]
    fn accepts_array_and_load_ops_with_non_reference_results_in_unreachable_block() {
        // Sanity check: `ArrayGet`/`ArraySet`/`Load` returning plain
        // numeric values flow through the pass unchanged.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = allocate -> &mut u8
            store u8 7 at v0
            v1 = load v0 -> u8
            v2 = make_array [u8 1, u8 2] : [u8; 2]
            v3 = array_get v2, index u32 0 -> u8
            v4 = array_set v2, index u32 0, value u8 9
            constrain v1 == u8 7, "loaded value"
            constrain u1 0 == u1 1, "Index out of bounds"
            unreachable
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::remove_dead_refs_in_unreachable_blocks);
    }
}
