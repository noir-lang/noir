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
//! # Algorithm
//!
//! For each `array_set vX, …`:
//!
//! 1. **Backward alias-set.** Compute the set of values that may share `vX`'s
//!    storage *at the array_set's program point* by walking block-parameter →
//!    predecessor-arg edges backward to a fixed point. Only aliasing introduced
//!    by the values that flow *into* `vX`'s binding is included — not the full
//!    UF closure, which would over-approximate by also conflating sibling
//!    parameters that share an arg through a downstream join.
//! 2. **inc_rc precedence / jmp-arg-participant.** If some `inc_rc` on an
//!    alias-set member either RPO-precedes the array_set or sits on a
//!    non-source alias that's also a jmp/jmpif arg, accept — the frontend is
//!    deliberately managing the aliasing.
//! 3. **Forward walk.** Otherwise, walk the CFG forward from the array_set with
//!    the alias-set as the initial use-set. At each block-parameter crossing
//!    we both **kill** params that the predecessor rebinds to a non-alias and
//!    **add** params whose arg is still an alias (so alias propagation stays
//!    accurate as the walk crosses joins and loops). A non-terminator
//!    instruction reading any value still in the use-set is the hazard.
//!
//! # Precondition
//!
//! Must be run **after `mem2reg_brillig`**. The alias walk through
//! block-parameter edges only reflects post-mem2reg aliasing; pre-mem2reg, an
//! `Allocate`/`Store`/`Load` chain would route aliasing through references that
//! this pass does not track.
//!
//! # Known limitations
//!
//! The verifier walks aliasing *only* through block-parameter edges. The
//! following are not tracked and are documented gaps:
//!
//! - **Sibling-args-of-same-value.** A `jmp b1(v, v)` makes `b1`'s two
//!   parameters runtime-equal, but the backward walk doesn't see this — each
//!   parameter's backward set includes `v` but not the other parameter. The
//!   `end_to_end_sibling_args_to_same_value_is_false_negative` and
//!   `end_to_end_sibling_args_across_jmp_is_false_negative` tests pin this
//!   down as documented false negatives. A previous version of this pass
//!   used a full union-find that caught these cases at the cost of
//!   path-merge over-approximation at downstream joins; the backward
//!   walk trades the sibling-detection for the precision.
//! - **Nested-array `MakeArray`**, **`IfElse` on arrays**, **non-inlined
//!   `Call` returns**, **`Store`/`Load` on ineligible (nested-ref)
//!   allocates** — same as before.

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use std::cmp::Ordering;

use acvm::FieldElement;

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
            value::ValueId,
        },
        opt::{LoopOrder, Loops},
        ssa_gen::Ssa,
    },
};

/// Verifies the `array_set` / `inc_rc` aliasing invariant on every Brillig
/// function in `ssa`. See the module-level docs for details.
///
/// The entire module containing this function is gated behind
/// `#[cfg(debug_assertions)]`, so it is a no-op (and absent at the linker
/// level) in release builds — see the pipeline wiring in
/// [`crate::ssa::primary_passes`].
pub(crate) fn verify_array_set_rc_invariant(ssa: &Ssa) -> RtResult<()> {
    for function in ssa.functions.values() {
        verify_function(function)?;
    }
    Ok(())
}

/// Per-function entry point. Skips ACIR functions (the invariant only
/// applies to Brillig, where `array_set` may mutate in place) and runs the
/// alias / dominance / reachable-use checks for every `array_set` in the
/// function.
fn verify_function(function: &Function) -> RtResult<()> {
    if !function.runtime().is_brillig() {
        return Ok(());
    }

    let ctx = Context::new(function);

    for block_id in function.reachable_blocks() {
        for (idx, instruction_id) in function.dfg[block_id].instructions().iter().enumerate() {
            let Instruction::ArraySet { array, index: write_index, .. } =
                function.dfg[*instruction_id]
            else {
                continue;
            };

            let alias_set = ctx.alias_set_for(array);

            // Cheap check first: if any `inc_rc` on an alias-set member
            // precedes this `array_set` in flow order, treat the aliasing
            // as already protected. See `some_inc_rc_precedes` for the
            // rationale (relaxed from dominance to RPO precedence).
            if ctx.some_inc_rc_precedes(&alias_set, array, block_id, idx) {
                continue;
            }

            // The array_set's index, if constant — lets the walk skip
            // `array_get`s on disjoint constant indices (the pedersen-style
            // pattern). A dynamic write index means we must conservatively
            // flag every aliased read.
            let write_index_const = function.dfg.get_numeric_constant(write_index);

            // Expensive: forward CFG walk looking for an aliased read.
            // A hit means the `array_set` may mutate storage in place
            // (RC=1) and a downstream instruction will observe the
            // pre-mutation contents through an aliased name.
            if let Some(hit) = ctx.find_reachable_aliased_use(
                &alias_set,
                *instruction_id,
                block_id,
                idx,
                write_index_const,
            ) {
                let call_stack = function.dfg.get_instruction_call_stack(*instruction_id);
                let aliased_use_call_stack =
                    function.dfg.get_instruction_call_stack(hit.instruction);
                let message = format!(
                    "array_set in function {} on array {array} has an aliased read of {} on a \
                     forward path with no preceding `inc_rc`; the in-place mutation would be \
                     observable through that alias",
                    function.name(),
                    hit.value,
                );
                return Err(RuntimeError::ArraySetAliasViolation {
                    message,
                    call_stack,
                    aliased_use_call_stack,
                });
            }
        }
    }
    Ok(())
}

/// Pre-computed indices over a Brillig function. The verifier's per-array_set
/// checks read from these structures rather than re-scanning the function.
struct Context<'f> {
    function: &'f Function,
    /// Dominator tree. Mutable because `dominates` populates an internal cache.
    dom_tree: DominatorTree,
    /// For each array-typed value `V`, the set of values that may share
    /// `V`'s storage **at `V`'s program point** — the source itself plus
    /// anything that flows backward into it through block-parameter →
    /// predecessor-arg chains. Computed to a fixed point.
    ///
    /// Values absent from this map have an implicit singleton class
    /// `{v}` (typically instruction results that aren't passed as args
    /// anywhere they'd matter — handled in [`Context::alias_set_for`]).
    ///
    /// This is the *backward* alias-set, not the full union-find closure
    /// over all `(param, arg)` pairs. The difference matters at joins:
    /// two values that flow into different parameters of the same block
    /// are *not* siblings of each other under backward aliasing, because
    /// neither is reachable from the other along param→arg edges. This
    /// avoids the path-merge over-approximation a full UF would
    /// introduce.
    backward_aliases: HashMap<ValueId, im::HashSet<ValueId>>,
    /// For each array-typed value defined by an instruction, the
    /// `(block, instruction-position-within-block)` of the defining
    /// instruction. Used by the def-block-entry kill rule in
    /// [`Context::succ_use_set`].
    array_value_defs: HashMap<ValueId, (BasicBlockId, usize)>,
    /// Every array-typed value that does **not** carry the pre-mutation
    /// aliasing of an `array_set`'s source. Filtered out of every
    /// alias-set (except when the value is the `array_set`'s own source)
    /// — see [`Context::alias_set_for`]. Includes:
    /// - `array_set` results: represent the *post*-mutation value of the
    ///   source. Uses of them (or anything threaded from them through
    ///   block params) are intentional reads, not hazards.
    /// - `Call` results: typically fresh arrays allocated by the callee.
    ///   Heuristic — a function that returns its input *is* a real
    ///   alias, and filtering would miss those cases; that's a
    ///   documented gap. In practice the frontend's array-returning
    ///   functions allocate fresh storage.
    non_aliasing_array_values: HashSet<ValueId>,
    /// `MakeArray` results that appear as a jmp/jmpif arg on at least one
    /// loop back-edge. Such a `make_array` re-executes on every loop
    /// iteration and represents fresh storage per iteration, so any
    /// back-edge that puts it in a loop-header parameter's class is
    /// conflating distinct runtime storages across iterations. Filtered
    /// out of the alias-set on lookup — see [`Context::alias_set_for`].
    ///
    /// `MakeArray` results that are *not* back-edge args are kept in the
    /// alias-set: they represent a one-time allocation whose storage
    /// the array_set may mutate in place.
    iteration_local_make_arrays: HashSet<ValueId>,
    /// `inc_rc value` instructions indexed by their operand. Each entry is
    /// the `(block, instruction-position-within-block)` of one `inc_rc`.
    inc_rc_locations: HashMap<ValueId, Vec<(BasicBlockId, usize)>>,
    /// Per-block sorted list of `(idx, instruction-id, array-operand)`
    /// triples — one entry per array-typed operand of each non-terminator
    /// instruction. Lets the reachable-use walk skip over instructions that
    /// have no array operand.
    array_operand_uses: HashMap<BasicBlockId, Vec<ArrayOperandUse>>,
    /// Values that appear at least once as a jmp/jmpif arg on **any**
    /// edge (forward or back). Used by [`Context::some_inc_rc_precedes`]
    /// as the "the frontend is managing aliasing introduced by
    /// block-parameter threading" signal: an `inc_rc` on a non-source
    /// alias that's also threaded through a block parameter suggests
    /// the codegen is aware of the aliasing and inserted the bump
    /// deliberately, even if the bump's program point doesn't dominate
    /// the array_set in flow order.
    jmp_args: HashSet<ValueId>,
}

impl<'f> Context<'f> {
    fn new(function: &'f Function) -> Self {
        let cfg = ControlFlowGraph::with_function(function);
        let post_order = PostOrder::with_cfg(&cfg);
        let dom_tree = DominatorTree::with_cfg_and_post_order(&cfg, &post_order);

        // Cache the reverse-post-order block list once. Used for the
        // per-instruction setup pass below and (more importantly) by
        // [`compute_backward_aliases`], whose fixed-point converges in
        // far fewer passes when blocks are visited in RPO than in the
        // ID-sorted order `function.reachable_blocks()` returns.
        let rpo: Vec<BasicBlockId> = post_order.into_vec_reverse();

        // Loop back-edges so we can recognize iteration-local
        // `make_array` results (those passed as args on a back-edge),
        // which represent fresh-per-iteration storage and so don't
        // carry the array_set's pre-mutation aliasing forward through
        // the loop header.
        let back_edges: HashSet<(BasicBlockId, BasicBlockId)> =
            Loops::find_all(function, LoopOrder::InsideOut)
                .yet_to_unroll
                .iter()
                .map(|l| (l.back_edge_start, l.header))
                .collect();

        let mut inc_rc_locations: HashMap<ValueId, Vec<(BasicBlockId, usize)>> = HashMap::default();
        let mut array_operand_uses: HashMap<BasicBlockId, Vec<ArrayOperandUse>> =
            HashMap::default();
        let mut array_value_defs: HashMap<ValueId, (BasicBlockId, usize)> = HashMap::default();
        let mut non_aliasing_array_values: HashSet<ValueId> = HashSet::default();
        let mut make_array_values: HashSet<ValueId> = HashSet::default();
        let mut back_edge_args: HashSet<ValueId> = HashSet::default();
        let mut jmp_args: HashSet<ValueId> = HashSet::default();
        // For each destination block, the list of `(predecessor, args)`
        // pairs collected from terminators. Used to drive the backward
        // alias-set fixed-point: each block parameter's set is the union
        // of its arg at that position from every incoming edge.
        let mut incoming_edges: HashMap<BasicBlockId, Vec<(BasicBlockId, Vec<ValueId>)>> =
            HashMap::default();

        // Single pass over every reachable block to populate
        // per-instruction indices and per-block incoming-edge tables.
        for &block_id in &rpo {
            let mut operand_uses: Vec<ArrayOperandUse> = Vec::new();
            for (idx, instruction_id) in function.dfg[block_id].instructions().iter().enumerate() {
                let instruction = &function.dfg[*instruction_id];

                if let Instruction::IncrementRc { value } = instruction {
                    inc_rc_locations.entry(*value).or_default().push((block_id, idx));
                }

                let is_non_aliasing =
                    matches!(instruction, Instruction::ArraySet { .. } | Instruction::Call { .. });
                let is_make_array = matches!(instruction, Instruction::MakeArray { .. });
                for &result in function.dfg.instruction_results(*instruction_id) {
                    if function.dfg.type_of_value(result).contains_an_array() {
                        array_value_defs.insert(result, (block_id, idx));
                        if is_non_aliasing {
                            non_aliasing_array_values.insert(result);
                        }
                        if is_make_array {
                            make_array_values.insert(result);
                        }
                    }
                }

                // `inc_rc` / `dec_rc` are ref-count bumps, not reads of the
                // array contents, so their operands aren't "aliased reads" of
                // pre-mutation storage. `inc_rc` is already accounted for
                // separately by `inc_rc_locations` / `some_inc_rc_precedes`.
                let is_rc_op = matches!(
                    instruction,
                    Instruction::IncrementRc { .. } | Instruction::DecrementRc { .. }
                );
                if !is_rc_op {
                    instruction.for_each_value(|v| {
                        if function.dfg.type_of_value(v).contains_an_array() {
                            operand_uses.push((idx, *instruction_id, v));
                        }
                    });
                }
            }
            if !operand_uses.is_empty() {
                array_operand_uses.insert(block_id, operand_uses);
            }

            if let Some(terminator) = function.dfg[block_id].terminator() {
                match terminator {
                    TerminatorInstruction::Jmp { destination, arguments, .. } => {
                        incoming_edges
                            .entry(*destination)
                            .or_default()
                            .push((block_id, arguments.to_vec()));
                        jmp_args.extend(arguments.iter().copied());
                        if back_edges.contains(&(block_id, *destination)) {
                            back_edge_args.extend(arguments.iter().copied());
                        }
                    }
                    TerminatorInstruction::JmpIf {
                        then_destination,
                        then_arguments,
                        else_destination,
                        else_arguments,
                        ..
                    } => {
                        incoming_edges
                            .entry(*then_destination)
                            .or_default()
                            .push((block_id, then_arguments.to_vec()));
                        incoming_edges
                            .entry(*else_destination)
                            .or_default()
                            .push((block_id, else_arguments.to_vec()));
                        jmp_args.extend(then_arguments.iter().copied());
                        jmp_args.extend(else_arguments.iter().copied());
                        if back_edges.contains(&(block_id, *then_destination)) {
                            back_edge_args.extend(then_arguments.iter().copied());
                        }
                        if back_edges.contains(&(block_id, *else_destination)) {
                            back_edge_args.extend(else_arguments.iter().copied());
                        }
                    }
                    _ => (),
                }
            }
        }

        // A `make_array` is iteration-local iff its result appears on a
        // loop back-edge: that's the signal that it re-executes each
        // iteration, so the storage the loop-header parameter receives
        // through the back-edge is freshly allocated rather than the
        // same storage the array_set may mutate in place.
        let iteration_local_make_arrays: HashSet<ValueId> =
            make_array_values.intersection(&back_edge_args).copied().collect();

        let backward_aliases = compute_backward_aliases(function, &rpo, &incoming_edges);

        Self {
            function,
            dom_tree,
            backward_aliases,
            array_value_defs,
            non_aliasing_array_values,
            iteration_local_make_arrays,
            inc_rc_locations,
            array_operand_uses,
            jmp_args,
        }
    }

    /// Look up `source`'s backward alias-set and filter out values that
    /// can't represent the pre-mutation storage of an `array_set` on
    /// `source`. The source itself is always kept.
    ///
    /// **Non-aliasing-result filter.** Drop values produced by an
    /// `array_set` or `Call`:
    /// - **`array_set` result** represents a *post*-mutation value of its
    ///   source. Uses of it (or of any name it gets re-bound to through
    ///   block-parameter threading) are intentional reads of the mutated
    ///   storage, not hazards. Keeping them in the alias-set would defeat
    ///   the per-arg kill rule in [`Context::succ_use_set`]: a back-edge
    ///   `jmp b(v_arr_set)` would see `v_arr_set` in the use-set and
    ///   refuse to kill the receiving param, letting the alias leak past
    ///   the loop.
    /// - **`Call` result** is typically a fresh array allocated by the
    ///   callee, so it isn't a real alias of the array_set's source.
    ///   This is a heuristic — a function that returns its input *would*
    ///   create a real alias, and we'd miss that. In practice the
    ///   frontend's array-returning functions allocate fresh storage.
    ///
    /// Also drop **iteration-local `MakeArray` results** — those that
    /// appear on at least one loop back-edge
    /// ([`Context::iteration_local_make_arrays`]). A `make_array` on a
    /// back-edge re-executes each iteration and allocates fresh
    /// storage, so the loop-header parameter it feeds on the back-edge
    /// holds a *different* allocation in the next iteration than the
    /// one this iteration's `array_set` may have mutated.
    /// **Non-back-edge `MakeArray` results stay in the alias-set** —
    /// they represent a one-time allocation whose storage the
    /// array_set can mutate in place.
    ///
    /// The source itself is kept even when it happens to be filtered
    /// (e.g. a chain `v_a = array_set _ ; v_b = array_set v_a`): `v_a`
    /// is *this* check's source, so its forward uses are exactly what
    /// we want to look for.
    fn alias_set_for(&self, source: ValueId) -> im::HashSet<ValueId> {
        let class = self
            .backward_aliases
            .get(&source)
            .cloned()
            .unwrap_or_else(|| im::HashSet::unit(source));
        class
            .into_iter()
            .filter(|&v| {
                if v == source {
                    return true;
                }
                if self.non_aliasing_array_values.contains(&v) {
                    return false;
                }
                if self.iteration_local_make_arrays.contains(&v) {
                    return false;
                }
                true
            })
            .collect()
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
    /// shape PR-12671 had. The jmp-arg-participant relaxation below
    /// widens this further to cover `inc_rc`s placed after the
    /// array_set or on sibling branches.
    ///
    /// # Jmp-arg-participant relaxation
    ///
    /// In addition to RPO precedence, an `inc_rc` is also accepted when
    /// it lives on an alias-set member that:
    ///
    /// - is **not** the array_set's own source (an `inc_rc` on the
    ///   source itself, *after* the array_set, doesn't protect that
    ///   array_set — it would be suspicious frontend output rather
    ///   than a signal); and
    /// - appears as a jmp/jmpif arg on **any** edge somewhere in the
    ///   function ([`Context::jmp_args`]) — forward or back.
    ///
    /// The intuition: the frontend doesn't bother emitting `inc_rc` on
    /// a non-source alias unless it's deliberately managing the
    /// aliasing introduced by threading the value through a block
    /// parameter — whether that's a loop's back-edge thread-back or a
    /// non-loop forward edge that re-seeds a value with (say) a global.
    /// Either way, the `inc_rc`'s presence is a signal that the codegen
    /// is RC-aware, regardless of where exactly the bump is placed in
    /// flow order.
    ///
    /// Well-formed SSA contains no `DecrementRc`, so we don't need to
    /// worry about a `dec_rc` intervening between the `inc_rc` and the
    /// `array_set`.
    fn some_inc_rc_precedes(
        &self,
        alias_set: &im::HashSet<ValueId>,
        source: ValueId,
        array_set_block: BasicBlockId,
        array_set_idx: usize,
    ) -> bool {
        for value in alias_set {
            let Some(locations) = self.inc_rc_locations.get(value) else {
                continue;
            };
            // Jmp-arg-participant relaxation: an `inc_rc` on a non-source
            // alias that's also a jmp/jmpif arg (forward or back-edge) is
            // a codegen signal regardless of program-point order.
            if *value != source && self.jmp_args.contains(value) {
                return true;
            }
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
    /// **Index-aware filtering.** `write_index_const` is the `array_set`'s
    /// index when it is a constant. When both the write and a candidate
    /// `array_get` on the aliased operand have constant indices that
    /// differ, the access is at a disjoint position from the in-place
    /// mutation — runtime contents at the read index are unaffected by
    /// the write — so we skip the candidate. Conservative whenever
    /// either side is dynamic, and applied only to `array_get` (other
    /// instruction kinds that use an alias-set operand are always
    /// flagged, since the SSA-vs-runtime divergence isn't index-local
    /// for them).
    ///
    /// **Cycle detection.** Re-visiting a block with a use-set that is a
    /// subset of what we already explored at that block adds no new
    /// information. We record the *union* of use-sets seen per block and
    /// skip on subset matches.
    fn find_reachable_aliased_use(
        &self,
        alias_set: &im::HashSet<ValueId>,
        array_set_id: InstructionId,
        array_set_block: BasicBlockId,
        array_set_idx: usize,
        write_index_const: Option<FieldElement>,
    ) -> Option<AliasedUse> {
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
                    if !use_set.contains(&operand) {
                        continue;
                    }
                    // Index-disjoint filter: if both the array_set and a
                    // candidate `array_get` use constant indices that
                    // don't match, the read is at a position the write
                    // didn't touch — skip.
                    if let Some(write_idx) = write_index_const
                        && let Instruction::ArrayGet { index: read_idx, .. } =
                            self.function.dfg[inst_id]
                        && let Some(read_idx) = self.function.dfg.get_numeric_constant(read_idx)
                        && write_idx != read_idx
                    {
                        continue;
                    }
                    return Some(AliasedUse { instruction: inst_id, value: operand });
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

        None
    }

    /// Compute the use-set carried into `dest` when its predecessor jumps
    /// with `arguments`.
    ///
    /// Three rules apply, in order:
    ///
    /// 1. **Per-arg propagation — block parameters of `dest`.** For each
    ///    `dest.params[i]`, look at the corresponding `arguments[i]`:
    ///    - **Kill.** If the param is in `use_set` and the arg is not, the
    ///      param is rebound to a value that no longer aliases the
    ///      array_set's source (most commonly the array_set's own
    ///      result, excluded at lookup time): drop it.
    ///    - **Add.** If the param is not in `use_set` but the arg is,
    ///      this edge introduces a fresh alias: the param at `dest`'s
    ///      entry shares storage with an alias-set member, so add it.
    ///      This is what makes alias propagation accurate at joins and
    ///      loop back-edges without needing a full path-insensitive UF.
    ///    - Otherwise (both in or both out), no change.
    ///    Only array-typed params participate.
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

        // (1) Per-arg propagation for params of `dest`.
        let params = self.function.dfg.block_parameters(dest);
        for (i, &param) in params.iter().enumerate() {
            if !self.function.dfg.type_of_value(param).contains_an_array() {
                continue;
            }
            let arg_in_use_set = arguments.get(i).is_some_and(|arg| use_set.contains(arg));
            let param_in_use_set = use_set.contains(&param);
            match (param_in_use_set, arg_in_use_set) {
                (true, false) => {
                    result.remove(&param);
                }
                (false, true) => {
                    result.insert(param);
                }
                _ => {}
            }
        }

        // (2) Unconditional kill for instruction-defined values whose
        // def-block is `dest` (re-execution on cycle re-entry produces a
        // fresh value).
        for &v in use_set {
            if self.array_value_defs.get(&v).map(|(b, _)| *b) == Some(dest) {
                result.remove(&v);
            }
        }

        result
    }
}

/// Compute, for each array-typed block parameter, the set of values that
/// may share its storage at the parameter's binding — itself plus every
/// value that flows into it through some chain of predecessor →
/// block-parameter edges.
///
/// Runs a fixed-point iteration in reverse-post-order: each block
/// parameter starts with `{p}`, and each pass unions in `args[p_index]`
/// (and its own backward set, if it's also a block parameter) for
/// every incoming edge. The iteration stops when no set grows.
///
/// Visiting blocks in RPO means each parameter sees its forward-edge
/// predecessors' sets *already at their fixed point* (RPO orders
/// predecessor before successor along forward edges), so the only
/// source of additional passes is loop back-edges. In practice this
/// converges in 1 pass for loop-free code and 2-3 for typical
/// loops — versus O(chain-depth) passes under arbitrary block order.
///
/// # Why backward, not full union-find
///
/// A full UF over `(param, arg)` pairs is symmetric: two sibling args
/// passed to the same block end up in the same class via the shared
/// parameter, even though at runtime they never refer to the same
/// storage (each runtime path takes only one of them). The backward
/// walk follows the directed param-←-arg edges, so sibling parameters
/// don't co-mingle in each other's sets — only the predecessors that
/// flow *into* a given parameter do.
///
/// # Why we deliberately do **not** chase past `array_set` results
///
/// At runtime the result may or may not alias the source depending on
/// RC. The verifier's job is to flag the cases where it *might*. We
/// model this by *excluding* the result from each alias-set at lookup
/// time (in [`Context::alias_set_for`]) rather than refusing to chase
/// through it here. Otherwise a chain
/// `v1 = array_set v0 ; v2 = array_set v1 ; v3 = array_set v2` would
/// pull `v0` into the source of every later array_set, and an
/// `inc_rc v0` that legitimately protects only `v1`'s array_set would
/// falsely appear to protect `v3` as well. See
/// `alias_set_does_not_walk_array_set_chains` for a worked example —
/// the chain isn't visible to the backward walk anyway because
/// `v1`, `v2`, `v3` are instruction results (not block parameters),
/// so there's nothing to chase from `v3`.
fn compute_backward_aliases(
    function: &Function,
    rpo: &[BasicBlockId],
    incoming_edges: &HashMap<BasicBlockId, Vec<(BasicBlockId, Vec<ValueId>)>>,
) -> HashMap<ValueId, im::HashSet<ValueId>> {
    let mut result: HashMap<ValueId, im::HashSet<ValueId>> = HashMap::default();

    // Seed each array-typed block parameter with `{p}`.
    for &block_id in rpo {
        for &param in function.dfg.block_parameters(block_id) {
            if function.dfg.type_of_value(param).contains_an_array() {
                result.insert(param, im::HashSet::unit(param));
            }
        }
    }

    // Fixed-point in RPO: keep iterating until no parameter's set grows.
    // Each pass folds in args from every incoming edge, transitively
    // through any predecessor arg that's itself a block parameter.
    let mut changed = true;
    while changed {
        changed = false;
        for &block_id in rpo {
            let params = function.dfg.block_parameters(block_id);
            let Some(incoming) = incoming_edges.get(&block_id) else {
                continue;
            };
            for (i, &param) in params.iter().enumerate() {
                if !function.dfg.type_of_value(param).contains_an_array() {
                    continue;
                }
                let mut new_set = result.get(&param).cloned().unwrap_or_else(im::HashSet::new);
                let prev_len = new_set.len();
                for (_pred, args) in incoming {
                    if let Some(&arg) = args.get(i) {
                        // Pull in `arg`'s backward set if it's a block
                        // parameter; otherwise the arg is an instruction
                        // result whose backward set is the singleton.
                        if let Some(arg_set) = result.get(&arg) {
                            new_set.extend(arg_set.iter().copied());
                        } else {
                            new_set.insert(arg);
                        }
                    }
                }
                if new_set.len() > prev_len {
                    result.insert(param, new_set);
                    changed = true;
                }
            }
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

/// A non-terminator instruction reachable forward from an `array_set` that
/// reads a value still in the alias-set — the *aliased use* that the
/// reachable-use walk surfaced. Carries both pieces so callers can build a
/// diagnostic that names the offending alias value and the instruction
/// that observed it.
#[derive(Debug)]
struct AliasedUse {
    /// The instruction that uses the aliased value as a (non-terminator)
    /// operand.
    instruction: InstructionId,
    /// The alias-set member that was used. Names *which* alias triggered
    /// the flag — useful when the alias-set has more than one member.
    value: ValueId,
}

#[cfg(test)]
mod tests {
    use super::Context;
    use crate::ssa::{
        ir::{function::Function, instruction::Instruction, value::ValueId},
        ssa_gen::Ssa,
    };

    /// Parse `src`, run the verifier, and require it to accept the SSA.
    /// Panics with the unexpected error otherwise.
    fn assert_verifier_accepts(src: &str) {
        assert_verifier_accepts_because(src, "");
    }

    /// Same as [`assert_verifier_accepts`] but includes `reason` in the
    /// panic message — useful for documenting why the SSA is *expected*
    /// to be accepted (e.g. "loop exit reads a rebound block-param").
    fn assert_verifier_accepts_because(src: &str, reason: &str) {
        let ssa = Ssa::from_str(src).expect("SSA parses");
        if let Err(err) = super::verify_array_set_rc_invariant(&ssa) {
            if reason.is_empty() {
                panic!("expected the verifier to accept, but it rejected: {err:?}");
            } else {
                panic!("expected the verifier to accept ({reason}), but it rejected: {err:?}");
            }
        }
    }

    /// Parse `src`, run the verifier, and require it to reject the SSA
    /// with an [`crate::errors::RuntimeError::ArraySetAliasViolation`].
    /// Panics on any other outcome.
    fn assert_verifier_rejects(src: &str) {
        let ssa = Ssa::from_str(src).expect("SSA parses");
        let err = super::verify_array_set_rc_invariant(&ssa)
            .expect_err("expected the verifier to reject");
        assert!(
            matches!(err, crate::errors::RuntimeError::ArraySetAliasViolation { .. }),
            "expected ArraySetAliasViolation, got {err:?}",
        );
    }

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
        assert_verifier_accepts(src);
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
        assert_verifier_accepts(src);
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
        assert_verifier_rejects(src);
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
        assert_verifier_accepts(src);
    }

    /// Index-aware filter: a constant-index `array_set` followed by a
    /// constant-index `array_get` on the same alias at a **different**
    /// index is safe. In-place mutation at one position doesn't affect
    /// reads at another, so the verifier should accept.
    #[test]
    fn end_to_end_array_set_array_get_disjoint_constant_indices_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2]):
                v3 = array_set v0, index u32 0, value u32 99
                v5 = array_get v0, index u32 1 -> u32
                return v5
            }"#;
        assert_verifier_accepts_because(
            src,
            "array_set at idx 0 + array_get at idx 1 access disjoint positions; not a hazard",
        );
    }

    /// Counterpart to the disjoint case: matching constant indices mean
    /// the read observes the in-place mutation, so the verifier must
    /// reject.
    #[test]
    fn end_to_end_array_set_array_get_matching_constant_indices_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2]):
                v3 = array_set v0, index u32 0, value u32 99
                v5 = array_get v0, index u32 0 -> u32
                return v5
            }"#;
        assert_verifier_rejects(src);
    }

    /// A **dynamic** write index could touch any position, so the filter
    /// can't prove disjointness; the verifier must conservatively reject
    /// any aliased read (even one at a known disjoint-looking constant
    /// index).
    #[test]
    fn end_to_end_array_set_dynamic_index_with_array_get_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2], v1: u32):
                v3 = array_set v0, index v1, value u32 99
                v5 = array_get v0, index u32 1 -> u32
                return v5
            }"#;
        assert_verifier_rejects(src);
    }

    /// Symmetric to the previous case: write index is constant but the
    /// read's index is dynamic. The runtime read could land on the
    /// write's position, so the verifier conservatively rejects.
    #[test]
    fn end_to_end_array_set_constant_with_dynamic_array_get_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2], v1: u32):
                v3 = array_set v0, index u32 0, value u32 99
                v5 = array_get v0, index v1 -> u32
                return v5
            }"#;
        assert_verifier_rejects(src);
    }

    /// The index filter applies *only* to `array_get`. A non-`array_get`
    /// use of the alias (here a second `array_set` on the same source
    /// with a different constant index) is still flagged conservatively,
    /// because the SSA-vs-runtime divergence isn't local to the read
    /// index for those use kinds.
    #[test]
    fn end_to_end_array_set_followed_by_another_array_set_on_alias_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2]):
                v3 = array_set v0, index u32 0, value u32 99
                v5 = array_set v0, index u32 1, value u32 88
                v7 = array_get v5, index u32 0 -> u32
                return v7
            }"#;
        assert_verifier_rejects(src);
    }

    /// Direct shape that the `MakeArray`-non-aliasing filter must NOT
    /// silence: the array_set's *source* is itself a `make_array` result,
    /// and the same `make_array` is read forward via `array_get`. The
    /// source-self-preservation rule in [`Context::alias_set_for`] keeps
    /// the source in its own alias-set regardless of the filter, so the
    /// walk still finds the aliased read and flags.
    #[test]
    fn end_to_end_array_set_on_make_array_with_forward_read_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v1 = make_array [u32 1, u32 2, u32 3] : [u32; 3]
                v3 = array_set v1, index u32 0, value u32 99
                v5 = array_get v1, index u32 0 -> u32
                return v5
            }"#;
        assert_verifier_rejects(src);
    }

    /// Aliased shape that the `MakeArray`-non-aliasing filter must NOT
    /// silence: a `make_array` is threaded forward into a block parameter
    /// via a *forward* edge (no loop), the parameter is the array_set's
    /// source, and the original `make_array` value is read forward. The
    /// `make_array` represents the same one-time allocation that the
    /// `array_set` may mutate in place at runtime — so dropping it from
    /// the alias-set would lose the hazard.
    #[test]
    fn end_to_end_make_array_aliased_via_forward_block_param_with_forward_read_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v1 = make_array [u32 1, u32 2, u32 3] : [u32; 3]
                jmp b1(v1)
              b1(v2: [u32; 3]):
                v4 = array_set v2, index u32 0, value u32 99
                v6 = array_get v1, index u32 0 -> u32
                return v6
            }"#;
        assert_verifier_rejects(src);
    }

    /// End-to-end regression for an AST-fuzzer-discovered pattern: a
    /// loop body that mutates the loop-variable (an `array_set` whose
    /// source is the loop-header parameter) and then re-seeds the
    /// loop variable with a global on the back-edge — the user-source
    /// equivalent of `for _ { a[i] = …; a = G_A }`. The UF unifies the
    /// loop param with the function arg (forward edge into the
    /// header) AND with the global (back-edge into the header), so a
    /// post-loop `array_get` on the loop param appears as an aliased
    /// read of the function arg's storage. At runtime the loop param
    /// at the loop exit is always the global (last back-edge binding),
    /// and the global has been `inc_rc`'d, so its `RC ≥ 2` from iter
    /// 1+ and the array_set never mutates it; iter 0's mutation hits
    /// the function arg's caller-side storage, which is no longer
    /// referenced after iter 0's back-edge re-bind. The inc_rc'd-global
    /// filter drops the global from the alias-set, the per-arg kill on
    /// the back-edge then drops the loop param, and the walk
    /// terminates without flagging.
    #[test]
    fn end_to_end_loop_reseeded_with_inc_rcd_global_is_accepted() {
        let src = r#"
            g0 = u32 10
            g1 = u32 20
            g2 = u32 30
            g3 = make_array [u32 10, u32 20, u32 30] : [u32; 3]

            brillig(inline) fn main f0 {
              b0(v4: [u32; 3]):
                jmp b1(u32 0, v4)
              b1(v8: u32, v19: [u32; 3]):
                v9 = lt v8, u32 3
                jmpif v9 then: b2(), else: b3()
              b2():
                v14 = array_set v19, index u32 1, value u32 40
                inc_rc g3
                v16 = unchecked_add v8, u32 1
                jmp b1(v16, g3)
              b3():
                v18 = array_get v19, index u32 1 -> u32
                return v18
            }"#;
        assert_verifier_accepts_because(
            src,
            "the back-edge re-seeds the loop variable with an inc_rc'd global, so the post-loop read sees the global's pristine storage — not the function arg's caller-side storage that iter 0's array_set may have mutated",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered pattern: a
    /// callee receives an array argument, makes a *fresh* `make_array`
    /// copy that's threaded into the loop header on the forward edge,
    /// mutates the loop variable inside the body, and then re-seeds the
    /// loop variable with the *original* argument on the back-edge.
    /// The original argument has its `inc_rc` emitted in the loop body
    /// (right before the back-edge), so iter 1+ sees `RC ≥ 2` on it
    /// and the array_set allocates fresh. Iter 0's array_set mutates
    /// the fresh local copy, which is no longer referenced after the
    /// back-edge re-bind — so the apparent post-array_set read of the
    /// loop variable (the next iteration's `array_get` on the loop
    /// header param) is actually reading the original argument's
    /// pristine storage. The jmp-arg-participant relaxation in
    /// [`Context::some_inc_rc_precedes`] accepts this: the original arg
    /// is a non-source alias with an `inc_rc`, and it appears as a
    /// back-edge arg — a codegen signal that the frontend is
    /// deliberately managing the loop aliasing.
    #[test]
    fn end_to_end_loop_reseeded_with_inc_rcd_entry_param_is_accepted() {
        let src = r#"
            brillig(inline_never) fn func_1 f0 {
              b0(v0: [[u8; 3]; 4]):
                v5 = array_get v0, index u32 0 -> [u8; 3]
                inc_rc v5
                inc_rc v5
                v6 = make_array [v5, v5, v5, v5] : [[u8; 3]; 4]
                jmp b1(u32 0, v6)
              b1(v9: u32, v24: [[u8; 3]; 4]):
                v10 = lt v9, u32 3
                jmpif v10 then: b2(), else: b3()
              b2():
                v13 = array_get v24, index u32 3 -> [u8; 3]
                inc_rc v13
                v20 = make_array b"XDR"
                v25 = array_set v24, index u32 3, value v20
                inc_rc v0
                v23 = unchecked_add v9, u32 1
                jmp b1(v23, v0)
              b3():
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "v0 (the function arg) is a back-edge arg with an inc_rc in the loop body, so iter 1+'s array_set on v0 allocates fresh; iter 0's array_set mutates v6 (the fresh forward-edge make_array), which is no longer referenced after the back-edge re-bind. The jmp-arg-participant relaxation accepts the inc_rc on v0 as the codegen signal.",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered pattern with
    /// *nested* loops: an inner-loop body mutates the inner-loop's
    /// header parameter, but the value that gets RC-protected by an
    /// `inc_rc` (the function arg `v0`) sits on the *outer* loop's
    /// back-edge — not directly on the inner loop's back-edge. The
    /// inner source's UF class still contains `v0` transitively, via
    /// the chain `inner_param ↔ outer_param` (forward edge into the
    /// inner loop) and `outer_param ↔ v0` (outer back-edge).
    ///
    /// The current jmp-arg-participant relaxation in
    /// [`Context::some_inc_rc_precedes`] handles this uniformly: `v0` is
    /// a non-source alias with an `inc_rc` and appears as a jmp arg
    /// (on the outer back-edge), so the array_set is accepted.
    #[test]
    fn end_to_end_nested_loop_outer_back_edge_inc_rcd_arg_filtered_for_inner_source() {
        let src = r#"
            brillig(inline) fn func_1 f0 {
              b0(v0: [u1; 3], v1: [u1; 3]):
                jmp b1(u32 0, v1)
              b1(v5: u32, v21: [u1; 3]):
                v7 = lt v5, u32 4
                jmpif v7 then: b2(), else: b3()
              b2():
                v11 = array_get v21, index u32 0 -> u1
                jmpif v11 then: b4(), else: b5()
              b3():
                return
              b4():
                jmp b6(v21)
              b5():
                jmp b7(u32 0, v21)
              b6(v23: [u1; 3]):
                inc_rc v0
                v20 = unchecked_add v5, u32 1
                jmp b1(v20, v0)
              b7(v14: u32, v22: [u1; 3]):
                v15 = lt v14, u32 2
                jmpif v15 then: b8(), else: b9()
              b8():
                v16 = array_get v0, index u32 0 -> u1
                v18 = array_set v22, index u32 0, value v16
                v19 = unchecked_add v14, u32 1
                jmp b7(v19, v18)
              b9():
                jmp b6(v22)
            }"#;
        assert_verifier_accepts_because(
            src,
            "v0 is in the inner source's class transitively (via outer header v21's back-edge) and has an `inc_rc`. The jmp-arg-participant relaxation accepts: v0 ≠ source and v0 is a jmp arg (on the outer back-edge), so the inc_rc on v0 is taken as the codegen signal that loop aliasing is being managed.",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered pattern: a
    /// function arg that's *both* forward-threaded into an unrelated
    /// early-return branch *and* back-edge-threaded into the outer loop
    /// that carries an inner-loop array_set. The inc_rc in the
    /// outer-loop body guarantees `RC ≥ 2` by the time the inner source
    /// actually equals the arg at runtime, so the array_set on it is
    /// forced to allocate fresh.
    #[test]
    fn end_to_end_inc_rcd_arg_forward_in_unrelated_branch_and_back_to_outer_loop_is_accepted() {
        let src = r#"
            brillig(inline) fn func_1 f0 {
              b0(v0: [u1; 3], v1: [u1; 3], v2: u1):
                jmpif v2 then: b1(), else: b2()
              b1():
                jmp b_join(v0, v1)
              b_join(v_a: [u1; 3], v_b: [u1; 3]):
                jmp b_exit()
              b_exit():
                return
              b2():
                jmp b_outer(u32 0, v1)
              b_outer(v5: u32, v_outer_param: [u1; 3]):
                jmpif u1 1 then: b_outer_body(), else: b_exit()
              b_outer_body():
                jmp b_inner(u32 0, v_outer_param)
              b_inner(v6: u32, v_inner_param: [u1; 3]):
                jmpif u1 1 then: b_inner_body(), else: b_outer_tail()
              b_inner_body():
                v_set = array_set v_inner_param, index u32 0, value u1 0
                v7 = unchecked_add v6, u32 1
                jmp b_inner(v7, v_set)
              b_outer_tail():
                inc_rc v0
                v8 = unchecked_add v5, u32 1
                jmp b_outer(v8, v0)
            }"#;
        assert_verifier_accepts_because(
            src,
            "v0 is a forward arg to b_early_exit (unrelated region) and a back-edge arg to the outer loop, and carries an inc_rc. The jmp-arg-participant relaxation accepts: v0 ≠ source, v0 ∈ jmp_args, and has inc_rc — taken as the codegen signal regardless of which edge the inc_rc-bearing arg was passed on.",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered pattern:
    /// `..=` (inclusive range) generates an extra `array_set v_loop`
    /// in a post-loop block, which forward-threads the back-edge value
    /// (`v0`) into a downstream block param (`v25`). The walk reaches
    /// a `array_get v25` and would flag it because UF unifies `v25`
    /// with the source `v24` via the post-loop forward edge.
    ///
    /// At runtime `v25` is always `v0` (or only `v1` on a path where
    /// the array_set never fired), and the `inc_rc v0` inside the
    /// loop body ensures `RC(v0) ≥ 2` for every iteration where the
    /// array_set is on `v0`. The verifier doesn't reason path-
    /// sensitively, but `inc_rc v0` *is* on an alias (`v0 != v24`)
    /// that participates in the loop's back-edge — that's a deliberate
    /// codegen signal that aliasing is being managed. The
    /// back-edge-participant relaxation in `some_inc_rc_precedes`
    /// accepts on that basis.
    #[test]
    fn end_to_end_inc_rc_on_back_edge_participant_alias_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u1; 3], v1: [u1; 3]):
                jmp b1(i16 -9421, v1)
              b1(v5: i16, v24: [u1; 3]):
                v8 = lt v5, i16 -9417
                jmpif v8 then: b2(), else: b3()
              b2():
                v14 = array_set v24, index u32 0, value u1 0
                inc_rc v0
                v16 = unchecked_add v5, i16 1
                jmp b1(v16, v0)
              b3():
                jmpif u1 1 then: b4(), else: b5(v24)
              b4():
                v21 = array_set v24, index u32 0, value u1 0
                inc_rc v0
                jmp b5(v0)
              b5(v25: [u1; 3]):
                v23 = array_get v25, index u32 0 -> u1
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "inc_rc v0 is on an alias of the source v24 (not v24 itself) that's also a back-edge arg to v24 via the b2->b1 back-edge — a codegen signal that loop aliasing is being managed",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered pattern: two
    /// array-typed function entry parameters get unified into the same
    /// alias class by the UF because they both flow into the same
    /// downstream block parameter — one via a forward edge into the
    /// loop header, the other via a back-edge that re-seeds the loop
    /// variable with the second entry parameter (the user-source-level
    /// equivalent of `c = b` at the bottom of an outer loop iteration).
    /// The walk then finds an aliased `array_get` of the *other* entry
    /// param on a forward path from the `array_set`, with no
    /// dominating `inc_rc`. The frontend trusts that distinct entry
    /// parameters point at distinct caller-side storage (and the only
    /// inc_rc it emits is on the back-edge, protecting iteration 2+);
    /// the verifier mirrors that trust by treating ≥ 2 array entry
    /// parameters in the alias-set as if a virtual `inc_rc` preceded
    /// the array_set at function entry.
    #[test]
    fn end_to_end_two_entry_array_params_cross_aliased_via_back_edge_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v_b: [i8; 4], v_c: [i8; 4]):
                jmp b1(v_c)
              b1(v_loop: [i8; 4]):
                jmpif u1 1 then: b3(), else: b2()
              b2():
                return
              b3():
                v6 = array_get v_b, index u32 0 -> i8
                v7 = array_set v_loop, index u32 0, value i8 0
                inc_rc v_b
                jmp b1(v_b)
            }"#;
        assert_verifier_accepts_because(
            src,
            "v_b and v_c are both array-typed entry parameters unified into the loop-header param's class — at runtime they point at distinct caller-side storage, so cross-arg aliasing through the UF is not a real hazard",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered minimal shape:
    /// two array-typed function entry parameters are unified into the
    /// same alias class by a *forward* if-else sibling join (no loops,
    /// no inc_rc anywhere). The walk flags an `array_get` of the other
    /// entry param inside the array_set's own block — *before* the walk
    /// reaches the join where the UF conflation actually happens.
    /// Source-level shape:
    ///
    /// ```ignore
    /// fn main(a, mut b, c) -> Field {
    ///     if c == 0 { b[1] = b[0]; b[0] = 20; b = a; b[1] }
    ///     else { c }
    /// }
    /// ```
    ///
    /// Distinct entry parameters point at distinct caller-side storage by
    /// Brillig calling convention; their conflation through a downstream
    /// join is a UF over-approximation, not a real hazard. The verifier
    /// recognizes this by accepting any `array_set` whose alias-set
    /// contains two or more array-typed entry parameters.
    #[test]
    fn end_to_end_two_entry_array_params_cross_aliased_via_forward_sibling_join_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [Field; 2], v1: [Field; 2], v3: Field):
                v5 = eq v3, Field 0
                jmpif v5 then: b1(), else: b2()
              b1():
                v11 = array_get v1, index u32 0 -> Field
                v13 = array_set v1, index u32 1, value v11
                v16 = array_set v13, index u32 0, value Field 20
                v18 = array_get v0, index u32 1 -> Field
                jmp b3(v18, v0)
              b2():
                jmp b3(v3, v1)
              b3(v19: Field, v20: [Field; 2]):
                return v19
            }"#;
        assert_verifier_accepts_because(
            src,
            "v0 and v1 are both array-typed function entry params unified into b3.v20's class via the if-else sibling join; at runtime they point at distinct caller-side storage, so the cross-arg aliasing through UF is not a real hazard",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered pattern: an
    /// `array_set` whose source is unified with a *global* through a
    /// forward (non-loop) sibling-join, and the codegen emits an `inc_rc`
    /// on the global right before the forward jump that threads the
    /// global into the join's block parameter. There is no loop, no
    /// back-edge, and only one array entry parameter (so the multi-entry
    /// heuristic doesn't apply). The `inc_rc` postdates the array_set in
    /// the same block, so the RPO-precedence check doesn't fire either.
    /// Source-level shape:
    ///
    /// ```ignore
    /// fn func_1(mut b: [[bool; 3]; 4]) {
    ///     if b[0][0] {
    ///         b[0] = b[3];
    ///         b = G_A;       // re-seed `b` with a global
    ///     }
    ///     b[0] = b[1];
    /// }
    /// ```
    ///
    /// The codegen's `inc_rc g_a` before the `jmp b_join(g_a)` is the
    /// signal we trust — the frontend is deliberately managing the
    /// aliasing introduced by threading `g_a` forward into the join's
    /// block parameter. The relaxation that already accepts this for
    /// loop back-edges should also accept forward-edge participants.
    #[test]
    fn end_to_end_inc_rc_on_forward_edge_participant_alias_is_accepted() {
        let src = r#"
            g0 = make_array [u1 0, u1 0, u1 0, u1 0] : [u1; 4]

            brillig(inline) fn main f0 {
              b0(v0: [u1; 4]):
                v2 = array_get v0, index u32 0 -> u1
                jmpif v2 then: b1(), else: b2(v0)
              b1():
                v4 = array_set v0, index u32 0, value u1 1
                inc_rc g0
                jmp b2(g0)
              b2(v5: [u1; 4]):
                v7 = array_get v5, index u32 1 -> u1
                v9 = array_set v5, index u32 0, value u1 1
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "inc_rc g0 sits on an alias-set member that's threaded into v5 via a forward jump — a codegen signal that the frontend is deliberately managing the aliasing, analogous to the existing back-edge-participant relaxation but for forward edges",
        );
    }

    /// Counterpart of
    /// [`end_to_end_inc_rc_on_forward_edge_participant_alias_is_accepted`]
    /// with `inc_rc g0` stripped. The codegen threads the global `g0`
    /// from one branch and the function arg `v0` from the other into the
    /// join's block parameter `v5`. The `array_set v0` lives only on the
    /// branch that subsequently passes `g0` (not `v0`) to the join.
    ///
    /// # Why it's safe at runtime
    ///
    /// - **then-branch (b0 → b1 → b2):** `array_set v0` may mutate `v0`'s
    ///   storage in place. The forward jump rebinds `v5 := g0`, so the
    ///   downstream reads of `v5` hit `g0`'s storage, not the mutated
    ///   `v0`.
    /// - **else-branch (b0 → b2):** `v0` is never mutated. `v5 := v0`,
    ///   reads of `v5` see the original `v0`.
    ///
    /// On the path where the mutation happens, the value threaded
    /// forward into `v5` is *not* the one that got mutated.
    ///
    /// # Why the backward-alias-set analysis catches this
    ///
    /// The full UF would unify `v0 ≈ v5 ≈ g0` and incorrectly flag
    /// `array_get v5` as an aliased read of `v0`. The backward walk
    /// follows param←arg edges directionally: from `v0`'s perspective
    /// (an entry param with no predecessors), the backward set is just
    /// `{v0}`. The forward walk's add-on-edge rule then watches for
    /// `v0` to be threaded into a downstream param — but the b1→b2
    /// edge passes `g0`, not `v0`, so no param gets added. The walk
    /// terminates without flagging.
    #[test]
    fn end_to_end_forward_edge_sibling_join_without_inc_rc_is_accepted() {
        let src = r#"
            g0 = make_array [u1 0, u1 0, u1 0, u1 0] : [u1; 4]

            brillig(inline) fn main f0 {
              b0(v0: [u1; 4]):
                v2 = array_get v0, index u32 0 -> u1
                jmpif v2 then: b1(), else: b2(v0)
              b1():
                v4 = array_set v0, index u32 0, value u1 1
                jmp b2(g0)
              b2(v5: [u1; 4]):
                v7 = array_get v5, index u32 1 -> u1
                v9 = array_set v5, index u32 0, value u1 1
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "backward-alias-set of v0 is {v0}; the b1→b2 edge passes g0 (not v0), so the add-on-edge rule doesn't pull v5 into the use-set; reads of v5 are unrelated to v0's storage",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered pattern: an
    /// `array_set v0` in one branch is followed in the same block by an
    /// `inc_rc w` of a *different* value `w` that the backward
    /// alias-set walk places in `v0`'s alias-set (because both `v0` and
    /// `w` flow into the same `b3` block parameter from two branches).
    /// The `inc_rc` is a ref-count bump, not a content read — `inc_rc`
    /// and `dec_rc` operands are excluded from `array_operand_uses`, so
    /// the walk skips them. Symmetric to the `Instruction::ArraySet` /
    /// `Call` "non-aliasing-result" filter: an instruction whose
    /// semantics don't read pre-mutation storage is not a hazard.
    #[test]
    fn end_to_end_array_set_followed_by_inc_rc_of_aliased_param_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 1], v1: [u32; 1]):
                jmpif u1 1 then: b1(), else: b2()
              b1():
                jmp b3(v0)
              b2():
                v5 = array_set v0, index u32 0, value u32 99
                inc_rc v1
                jmp b3(v1)
              b3(v6: [u32; 1]):
                return v6
            }"#;
        assert_verifier_accepts_because(
            src,
            "`inc_rc v1` after the array_set is a ref-count op, not a read of array contents",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered shape: a
    /// `make_array` defined in the same block as (and *after*) the
    /// `array_set`, whose result feeds the loop-header parameter on the
    /// loop's back-edge. The make_array is iteration-local (back-edge
    /// arg), so the `iteration_local_make_arrays` filter drops it from
    /// the alias-set: the per-arg kill at the back-edge then sees the
    /// arg ∉ use_set and correctly drops the loop-header parameter, so
    /// the walk terminates without flagging the loop-body reads.
    #[test]
    fn end_to_end_loop_back_edge_with_post_array_set_make_array_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 4]):
                jmp b1(v0, u32 0)
              b1(v1: [u32; 4], v2: u32):
                jmpif u1 1 then: b3(), else: b2()
              b2():
                return
              b3():
                call f1(v1)
                v6 = array_set v1, index u32 3, value u32 0
                v7 = make_array [u32 0, u32 0, u32 0, u32 0] : [u32; 4]
                v9 = add v2, u32 1
                jmp b1(v7, v9)
            }
            brillig(inline) fn observer f1 {
              b0(v0: [u32; 4]):
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "v7 (make_array) is defined after the array_set in the same block — it can't represent the mutated storage, and the loop-header param v1 is bound to v7 on the back-edge",
        );
    }

    /// End-to-end regression for an AST-fuzzer-discovered shape: a
    /// `make_array` defined in the outer-loop body *before* the inner
    /// loop's `array_set`, whose result is threaded back to the outer
    /// loop's header on the back-edge. The outer header's parameter
    /// would acquire the make_array via the backward chain (outer
    /// header → outer back-edge → make_array), pulling the make_array
    /// into the inner array_set's alias-set. The per-arg kill on the
    /// outer back-edge would then see the `make_array` in the use-set
    /// and refuse to kill the outer header parameter, letting the
    /// walk reach an earlier-in-source `array_get` — even though at
    /// runtime the parameter is rebound to a fresh `make_array` on
    /// every back-edge crossing, so the iteration-aliasing is illusory.
    ///
    /// The `iteration_local_make_arrays` filter drops a `make_array`
    /// result that appears on a loop back-edge: the make_array always
    /// allocates fresh top-level storage, so it
    /// can't represent the pre-mutation storage of any array_set source.
    /// (Aliasing through *nested-array elements* of a `make_array` is a
    /// documented gap — see the module-level docs.)
    #[test]
    fn end_to_end_nested_loop_outer_back_edge_with_pre_array_set_make_array_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u64; 1]):
                jmp b1(v0, u32 0)
              b1(v22: [u64; 1], v23: u32):
                v5 = eq v23, u32 0
                jmpif v5 then: b3(), else: b4()
              b2():
                return
              b3():
                jmp b2()
              b4():
                v8 = add v23, u32 1
                v11 = array_get v22, index u32 0 -> u64
                v12 = make_array [v11] : [u64; 1]
                jmp b6(v22, u32 0)
              b5():
                jmp b1(v12, v8)
              b6(v24: [u64; 1], v25: u32):
                v16 = eq v25, u32 3
                jmpif v16 then: b8(), else: b9()
              b7():
                jmp b5()
              b8():
                jmp b7()
              b9():
                v18 = add v25, u32 1
                v21 = array_set v24, index u32 0, value u64 0
                jmp b10()
              b10():
                jmp b6(v21, v18)
            }"#;
        assert_verifier_accepts_because(
            src,
            "v12 (make_array) on the outer back-edge to v22 is fresh storage by construction; the outer parameter v22 is rebound to v12 each outer iteration, so reading v22 in the next iteration's b4 observes the fresh array, not the inner array_set's mutated storage",
        );
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
        assert_verifier_accepts_because(
            src,
            "loop exit reads `v2`, which is rebound on the back-edge to the chained array_set's result; not a hazard",
        );
    }

    /// End-to-end: the poseidon-style "array_set then call returning a
    /// fresh array threaded back via the loop's back-edge". Reduced from
    /// `bench_2_to_17` (stdlib's `poseidon2::hash_internal`):
    ///
    /// - `v8 = array_set v2, idx 0, …` in the loop body.
    /// - `v9 = call permute(v8)` returns a *fresh* array.
    /// - Back-edge `jmp b1(v9, …)` threads the fresh call result back to
    ///   `v2` for the next iteration.
    /// - After the loop, `array_get v2, idx 0` reads the final loop state.
    ///
    /// Without the call-result filter, the backward walk would pull `v9`
    /// into `v2`'s alias-set via the back-edge, the per-arg kill at the
    /// back-edge would see `v9` still in the use-set, and the loop-exit
    /// `array_get v2` would be flagged as an aliased read. The
    /// call-result filter drops `v9` from the alias-set, so the per-arg
    /// kill fires and the walk doesn't reach the post-loop read.
    #[test]
    fn end_to_end_loop_array_set_then_call_returning_fresh_array_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2], v1: u32):
                jmp b1(v0, u32 0)
              b1(v2: [u32; 2], v3: u32):
                v5 = lt v3, v1
                jmpif v5 then: b2(), else: b3()
              b2():
                v8 = array_set v2, index u32 0, value u32 99
                v9 = call f1(v8) -> [u32; 2]
                v11 = add v3, u32 1
                jmp b1(v9, v11)
              b3():
                v13 = array_get v2, index u32 0 -> u32
                return v13
            }
            brillig(inline) fn permute f1 {
              b0(v0: [u32; 2]):
                return v0
            }"#;
        assert_verifier_accepts_because(
            src,
            "loop exit reads `v2` but the back-edge threads a fresh call result, breaking the alias chain at the call boundary",
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
        assert_verifier_accepts_because(
            src,
            "load result is re-executed each iteration; the cycle's array_get is not a hazard",
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
    /// `v2`, `v4`, `v6` are instruction results (not block parameters),
    /// so the backward walk has nothing to chase from `v6` — `v6`'s
    /// alias-set is just `{v6}`. The walk never reaches `v0` via the
    /// chain, and the absence of `inc_rc v4` correctly surfaces the
    /// violation. (Note: this test uses `last_array_set`, whose source
    /// is `v4`; the third array_set's source is `v4`, whose alias-set
    /// is `{v4}` — see the assertion below.)
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

        let ArraySetSite { source, alias_set, .. } =
            last_array_set(function, &ctx).expect("array_set present");

        // Only the source itself — no walking into the upstream chain links
        // (v2, v0) or any block-parameter predecessors.
        assert_eq!(alias_set.iter().copied().collect::<Vec<_>>(), vec![source]);
    }

    /// Two things at once:
    ///   1. The backward alias-set follows block-parameter edges — for
    ///      the `array_set` in `b5`, `v2`'s pre-header source (`v0`) lands
    ///      in the same set.
    ///   2. The fixed-point terminates on cycles. `b1`'s parameter `v2` has
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

        let ArraySetSite { source, alias_set, .. } =
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

        for ArraySetSite { block, idx, source, alias_set, .. } in &array_sets {
            let precedes = ctx.some_inc_rc_precedes(alias_set, *source, *block, *idx);
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
        let ArraySetSite { block, idx, instruction_id, alias_set, write_index_const, .. } =
            first_array_set(function, &ctx).expect("array_set present");

        let has_use = ctx
            .find_reachable_aliased_use(&alias_set, instruction_id, block, idx, write_index_const)
            .is_some();
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
        let ArraySetSite { block, idx, instruction_id, alias_set, write_index_const, .. } =
            first_array_set(function, &ctx).expect("array_set present");

        let has_use = ctx
            .find_reachable_aliased_use(&alias_set, instruction_id, block, idx, write_index_const)
            .is_some();
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
        let ArraySetSite { block, idx, instruction_id, alias_set, write_index_const, .. } =
            first_array_set(function, &ctx).expect("array_set present");

        let has_use = ctx
            .find_reachable_aliased_use(&alias_set, instruction_id, block, idx, write_index_const)
            .is_some();
        assert!(
            !has_use,
            "no aliased read exists; the walk must terminate and return false despite re-entering b3 with non-overlapping use-sets"
        );
    }

    /// **Documents a known false negative.** When a single jmp passes the
    /// same value to multiple parameter positions (e.g. `jmp b1(v0, v0)`),
    /// the resulting sibling parameters refer to the same runtime storage,
    /// but the backward alias-set walk doesn't see this: it follows each
    /// param's predecessors directionally, so `v1`'s backward set is
    /// `{v1, v0}` and `v2`'s is `{v2, v0}` — neither contains the other.
    ///
    /// At runtime, `array_set v1` mutates `v0`'s storage in place if RC=1,
    /// and `array_get v2` would observe the mutation. The verifier should
    /// flag this, but it doesn't.
    ///
    /// The previous full-UF formulation caught this case (sibling args
    /// land in the same equivalence class via the shared parameter), but
    /// at the cost of conflating distinct caller-side storages at join
    /// points, which produced several fuzzer-found false positives. The
    /// trade-off — give up sibling-same-value detection to avoid
    /// path-merge over-approximation — is documented in the module-level
    /// docs.
    ///
    /// This test pins down the current behavior. If a future change
    /// makes the analysis precise enough to catch this shape, flip the
    /// assertion to `assert_verifier_rejects`.
    #[test]
    fn end_to_end_sibling_args_to_same_value_is_false_negative() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: [u32; 2]):
                jmp b1(v0, v0)
              b1(v1: [u32; 2], v2: [u32; 2]):
                v5 = array_set v1, index u32 0, value u32 99
                v7 = array_get v2, index u32 0 -> u32
                return
            }"#;
        assert_verifier_accepts_because(
            src,
            "sibling-same-value blind spot: v1 and v2 share storage at runtime (both bound to v0 by b0's jmp), but backward-alias-set walks each param's predecessors directionally and doesn't co-mingle siblings",
        );
    }

    /// **Documents the same known false negative across a block boundary.**
    /// Same sibling pattern as
    /// [`end_to_end_sibling_args_to_same_value_is_false_negative`], but
    /// the aliased read is in a downstream block reached by threading
    /// the sibling param `v2` forward. The forward walk's add-on-edge
    /// rule does propagate `v2 → v7` correctly *once `v2` is in the
    /// use-set*, but `v2` never enters the use-set in the first place
    /// (it's not in `v1`'s backward alias-set), so the chain doesn't
    /// fire.
    #[test]
    fn end_to_end_sibling_args_across_jmp_is_false_negative() {
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
        assert_verifier_accepts_because(
            src,
            "sibling-same-value blind spot extended across a forward jmp: v2 isn't in v1's backward alias-set, so the add-on-edge rule never pulls v7 into the use-set either",
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
        let ArraySetSite { block, idx, instruction_id, alias_set, write_index_const, .. } =
            first_array_set(function, &ctx).expect("array_set present");

        let has_use = ctx
            .find_reachable_aliased_use(&alias_set, instruction_id, block, idx, write_index_const)
            .is_some();
        assert!(
            !has_use,
            "b2's v5 is rebound to v4 (the array_set's result, excluded from alias-set), so it is killed and array_get v5 is not aliased"
        );
    }

    /// A located `array_set` plus everything the per-array_set verifier
    /// checks would need. Returned by [`find_array_sets`] / [`first_array_set`]
    /// / [`last_array_set`] so each test reads one struct rather than a
    /// six-element tuple.
    struct ArraySetSite {
        block: super::BasicBlockId,
        idx: usize,
        instruction_id: super::InstructionId,
        source: ValueId,
        alias_set: im::HashSet<ValueId>,
        /// The `array_set`'s index when it is a numeric constant. `None`
        /// indicates a dynamic index, in which case the verifier
        /// conservatively flags every aliased use.
        write_index_const: Option<super::FieldElement>,
    }

    fn find_array_sets(function: &Function, ctx: &Context<'_>) -> Vec<ArraySetSite> {
        let mut out = Vec::new();
        for block_id in function.reachable_blocks() {
            for (idx, instruction_id) in function.dfg[block_id].instructions().iter().enumerate() {
                if let Instruction::ArraySet { array, index, .. } = function.dfg[*instruction_id] {
                    out.push(ArraySetSite {
                        block: block_id,
                        idx,
                        instruction_id: *instruction_id,
                        source: array,
                        alias_set: ctx.alias_set_for(array),
                        write_index_const: function.dfg.get_numeric_constant(index),
                    });
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
