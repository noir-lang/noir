//! Block-local DCE that runs immediately after [`Ssa::remove_unreachable_instructions`].
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
//! ## Aliasing / fail-secure invariants
//!
//! `flatten_cfg`, `Remove IfElse`, and the major inlining passes have all
//! run before [`Ssa::remove_unreachable_instructions`] (and therefore
//! before this pass). The combinations below would invalidate the
//! observability argument above, so they are treated as ICEs rather than
//! silently mishandled:
//!
//! - `Instruction::IfElse` — should have been lowered away.
//! - `Instruction::Call` whose return type contains a reference — would
//!   surface a value that can alias caller-visible memory, which this
//!   block-local DCE has no way to reason about.
//!
//! Both conditions are checked before any removal happens; the pass panics
//! at the first offender.
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

            for &instruction_id in &instruction_ids {
                self.assert_unreachable_block_invariants(instruction_id);
            }

            let mut used: HashSet<ValueId> = databus_used.clone();
            let mut kept_rev: Vec<InstructionId> = Vec::new();

            for instruction_id in instruction_ids.into_iter().rev() {
                let instruction = &self.dfg[instruction_id];

                let droppable = matches!(
                    instruction,
                    Instruction::Allocate
                        | Instruction::Store { .. }
                        | Instruction::MakeArray { .. }
                        | Instruction::IncrementRc { .. }
                        | Instruction::DecrementRc { .. }
                );

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

    fn assert_unreachable_block_invariants(&self, instruction_id: InstructionId) {
        let instruction = &self.dfg[instruction_id];
        match instruction {
            Instruction::IfElse { .. } => {
                panic!(
                    "remove_dead_refs_in_unreachable_blocks: encountered `IfElse` in a block \
                     whose terminator is `Unreachable`. The `Remove IfElse` pass is expected to \
                     run before this one; please ensure the SSA pipeline order is preserved."
                )
            }
            Instruction::Call { .. } => {
                let results = self.dfg.instruction_results(instruction_id);
                for &result in results {
                    if self.dfg.type_of_value(result).contains_reference() {
                        panic!(
                            "remove_dead_refs_in_unreachable_blocks: encountered `Call` returning \
                             a reference-typed result in an `Unreachable` block. This pass \
                             cannot reason about aliasing through caller-visible memory \
                             surfaced by such a call."
                        );
                    }
                }
            }
            _ => {}
        }
    }
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
        // The block-local DCE must keep every `Constrain` so the set and
        // order of constraints emitted to ACIR is unchanged.
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
    #[should_panic(
        expected = "encountered `Call` returning a reference-typed result in an `Unreachable` block"
    )]
    fn rejects_call_returning_reference_in_unreachable_block() {
        // A `Call` returning a `&mut T` would surface aliasing this pass
        // cannot reason about; treat it as an ICE.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = call f1() -> &mut u8
            store u8 0 at v0
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
        let ssa = Ssa::from_str(src).unwrap();
        let _ = ssa.remove_dead_refs_in_unreachable_blocks();
    }

    #[test]
    fn accepts_call_with_non_reference_result_in_unreachable_block() {
        // The fail-secure guard must not fire for non-reference call
        // results.
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
}
