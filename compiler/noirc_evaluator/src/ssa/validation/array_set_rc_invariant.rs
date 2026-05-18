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

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use std::cmp::Ordering;

use crate::{
    errors::{RtResult, RuntimeError},
    ssa::{
        ir::{
            basic_block::BasicBlockId,
            cfg::ControlFlowGraph,
            dom::DominatorTree,
            function::Function,
            instruction::{Instruction, InstructionId, TerminatorInstruction},
            post_order::PostOrder,
            union_find::UnionFind,
            value::ValueId,
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

        let ctx = Context::new(self);

        for block_id in self.reachable_blocks() {
            for (idx, instruction_id) in self.dfg[block_id].instructions().iter().enumerate() {
                let Instruction::ArraySet { array, .. } = self.dfg[*instruction_id] else {
                    continue;
                };

                let alias_set = ctx.alias_set_for(array);

                // Cheap check first: if any `inc_rc` on an alias-set member
                // precedes this `array_set` in flow order, treat the
                // aliasing as already protected. See `some_inc_rc_precedes`
                // for the rationale (relaxed from dominance to RPO
                // precedence).
                if ctx.some_inc_rc_precedes(&alias_set, block_id, idx) {
                    continue;
                }

                // Expensive: forward CFG walk looking for an aliased read.
                // A hit means the `array_set` may mutate storage in place
                // (RC=1) and a downstream instruction will observe the
                // pre-mutation contents through an aliased name.
                if ctx.has_reachable_aliased_use(&alias_set, *instruction_id, block_id, idx) {
                    let call_stack = self.dfg.get_instruction_call_stack(*instruction_id);
                    let message = format!(
                        "array_set in function {} on array {array} has an aliased read on a \
                         forward path but no `inc_rc` dominates it; the in-place mutation \
                         would be observable through an alias",
                        self.name(),
                    );
                    return Err(RuntimeError::SsaValidationError { message, call_stack });
                }
            }
        }
        Ok(())
    }
}

/// Pre-computed indices over a Brillig function, built in a single pass over
/// its reachable blocks (plus one finalization pass for the union-find
/// equivalence classes). The verifier's per-array_set checks read from these
/// structures rather than re-scanning the function.
struct Context<'f> {
    function: &'f Function,
    /// Dominator tree. Mutable because `dominates` populates an internal cache.
    dom_tree: DominatorTree,
    /// For each value that participates in any alias-equivalence union, its
    /// full equivalence class. Values absent from this map have an implicit
    /// singleton class `{v}` — handled by [`Context::alias_set_for`].
    value_aliases: HashMap<ValueId, im::HashSet<ValueId>>,
    /// For each array-typed value defined by an instruction, the block where
    /// the defining instruction lives. Used by the kill-on-re-entry rule
    /// inside [`Context::succ_use_set`].
    array_value_defs: HashMap<ValueId, BasicBlockId>,
    /// Every value that is the result of an `array_set` instruction.
    /// Filtered out of every alias-set (except when the value is the
    /// `array_set`'s own source) — see [`Context::alias_set_for`].
    array_set_results: HashSet<ValueId>,
    /// `inc_rc value` instructions indexed by their operand. Each entry is
    /// the `(block, instruction-position-within-block)` of one `inc_rc`.
    inc_rc_locations: HashMap<ValueId, Vec<(BasicBlockId, usize)>>,
    /// Per-block sorted list of `(idx, instruction-id, array-operand)`
    /// triples — one entry per array-typed operand of each non-terminator
    /// instruction. Lets the reachable-use walk skip over instructions that
    /// have no array operand.
    array_operand_uses: HashMap<BasicBlockId, Vec<ArrayOperandUse>>,
}

impl<'f> Context<'f> {
    fn new(function: &'f Function) -> Self {
        let cfg = ControlFlowGraph::with_function(function);
        let post_order = PostOrder::with_cfg(&cfg);
        let dom_tree = DominatorTree::with_cfg_and_post_order(&cfg, &post_order);

        let mut inc_rc_locations: HashMap<ValueId, Vec<(BasicBlockId, usize)>> = HashMap::default();
        let mut array_operand_uses: HashMap<BasicBlockId, Vec<ArrayOperandUse>> =
            HashMap::default();
        let mut array_value_defs: HashMap<ValueId, BasicBlockId> = HashMap::default();
        let mut array_set_results: HashSet<ValueId> = HashSet::default();
        let mut uf: UnionFind<ValueId> = UnionFind::new();

        // Single pass over every reachable block. Each iteration populates
        // all per-instruction indices plus the union-find from the
        // terminator's (param, arg) pairs.
        for block_id in function.reachable_blocks() {
            let mut operand_uses: Vec<ArrayOperandUse> = Vec::new();
            for (idx, instruction_id) in function.dfg[block_id].instructions().iter().enumerate() {
                let instruction = &function.dfg[*instruction_id];

                if let Instruction::IncrementRc { value } = instruction {
                    inc_rc_locations.entry(*value).or_default().push((block_id, idx));
                }

                let is_array_set = matches!(instruction, Instruction::ArraySet { .. });
                for &result in function.dfg.instruction_results(*instruction_id) {
                    if function.dfg.type_of_value(result).contains_an_array() {
                        array_value_defs.insert(result, block_id);
                        if is_array_set {
                            array_set_results.insert(result);
                        }
                    }
                }

                instruction.for_each_value(|v| {
                    if function.dfg.type_of_value(v).contains_an_array() {
                        operand_uses.push((idx, *instruction_id, v));
                    }
                });
            }
            if !operand_uses.is_empty() {
                array_operand_uses.insert(block_id, operand_uses);
            }

            if let Some(terminator) = function.dfg[block_id].terminator() {
                match terminator {
                    TerminatorInstruction::Jmp { destination, arguments, .. } => {
                        union_param_args(function, &mut uf, *destination, arguments);
                    }
                    TerminatorInstruction::JmpIf {
                        then_destination,
                        then_arguments,
                        else_destination,
                        else_arguments,
                        ..
                    } => {
                        union_param_args(function, &mut uf, *then_destination, then_arguments);
                        union_param_args(function, &mut uf, *else_destination, else_arguments);
                    }
                    _ => (),
                }
            }
        }

        let value_aliases = materialize_value_aliases(uf);

        Self {
            function,
            dom_tree,
            value_aliases,
            array_value_defs,
            array_set_results,
            inc_rc_locations,
            array_operand_uses,
        }
    }

    /// Look up `source`'s alias-equivalence class and filter out every
    /// `array_set` result *except* the source itself.
    ///
    /// Every `array_set` result represents a post-mutation value — uses
    /// of it (or of any name it gets re-bound to through block-parameter
    /// threading) are intentional reads of the mutated storage, not
    /// hazards. Keeping them in the alias-set would defeat the per-arg
    /// kill rule in [`Context::succ_use_set`]: a back-edge
    /// `jmp b(v_arrset)` would see `v_arrset` in the use-set and refuse
    /// to kill the receiving param, letting the alias leak past the loop.
    ///
    /// The source itself is kept even when it happens to be an
    /// `array_set` result (e.g. a chain
    /// `v_a = array_set _ ; v_b = array_set v_a`): `v_a` is *this*
    /// check's source, so its forward uses are exactly what we want to
    /// look for.
    fn alias_set_for(&self, source: ValueId) -> im::HashSet<ValueId> {
        let class =
            self.value_aliases.get(&source).cloned().unwrap_or_else(|| im::HashSet::unit(source));
        class.into_iter().filter(|&v| v == source || !self.array_set_results.contains(&v)).collect()
    }

    /// Returns `true` if some `inc_rc r` for an `r ∈ alias_set` exists at a
    /// program point that precedes the `array_set` in flow order — either
    /// in a strictly-earlier position within the same block, or in a
    /// different block whose reverse-post-order index is smaller.
    ///
    /// # Why RPO precedence, not dominance
    ///
    /// The strict invariant would be "the `inc_rc` dominates the
    /// `array_set`" — i.e., is on *every* path. But the frontend in
    /// practice emits **path-specific** `inc_rc`s: each path that creates
    /// an alias gets its own `inc_rc`, with no single dominating one. See
    /// e.g. `brillig_array_ifelse` where `b1` has `inc_rc v8` and `b4` has
    /// `inc_rc v2`, neither dominating the `array_set` in `b6`. On every
    /// runtime path the alias is either covered by an `inc_rc` or the
    /// values are freshly allocated, but the verifier can't observe
    /// path-specific freshness with the current alias-set model.
    ///
    /// We weaken the check accordingly: presence of *any* `inc_rc` on an
    /// alias-set member, anywhere earlier in flow, is taken as the
    /// frontend signalling "I'm aware of this aliasing and have handled
    /// it." Absence of any such `inc_rc` *combined* with a forward aliased
    /// read (checked separately) still flags as a hazard, which is the
    /// shape PR-12671 had.
    ///
    /// **Known weakness:** this accepts SSAs where an `inc_rc` is placed
    /// on a different branch than the `array_set` (e.g.
    /// `b0: jmpif then b1{inc_rc v0} else b2{v_ = array_set v0}`). The
    /// frontend doesn't appear to produce that shape in practice; a
    /// fuzzer might.
    ///
    /// Well-formed SSA contains no `DecrementRc`, so we don't need to
    /// worry about a `dec_rc` intervening between the `inc_rc` and the
    /// `array_set`.
    fn some_inc_rc_precedes(
        &self,
        alias_set: &im::HashSet<ValueId>,
        array_set_block: BasicBlockId,
        array_set_idx: usize,
    ) -> bool {
        for value in alias_set {
            let Some(locations) = self.inc_rc_locations.get(value) else {
                continue;
            };
            for &(inc_block, inc_idx) in locations {
                if inc_block == array_set_block {
                    if inc_idx < array_set_idx {
                        return true;
                    }
                } else if self.dom_tree.reverse_post_order_cmp(inc_block, array_set_block)
                    == Ordering::Less
                {
                    return true;
                }
            }
        }
        false
    }

    /// Forward CFG walk from after the `array_set` looking for a
    /// non-terminator instruction that reads a value still in the alias
    /// use-set.
    ///
    /// **Use-set evolution.** Starts as `alias_set` and only shrinks. Kills
    /// are applied **during propagation** to each successor — see
    /// [`Context::succ_use_set`] for the kill rules.
    ///
    /// **What counts as a use.** Only non-terminator operands. Terminator
    /// arguments are the legitimate threading mechanism the invariant
    /// relies on; `jmp b(v_alias)` is how the post-mutation value reaches
    /// the next block where it is re-bound to that block's parameter. The
    /// kill logic already accounts for these args.
    ///
    /// The original `array_set` itself is also skipped, in case a cycle
    /// re-enters its block — it is, by construction, a use of its own
    /// source, not a hazard.
    ///
    /// **Cycle detection.** Re-visiting a block with a use-set that is a
    /// subset of what we already explored at that block adds no new
    /// information. We record the *union* of use-sets seen per block and
    /// skip on subset matches.
    fn has_reachable_aliased_use(
        &self,
        alias_set: &im::HashSet<ValueId>,
        array_set_id: InstructionId,
        array_set_block: BasicBlockId,
        array_set_idx: usize,
    ) -> bool {
        let mut visited: HashMap<BasicBlockId, im::HashSet<ValueId>> = HashMap::default();

        // (block, start_idx, use_set_on_entry)
        //
        // `start_idx > 0` denotes the very first frame, which continues
        // inside the array_set's own block past the array_set instruction
        // itself.
        let mut worklist: Vec<(BasicBlockId, usize, im::HashSet<ValueId>)> =
            vec![(array_set_block, array_set_idx + 1, alias_set.clone())];

        while let Some((block, start_idx, use_set)) = worklist.pop() {
            if use_set.is_empty() {
                continue;
            }

            // Cycle/redundancy check + bookkeeping only applies to *full*
            // block entries (start_idx == 0). The very first frame of the
            // walk starts mid-block and only covers a suffix of the block —
            // recording it here would incorrectly let a later back-edge
            // entry to the same block skip the unexplored prefix.
            if start_idx == 0 {
                if let Some(prev) = visited.get(&block)
                    && use_set.is_subset(prev)
                {
                    continue;
                }
                let merged =
                    visited.get(&block).cloned().unwrap_or_default().union(use_set.clone());
                visited.insert(block, merged);
            }

            // Iterate only non-terminator instructions that have an
            // array-typed operand. Entries are pre-sorted by `idx`.
            if let Some(uses) = self.array_operand_uses.get(&block) {
                for &(_idx, inst_id, operand) in
                    uses.iter().skip_while(|(idx, _, _)| *idx < start_idx)
                {
                    if inst_id == array_set_id {
                        continue;
                    }
                    if use_set.contains(&operand) {
                        return true;
                    }
                }
            }

            let Some(terminator) = self.function.dfg[block].terminator() else { continue };
            match terminator {
                TerminatorInstruction::Jmp { destination, arguments, .. } => {
                    let next = self.succ_use_set(*destination, arguments, &use_set);
                    worklist.push((*destination, 0, next));
                }
                TerminatorInstruction::JmpIf {
                    then_destination,
                    then_arguments,
                    else_destination,
                    else_arguments,
                    ..
                } => {
                    let then_next = self.succ_use_set(*then_destination, then_arguments, &use_set);
                    worklist.push((*then_destination, 0, then_next));
                    let else_next = self.succ_use_set(*else_destination, else_arguments, &use_set);
                    worklist.push((*else_destination, 0, else_next));
                }
                TerminatorInstruction::Return { .. }
                | TerminatorInstruction::Unreachable { .. } => (),
            }
        }
        false
    }

    /// Compute the use-set carried into `dest` when its predecessor jumps
    /// with `arguments`.
    ///
    /// Two kill rules apply, in order:
    ///
    /// 1. **Conditional kill — block parameters of `dest`.** For each
    ///    `dest.params[i]` that is currently in `use_set`, look at the
    ///    corresponding `arguments[i]`:
    ///    - If the arg is *also* in `use_set`, the parameter is rebound to
    ///      a value that still aliases the array_set's source; keep it.
    ///    - Otherwise (arg is a fresh value — most commonly the
    ///      array_set's own result, excluded at lookup time), the parameter
    ///      is rebound to a non-aliased value; drop it.
    ///
    /// 2. **Unconditional kill — instructions defined in `dest`.** For
    ///    each alias-set member whose defining block is `dest`: drop it.
    ///    Re-entering `dest` re-executes the defining instruction (e.g.
    ///    `load`, `make_array`), producing a fresh runtime value; the
    ///    previous iteration's storage is no longer reachable through that
    ///    name.
    fn succ_use_set(
        &self,
        dest: BasicBlockId,
        arguments: &[ValueId],
        use_set: &im::HashSet<ValueId>,
    ) -> im::HashSet<ValueId> {
        let mut result = use_set.clone();

        // (1) Per-arg conditional kill for params of `dest`.
        let params = self.function.dfg.block_parameters(dest);
        for (i, &param) in params.iter().enumerate() {
            if !use_set.contains(&param) {
                continue;
            }
            let arg_is_alias = arguments.get(i).is_some_and(|arg| use_set.contains(arg));
            if !arg_is_alias {
                result.remove(&param);
            }
        }

        // (2) Unconditional kill for instruction-defined values whose
        // def-block is `dest` (re-execution on cycle re-entry produces a
        // fresh value).
        for &v in use_set {
            if self.array_value_defs.get(&v) == Some(&dest) {
                result.remove(&v);
            }
        }

        result
    }
}

/// Materialize the union-find into a `value → equivalence-class` map.
///
/// Two values are in the same class iff they may share storage at runtime
/// through any chain of `jmp` / `jmpif` argument-to-parameter bindings.
/// Values absent from the resulting map have an implicit singleton class
/// `{v}` (they didn't participate in any union).
///
/// # Why we deliberately do **not** unify `array_set`'s source and result
///
/// At runtime the result may or may not alias the source depending on RC.
/// The verifier's job is to flag the cases where it *might*. We model this
/// by *excluding* the result from the alias-set at lookup time (in
/// [`Context::alias_set_for`]) rather than unifying it into the
/// equivalence class. Otherwise a chain
/// `v1 = array_set v0 ; v2 = array_set v1 ; v3 = array_set v2` would put
/// `{v0, v1, v2, v3}` in `v3`'s class, and an `inc_rc v0` that
/// legitimately protects only `v1`'s array_set would falsely appear to
/// protect `v3` as well. See `alias_set_does_not_walk_array_set_chains`
/// for a worked example.
fn materialize_value_aliases(mut uf: UnionFind<ValueId>) -> HashMap<ValueId, im::HashSet<ValueId>> {
    let keys: Vec<ValueId> = uf.keys().collect();

    let mut class_of_rep: HashMap<ValueId, im::HashSet<ValueId>> = HashMap::default();
    for &v in &keys {
        let rep = uf.find(v);
        class_of_rep.entry(rep).or_default().insert(v);
    }

    let mut result: HashMap<ValueId, im::HashSet<ValueId>> = HashMap::default();
    for v in keys {
        let rep = uf.find(v);
        if let Some(class) = class_of_rep.get(&rep) {
            result.insert(v, class.clone());
        }
    }
    result
}

/// A non-terminator instruction's reference to an array-typed value.
///
/// One entry per (instruction, array-typed operand) pair — an instruction
/// with two array operands contributes two entries. Tuples are
/// `(instruction-index-within-block, instruction-id, operand-value)`.
type ArrayOperandUse = (usize, InstructionId, ValueId);

/// For each parameter of `dest`, union it with the corresponding argument
/// from the calling terminator. Only array-typed parameters participate —
/// non-array values cannot be in an alias-set anyway, and skipping them
/// keeps the equivalence-class structure focused.
fn union_param_args(
    function: &Function,
    uf: &mut UnionFind<ValueId>,
    dest: BasicBlockId,
    arguments: &[ValueId],
) {
    let params = function.dfg.block_parameters(dest);
    for (i, &arg) in arguments.iter().enumerate() {
        let Some(&param) = params.get(i) else { continue };
        if !function.dfg.type_of_value(param).contains_an_array() {
            continue;
        }
        uf.union(param, arg);
    }
}

#[cfg(test)]
mod tests {
    use super::Context;
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

    /// End-to-end: the user's well-formed example from the design
    /// discussion. The loop mutates `v2` in place each iteration and
    /// threads the result back through the block-parameter, so no
    /// `inc_rc` is needed and the verifier must accept.
    #[test]
    fn end_to_end_well_formed_loop_is_accepted() {
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
        assert!(ssa.verify_array_set_rc_invariant().is_ok());
    }

    /// End-to-end: PR-12671 malformed repro. `array_get v0` reads the
    /// pre-header value while `array_set v2` mutates the same storage in
    /// place — verifier must reject.
    #[test]
    fn end_to_end_pr_12671_repro_is_rejected() {
        let src = r#"
            brillig(inline) impure fn main f0 {
              b0(v0: [u32; 2], v1: u32):
                jmp b1(v0, u32 0, u32 0)
              b1(v2: [u32; 2], v3: u32, v4: u32):
                v7 = lt v4, v1
                jmpif v7 then: b2(), else: b3()
              b2():
                v6 = array_get v0, index u32 0 -> u32
                v10 = add v3, v6
                v12 = array_set v2, index u32 0, value u32 99
                v14 = add v4, u32 1
                jmp b1(v12, v10, v14)
              b3():
                return v3
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        let err = ssa.verify_array_set_rc_invariant().err().expect("expected an error");
        assert!(
            matches!(err, crate::errors::RuntimeError::SsaValidationError { .. }),
            "expected SsaValidationError, got {err:?}"
        );
    }

    /// End-to-end: same PR-12671 SSA but with an `inc_rc v0` placed in the
    /// pre-header. `inc_rc v0` dominates the array_set, so the check
    /// short-circuits and the verifier must accept.
    #[test]
    fn end_to_end_pr_12671_repro_with_inc_rc_is_accepted() {
        let src = r#"
            brillig(inline) impure fn main f0 {
              b0(v0: [u32; 2], v1: u32):
                inc_rc v0
                jmp b1(v0, u32 0, u32 0)
              b1(v2: [u32; 2], v3: u32, v4: u32):
                v7 = lt v4, v1
                jmpif v7 then: b2(), else: b3()
              b2():
                v6 = array_get v0, index u32 0 -> u32
                v10 = add v3, v6
                v12 = array_set v2, index u32 0, value u32 99
                v14 = add v4, u32 1
                jmp b1(v12, v10, v14)
              b3():
                return v3
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        assert!(ssa.verify_array_set_rc_invariant().is_ok());
    }

    /// End-to-end regression for the pattern in stdlib's `compute_root`
    /// (`array_dynamic_blackbox_input`). The loop body has *two* chained
    /// `array_set`s — the back-edge threads the second one's result back
    /// into the block-parameter, and the loop exit reads the parameter
    /// directly via `array_get`.
    ///
    /// Without the array_set-results filter, the alias-set would include
    /// the second array_set's result; the per-arg kill rule at the
    /// back-edge would then see that result in the use-set and refuse to
    /// kill the parameter, letting the alias leak to the loop exit and
    /// producing a false positive on the post-loop `array_get`.
    #[test]
    fn end_to_end_loop_chained_array_sets_with_post_loop_read_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2], v1: u32):
                jmp b1(v0, u32 0)
              b1(v2: [u32; 2], v3: u32):
                v5 = lt v3, v1
                jmpif v5 then: b2(), else: b3()
              b2():
                v8 = array_set v2, index u32 0, value u32 1
                v11 = array_set v8, index u32 1, value u32 2
                v13 = add v3, u32 1
                jmp b1(v11, v13)
              b3():
                v15 = array_get v2, index u32 0 -> u32
                return v15
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        assert!(
            ssa.verify_array_set_rc_invariant().is_ok(),
            "loop exit reads `v2`, which is rebound on the back-edge to the chained array_set's result; not a hazard"
        );
    }

    /// End-to-end: `&mut [u32; 2]` parameter pattern (like stdlib's
    /// `quicksort::partition`). mem2reg can't eliminate the reference, so
    /// the loop body has `v_loaded = load v_ref` re-executed each iteration,
    /// followed by `array_set v_loaded; store`. The cycle re-enters the
    /// load's block, where the load is re-executed and produces a *fresh*
    /// runtime value — so the `array_get v_loaded` on the next iteration is
    /// not a hazard.
    ///
    /// This test verifies the re-execution kill rule: on entry to the
    /// load's defining block, the loaded value is dropped from the use-set.
    #[test]
    fn end_to_end_loop_load_array_set_store_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: &mut [u32; 2], v1: u32):
                jmp b1(u32 0)
              b1(v3: u32):
                v5 = lt v3, v1
                jmpif v5 then: b2(), else: b3()
              b2():
                v6 = load v0 -> [u32; 2]
                v8 = array_get v6, index u32 0 -> u32
                v10 = array_set v6, index u32 0, value u32 99
                store v10 at v0
                v12 = add v3, u32 1
                jmp b1(v12)
              b3():
                return
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        assert!(
            ssa.verify_array_set_rc_invariant().is_ok(),
            "load result is re-executed each iteration; the cycle's array_get is not a hazard"
        );
    }

    /// The alias-equivalence classes deliberately do **not** unify an
    /// `array_set`'s source and result. This test establishes the design
    /// contract.
    ///
    /// In the program below, `inc_rc v0` forces `v2 = array_set v0` to copy,
    /// so `v2` is in fresh storage. The chain `v4 = array_set v2 ; v6 =
    /// array_set v4` then mutates `v2`'s storage in place at each step.
    /// A use of `v4` after `v6` (the `array_get v4` below) is a real hazard
    /// because no `inc_rc v4` protects it.
    ///
    /// If we unified each `array_set`'s source and result, `v6`'s alias-set
    /// would include `v0`, and the dominance check would falsely accept
    /// `inc_rc v0` as protection — silently missing the hazard. Excluding
    /// the result at lookup time keeps the alias-set for each link tied to
    /// its immediate source: `v6`'s alias-set is just `{v4}`, and the
    /// absence of `inc_rc v4` correctly surfaces the violation.
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
        let ctx = Context::new(function);

        let (_block, _idx, _inst_id, source, alias_set) =
            last_array_set(function, &ctx).expect("array_set present");

        // Only the source itself — no walking into the upstream chain links
        // (v2, v0) or any block-parameter predecessors.
        assert_eq!(alias_set.iter().copied().collect::<Vec<_>>(), vec![source]);
    }

    /// Two things at once:
    ///   1. The alias-equivalence classes follow block-parameter edges — for
    ///      the `array_set` in `b5`, `v2`'s pre-header source (`v0`) lands
    ///      in the same class.
    ///   2. The construction terminates on cycles. `b1`'s parameter `v2` has
    ///      two inbound jumps, including the back-edge from `b5` that
    ///      carries the `array_set`'s own result. The result is excluded
    ///      from the alias-set at lookup time so cycles through it don't
    ///      leak the post-mutation value into the live aliases.
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
        let ctx = Context::new(function);

        let (_block, _idx, _inst_id, source, alias_set) =
            last_array_set(function, &ctx).expect("array_set present");

        // Expect `{v2, v0}`: the source itself plus the function's array
        // parameter that flows into `v2` via the pre-header jmp from `b0`.
        // The back-edge's argument is the array_set's own result and is
        // excluded by the array_set-result filter inside `alias_set_for`.
        let entry_v0 = function.dfg.block_parameters(function.entry_block())[0];
        assert_eq!(alias_set.len(), 2);
        assert!(alias_set.contains(&source));
        assert!(alias_set.contains(&entry_v0));
    }

    /// Five `inc_rc` placements, each isolated on its own array parameter
    /// so the inc_rcs don't accidentally cover for each other. Tests the
    /// **relaxed** precedence check (any `inc_rc` earlier in RPO suffices;
    /// it does **not** need to dominate the array_set's block):
    ///   - `v0`: same-block, inc_rc *earlier* than the array_set → **precedes**.
    ///   - `v1`: inc_rc in entry block → **precedes**.
    ///   - `v2`: inc_rc in a sibling branch (b1) → **precedes** under the
    ///     relaxed check (sibling blocks come earlier in RPO than the
    ///     common-successor block); would fail a strict dominance check.
    ///   - `v3`: same-block, inc_rc *later* than the array_set → does
    ///     **not** precede (same-block comparison still requires earlier
    ///     position).
    ///   - `v4`: no inc_rc anywhere → does **not** precede.
    #[test]
    fn inc_rc_precedence_recognizes_earlier_in_flow_positions() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2], v1: [u32; 2], v2: [u32; 2], v3: [u32; 2], v4: [u32; 2], v5: u1):
                inc_rc v1
                jmpif v5 then: b1(), else: b2()
              b1():
                inc_rc v2
                jmp b3()
              b2():
                jmp b3()
              b3():
                inc_rc v0
                v8 = array_set v0, index u32 10, value u32 11
                v11 = array_set v1, index u32 20, value u32 21
                v14 = array_set v3, index u32 30, value u32 31
                inc_rc v3
                v17 = array_set v4, index u32 40, value u32 41
                v20 = array_set v2, index u32 50, value u32 51
                return v20
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let ctx = Context::new(function);

        let entry_params = function.dfg.block_parameters(function.entry_block());
        let v0 = entry_params[0];
        let v1 = entry_params[1];
        let v2 = entry_params[2];
        let v3 = entry_params[3];
        let v4 = entry_params[4];

        let array_sets = find_array_sets(function, &ctx);
        assert_eq!(array_sets.len(), 5, "five array_set instructions expected");

        for (block_id, idx, _inst_id, source, alias_set) in &array_sets {
            let precedes = ctx.some_inc_rc_precedes(alias_set, *block_id, *idx);
            let (expected, label) = if *source == v0 {
                (true, "v0: same-block earlier inc_rc")
            } else if *source == v1 {
                (true, "v1: entry-block inc_rc")
            } else if *source == v2 {
                (true, "v2: inc_rc in sibling branch (precedes in RPO)")
            } else if *source == v3 {
                (false, "v3: same-block later inc_rc")
            } else if *source == v4 {
                (false, "v4: no inc_rc")
            } else {
                panic!("unexpected array_set source {source:?}");
            };
            assert_eq!(precedes, expected, "{label}: expected precedes={expected}, got {precedes}");
        }
    }

    /// Well-formed example: in-loop `array_get v2` is fine because `v2`
    /// is a block parameter that is re-bound each iteration to the
    /// `array_set`'s result via the back-edge. The forward walk should kill
    /// `v2` from the use-set on entry to `b1`, and the `array_get v2` in `b2`
    /// (reached via cycle) is no longer aliased.
    #[test]
    fn reachable_use_walk_kills_block_param_on_entry() {
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
        let ctx = Context::new(function);
        let (block, idx, inst_id, _source, alias_set) =
            first_array_set(function, &ctx).expect("array_set present");

        let has_use = ctx.has_reachable_aliased_use(&alias_set, inst_id, block, idx);
        assert!(
            !has_use,
            "well-formed loop: array_get v2 is reached via cycle but v2 is rebound at b1; no aliased use should be found"
        );
    }

    /// PR-12671 malformed repro: the in-loop `array_get v0` reads the
    /// pre-header value `v0` directly, which is in the alias-set of the
    /// `array_set`'s source `v2`. `v0` is *not* a parameter of any block on the
    /// cycle, so it stays live in the use-set, and the walk flags the read.
    #[test]
    fn reachable_use_walk_detects_unprotected_predecessor_read() {
        let src = r#"
            brillig(inline) impure fn main f0 {
              b0(v0: [u32; 2], v1: u32):
                jmp b1(v0, u32 0, u32 0)
              b1(v2: [u32; 2], v3: u32, v4: u32):
                v7 = lt v4, v1
                jmpif v7 then: b2(), else: b3()
              b2():
                v6 = array_get v0, index u32 0 -> u32
                v10 = add v3, v6
                v12 = array_set v2, index u32 0, value u32 99
                v14 = add v4, u32 1
                jmp b1(v12, v10, v14)
              b3():
                return v3
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let ctx = Context::new(function);
        let (block, idx, inst_id, _source, alias_set) =
            first_array_set(function, &ctx).expect("array_set present");

        let has_use = ctx.has_reachable_aliased_use(&alias_set, inst_id, block, idx);
        assert!(
            has_use,
            "malformed loop: array_get v0 reads the pre-header value, which aliases the array_set's source via b1's pre-header jmp"
        );
    }

    /// Diamond-with-back-edges CFG: two predecessors of the array_set's
    /// block (`b3`) each kill a *different* alias-set member (`b1` kills
    /// `v3`, `b2` kills `v4`). The forward walk re-enters `b3` via
    /// `b3 → b4 → b1 → b3` with use-set `{v4, v0, v1}` and via
    /// `b3 → b4 → b2 → b3` with `{v3, v0, v1}` — neither a subset of the
    /// other. The bookkeeping must record the **union** of explored
    /// use-sets at `b3` so the cycle terminates.
    ///
    /// No aliased read exists; the walk should return `false`. A bug in the
    /// merge logic would surface either as non-termination or as a false
    /// positive on the array_set's own source `v5` (re-killed on each cycle
    /// entry).
    #[test]
    fn reachable_use_walk_merges_use_sets_across_paths() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2], v1: [u32; 2], v2: u1):
                jmpif v2 then: b1(v0), else: b2(v1)
              b1(v3: [u32; 2]):
                jmp b3(v3)
              b2(v4: [u32; 2]):
                jmp b3(v4)
              b3(v5: [u32; 2]):
                v8 = array_set v5, index u32 0, value u32 99
                jmpif v2 then: b4(), else: b5()
              b4():
                jmpif v2 then: b1(v5), else: b2(v5)
              b5():
                return
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let ctx = Context::new(function);
        let (block, idx, inst_id, _source, alias_set) =
            first_array_set(function, &ctx).expect("array_set present");

        let has_use = ctx.has_reachable_aliased_use(&alias_set, inst_id, block, idx);
        assert!(
            !has_use,
            "no aliased read exists; the walk must terminate and return false despite re-entering b3 with non-overlapping use-sets"
        );
    }

    /// When a single jmp passes the same value to multiple parameter
    /// positions (e.g. `jmp b1(v0, v0)`), the resulting sibling parameters
    /// alias each other. The union-find equivalence classes unify both
    /// parameters with `v0` (and so with each other), so an `array_get v2`
    /// after `array_set v1` is correctly recognized as an aliased use.
    #[test]
    fn alias_set_unifies_sibling_args_to_same_value() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2]):
                jmp b1(v0, v0)
              b1(v1: [u32; 2], v2: [u32; 2]):
                v5 = array_set v1, index u32 0, value u32 99
                v7 = array_get v2, index u32 0 -> u32
                return
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let ctx = Context::new(function);
        let (block, idx, inst_id, source, alias_set) =
            first_array_set(function, &ctx).expect("array_set present");

        // alias_set should include v0 (function param), v1 (= source), and
        // v2 (sibling param threaded the same v0 by b0's jmp).
        let v0 = function.dfg.block_parameters(function.entry_block())[0];
        let v2_param = function.dfg.block_parameters(block)[1];
        assert!(alias_set.contains(&v0), "alias-set must include v0");
        assert!(alias_set.contains(&source), "alias-set must include the source v1");
        assert!(alias_set.contains(&v2_param), "alias-set must include the sibling param v2");

        // The forward walk catches `array_get v2` as an aliased read.
        let has_use = ctx.has_reachable_aliased_use(&alias_set, inst_id, block, idx);
        assert!(has_use, "array_get v2 reads a sibling alias of the array_set's source v1");
    }

    /// Cross-block sibling aliasing. The array_set is in `b1`; its source
    /// `v1` aliases its sibling param `v2` (both bound to `v0` by `b0`'s
    /// `jmp b1(v0, v0)`). The post-array_set jmp threads `v2` forward to
    /// `b3`'s parameter `v4` — so `v4` is rebound to a value (`v2`) that
    /// is still in the alias-set, meaning `v4` *stays* aliased to the
    /// array_set's source.
    ///
    /// The forward walk must therefore *not* kill `v4` on entry to `b3`,
    /// and must flag the `array_get v4` as an aliased read. The
    /// per-arg kill rule (see [`super::succ_use_set`]) is what makes this
    /// work: `v4`'s arg `v2` is in `use_set`, so `v4` is preserved.
    #[test]
    fn reachable_use_walk_preserves_aliased_param_across_jmp() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2]):
                jmp b1(v0, v0)
              b1(v1: [u32; 2], v2: [u32; 2]):
                v5 = array_set v1, index u32 0, value u32 99
                jmp b3(v5, v2)
              b3(v6: [u32; 2], v7: [u32; 2]):
                v9 = array_get v7, index u32 0 -> u32
                return
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let ctx = Context::new(function);
        let (block, idx, inst_id, _source, alias_set) =
            first_array_set(function, &ctx).expect("array_set present");

        let has_use = ctx.has_reachable_aliased_use(&alias_set, inst_id, block, idx);
        assert!(
            has_use,
            "array_get v7 in b3 reads a value (v7 ← v2) that still aliases the array_set's source"
        );
    }

    /// Counterpart of the above: when the jmp arg is the array_set's own
    /// result (`v5`), the destination's parameter `v6` is rebound to a
    /// value that is *not* in the alias-set (the result was excluded at
    /// lookup time). The kill must fire so `v6` is removed from the
    /// use-set, and a downstream `array_get v6` must not be flagged.
    ///
    /// This is the "happy path" of the per-arg kill rule.
    #[test]
    fn reachable_use_walk_kills_param_rebound_to_array_set_result() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2]):
                jmp b1(v0)
              b1(v1: [u32; 2]):
                v4 = array_set v1, index u32 0, value u32 99
                jmp b2(v4)
              b2(v5: [u32; 2]):
                v7 = array_get v5, index u32 0 -> u32
                return
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let ctx = Context::new(function);
        let (block, idx, inst_id, _source, alias_set) =
            first_array_set(function, &ctx).expect("array_set present");

        let has_use = ctx.has_reachable_aliased_use(&alias_set, inst_id, block, idx);
        assert!(
            !has_use,
            "b2's v5 is rebound to v4 (the array_set's result, excluded from alias-set), so it is killed and array_get v5 is not aliased"
        );
    }

    /// `(block, idx-within-block, instruction-id, source-operand, alias-set)`
    /// for each `array_set` instruction in `function`, in source order.
    type ArraySetSite =
        (super::BasicBlockId, usize, super::InstructionId, ValueId, im::HashSet<ValueId>);

    fn find_array_sets(function: &Function, ctx: &Context<'_>) -> Vec<ArraySetSite> {
        let mut out = Vec::new();
        for block_id in function.reachable_blocks() {
            for (idx, instruction_id) in function.dfg[block_id].instructions().iter().enumerate() {
                if let Instruction::ArraySet { array, .. } = function.dfg[*instruction_id] {
                    let alias_set = ctx.alias_set_for(array);
                    out.push((block_id, idx, *instruction_id, array, alias_set));
                }
            }
        }
        out
    }

    fn first_array_set(function: &Function, ctx: &Context<'_>) -> Option<ArraySetSite> {
        find_array_sets(function, ctx).into_iter().next()
    }

    fn last_array_set(function: &Function, ctx: &Context<'_>) -> Option<ArraySetSite> {
        find_array_sets(function, ctx).into_iter().last()
    }
}
