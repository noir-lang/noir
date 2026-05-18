//! Verifies the implicit invariant that Brillig SSA must satisfy around
//! `array_set` and reference counts.
//!
//! # The invariant
//!
//! In Brillig, `array_set vX, i, x` may modify `vX`'s storage in place at runtime
//! when `vX`'s reference count is 1. SSA-level semantics still says `vX` is unchanged
//! and the array_set produces a fresh value; the in-place mutation is a runtime
//! optimization that's only sound when no later use can observe `vX`'s pre-mutation
//! contents through aliasing.
//!
//! Two mechanisms keep the optimization invisible to SSA semantics:
//!
//! 1. **`inc_rc`** before the `array_set` forces RC ≥ 2 at runtime so `array_set`
//!    copies rather than mutating in place.
//! 2. **Block-parameter threading** routes the post-mutation value forward as a new
//!    SSA value (the `array_set`'s result), so no later instruction references
//!    `vX` after the mutation.
//!
//! The frontend uses whichever mechanism the program needs. This pass verifies
//! that one of them is in place for every `array_set` whose source has an
//! aliased use reachable forward in the CFG.
//!
//! # Precondition
//!
//! Must be run **after `mem2reg_brillig`**. The alias-root walk through
//! block-parameter edges only reflects post-mem2reg aliasing; pre-mem2reg, an
//! `Allocate`/`Store`/`Load` chain would route aliasing through references that
//! this pass does not track.
//!
//! # Known alias-tracking gaps
//!
//! The verifier walks aliasing *only* through block-parameter edges. Aliasing
//! introduced via `MakeArray` of nested arrays, `IfElse` on arrays, non-inlined
//! `Call` returns, or `Store`/`Load` on ineligible (nested-ref) allocates is
//! **not** tracked. This is intentional for the first cut: the verifier asserts
//! the common invariant the frontend produces after mem2reg, not a universal
//! safety property.

use rustc_hash::FxHashSet as HashSet;

use crate::{
    errors::RtResult,
    ssa::{
        ir::{
            cfg::ControlFlowGraph,
            function::Function,
            instruction::{Instruction, TerminatorInstruction},
            value::{Value, ValueId},
        },
        ssa_gen::Ssa,
    },
};

impl Ssa {
    /// Verifies the `array_set` / `inc_rc` aliasing invariant on every Brillig
    /// function. See the module-level docs for details.
    ///
    /// Compiled to a no-op in release builds — this is a sanity check on
    /// frontend codegen, not a hot-path safety net.
    pub(crate) fn verify_array_set_rc_invariant(self) -> RtResult<Ssa> {
        #[cfg(debug_assertions)]
        for function in self.functions.values() {
            function.verify_array_set_rc_invariant()?;
        }
        Ok(self)
    }
}

impl Function {
    #[cfg(debug_assertions)]
    pub(crate) fn verify_array_set_rc_invariant(&self) -> RtResult<()> {
        if !self.runtime().is_brillig() {
            return Ok(());
        }

        let cfg = ControlFlowGraph::with_function(self);

        for block_id in self.reachable_blocks() {
            for instruction_id in self.dfg[block_id].instructions() {
                let Instruction::ArraySet { array, .. } = self.dfg[*instruction_id] else {
                    continue;
                };
                let [result] = self.dfg.instruction_result(*instruction_id);
                // Compute alias-roots; remaining checks land in subsequent steps.
                let _alias_set = compute_alias_set(self, &cfg, array, result);
            }
        }
        Ok(())
    }
}

/// Compute the set of value-IDs that alias the storage of `source` through
/// block-parameter threading.
///
/// Walks backward through block-parameter edges: every time the walk reaches
/// a [`Value::Param`], it follows each predecessor's `Jmp` / `JmpIf` to the
/// argument at the parameter's position, adding that argument to the set.
///
/// `exclude` is pre-seeded as visited so the walk stops at the `array_set`'s
/// own result. Without this, a back-edge that threads the `array_set`'s result
/// back into the source parameter would pull the result into the alias set —
/// but the result *is* the post-mutation value, so uses of it through that
/// back-edge are intentional, not a hazard.
///
/// The walk deliberately does **not** follow `array_set` chains within a block.
/// Each link of a chain `v1 = array_set v0 ; v2 = array_set v1 ; …` is verified
/// independently — including an upstream link's source in a downstream link's
/// alias-set would let an `inc_rc` that protects one link falsely appear to
/// protect another. For example:
///
/// ```text
/// inc_rc v0
/// v1 = array_set v0          // copies (RC>1); v1 is in fresh storage
/// v2 = array_set v1          // mutates v1 in place (RC==1)
/// v3 = array_set v2          // mutates v1 in place again
/// array_get v1               // ← hazard: reads post-v3 contents
/// ```
///
/// The hazard is rooted at `v2 = array_set v1`'s missing `inc_rc v1`. Walking
/// the chain would put `{v2, v1, v0}` in `v3`'s alias-set, find that
/// `inc_rc v0` dominates `v3`, and accept the SSA. Restricting the walk to
/// block-parameter edges keeps the alias-set for each link tied to its
/// immediate source, so `v2`'s check (alias-set `{v1}`) correctly demands
/// `inc_rc v1` and flags the violation.
fn compute_alias_set(
    function: &Function,
    cfg: &ControlFlowGraph,
    source: ValueId,
    exclude: ValueId,
) -> im::HashSet<ValueId> {
    let mut set = im::HashSet::new();
    let mut visited: HashSet<ValueId> = HashSet::default();
    let mut worklist: Vec<ValueId> = Vec::new();

    visited.insert(exclude);

    if visited.insert(source) {
        set.insert(source);
        worklist.push(source);
    }

    while let Some(v) = worklist.pop() {
        let Value::Param { block, position, .. } = function.dfg[v] else {
            continue;
        };

        for pred in cfg.predecessors(block) {
            let term = function.dfg[pred]
                .terminator()
                .expect("reachable predecessor block must have a terminator");
            match term {
                TerminatorInstruction::Jmp { destination, arguments, .. }
                    if *destination == block =>
                {
                    push_if_new(
                        arguments.get(position).copied(),
                        &mut set,
                        &mut visited,
                        &mut worklist,
                    );
                }
                TerminatorInstruction::JmpIf {
                    then_destination,
                    then_arguments,
                    else_destination,
                    else_arguments,
                    ..
                } => {
                    // A `jmpif` may target the same block from both arms; walk
                    // each argument list whose destination matches.
                    if *then_destination == block {
                        push_if_new(
                            then_arguments.get(position).copied(),
                            &mut set,
                            &mut visited,
                            &mut worklist,
                        );
                    }
                    if *else_destination == block {
                        push_if_new(
                            else_arguments.get(position).copied(),
                            &mut set,
                            &mut visited,
                            &mut worklist,
                        );
                    }
                }
                _ => (),
            }
        }
    }

    set
}

fn push_if_new(
    arg: Option<ValueId>,
    set: &mut im::HashSet<ValueId>,
    visited: &mut HashSet<ValueId>,
    worklist: &mut Vec<ValueId>,
) {
    if let Some(arg) = arg
        && visited.insert(arg)
    {
        set.insert(arg);
        worklist.push(arg);
    }
}

#[cfg(test)]
mod tests {
    use super::{ControlFlowGraph, compute_alias_set};
    use crate::ssa::{
        ir::{function::Function, instruction::Instruction, value::ValueId},
        ssa_gen::Ssa,
    };

    /// ACIR functions are skipped: `inc_rc` / `dec_rc` are no-ops in ACIR and
    /// `array_set` always produces a fresh array.
    #[test]
    fn acir_function_is_skipped() {
        let src = r#"
            acir(inline) fn main f0 {
              b0(v0: [u32; 2]):
                v3 = array_set v0, index u32 0, value u32 99
                v5 = array_get v0, index u32 0 -> u32
                return v5
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        assert!(ssa.verify_array_set_rc_invariant().is_ok());
    }

    /// `compute_alias_set` deliberately does **not** walk through `array_set`
    /// chains — only through block-parameter edges. This test establishes the
    /// design contract.
    ///
    /// In the program below, `inc_rc v0` forces `v2 = array_set v0` to copy,
    /// so `v2` is in fresh storage. The chain `v4 = array_set v2 ; v6 =
    /// array_set v4` then mutates `v2`'s storage in place at each step.
    /// A use of `v4` after `v6` (the `array_get v4` below) is a real hazard
    /// because no `inc_rc v4` protects it.
    ///
    /// If `compute_alias_set` walked the chain, `v6`'s alias-set would include
    /// `v0`, and the dominance check would falsely accept `inc_rc v0` as
    /// protection — silently missing the hazard. By restricting the walk to
    /// block-parameter edges, each link of the chain is checked against its
    /// own immediate source: `v6`'s alias-set is just `{v4}`, and the absence
    /// of `inc_rc v4` correctly surfaces the violation.
    #[test]
    fn alias_set_does_not_walk_array_set_chains() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2]):
                inc_rc v0
                v2 = array_set v0, index u32 0, value u32 1
                v4 = array_set v2, index u32 1, value u32 2
                v6 = array_set v4, index u32 0, value u32 3
                v8 = array_get v4, index u32 0 -> u32
                return v8
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let cfg = ControlFlowGraph::with_function(function);

        let (source, result) = last_array_set(function).expect("array_set present");

        let alias_set = compute_alias_set(function, &cfg, source, result);

        // Only the source itself — no walking into the upstream chain links
        // (v2, v0) or any block-parameter predecessors.
        assert_eq!(alias_set.iter().copied().collect::<Vec<_>>(), vec![source]);
    }

    /// Two things at once:
    ///   1. The walk follows block-parameter edges — for the `array_set` in `b5`,
    ///      `compute_alias_set` resolves `v2`'s pre-header source (`v0`) and
    ///      includes it in the alias set.
    ///   2. It terminates on cycles. `b1`'s parameter `v2` has two inbound jumps,
    ///      including the back-edge from `b5` that carries the `array_set`'s own
    ///      result. The exclude-pre-seed both keeps the result out of the set
    ///      *and* prevents the walk from re-entering the cycle through it.
    ///
    /// This is the well-formed program from the design discussion: the loop reads
    /// `v2` each iteration, mutates `v2` in place via the `array_set`, and threads
    /// the result back through the block-parameter. No `inc_rc` is needed because
    /// block-parameter threading already keeps the post-mutation value visible to
    /// the next iteration.
    #[test]
    fn alias_set_follows_block_params_and_terminates_on_cycles() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2], v1: u32):
                jmp b1(v0, u32 0)
              b1(v2: [u32; 2], v3: u32):
                v5 = lt v3, v1
                jmpif v5 then: b2(), else: b3()
              b2():
                v6 = array_get v2, index u32 0 -> u32
                v8 = eq v3, u32 1
                jmpif v8 then: b4(), else: b5()
              b3():
                return
              b4():
                v10 = eq v6, u32 99
                constrain v6 == u32 99
                jmp b5()
              b5():
                v11 = array_set v2, index u32 0, value u32 99
                v12 = add v3, u32 1
                jmp b1(v11, v12)
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let cfg = ControlFlowGraph::with_function(function);

        let (source, result) = last_array_set(function).expect("array_set present");

        let alias_set = compute_alias_set(function, &cfg, source, result);

        // Expect `{v2, v0}`: the source itself plus the function's array
        // parameter that flows into `v2` via the pre-header jmp from `b0`.
        // The back-edge's argument is the array_set's own result and is
        // excluded by construction.
        let entry_v0 = function.dfg.block_parameters(function.entry_block())[0];
        assert_eq!(alias_set.len(), 2);
        assert!(alias_set.contains(&source));
        assert!(alias_set.contains(&entry_v0));
        assert!(!alias_set.contains(&result));
    }

    fn last_array_set(function: &Function) -> Option<(ValueId, ValueId)> {
        let mut found = None;
        for block_id in function.reachable_blocks() {
            for inst_id in function.dfg[block_id].instructions() {
                if let Instruction::ArraySet { array, .. } = function.dfg[*inst_id] {
                    let [result] = function.dfg.instruction_result(*inst_id);
                    found = Some((array, result));
                }
            }
        }
        found
    }
}
