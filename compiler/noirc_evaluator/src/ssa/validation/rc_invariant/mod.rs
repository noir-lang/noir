//! Verifies the implicit invariant that Brillig SSA must satisfy around
//! `array_set` and reference counts.
//!
//! # The invariant
//!
//! In Brillig, `array_set vX, i, x` may modify `vX`'s storage in place at runtime
//! when `vX`'s reference count is 1. SSA-level semantics still says `vX` is unchanged
//! and the `array_set` produces a fresh value; the in-place mutation is a runtime
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
//!    storage *at the `array_set`'s program point* by walking block-parameter →
//!    predecessor-arg edges backward to a fixed point. Only aliasing introduced
//!    by the values that flow *into* `vX`'s binding is included. Filtered to
//!    drop values that can't represent pre-mutation storage: `array_set` /
//!    `Call` results (always fresh), iteration-local fresh results (`MakeArray`
//!    or `Call` on back-edge args), instruction results defined in the
//!    `array_set`'s own block at an index after the `array_set` (they can land in
//!    the backward set through a forward-then-back round-trip, but don't exist
//!    as storage at the `array_set`'s program point yet), and **swap-excluded
//!    siblings** — when `vX` is a loop-header parameter swapped onto a sibling
//!    parameter that is freshly re-allocated each iteration, the sibling is a
//!    distinct per-iteration storage and is dropped (see
//!    [`Context::swap_excluded_aliases`]).
//! 2. **Fast full-accept.** Accept the `array_set` outright (skipping the
//!    forward walk) when some `inc_rc` on an alias-set member protects it on
//!    every path by a cheap test: an `inc_rc` earlier in the `array_set`'s own
//!    block, in a block that **dominates** it, or — for a non-source alias —
//!    on a loop **back-edge** (the back-edge-participant relaxation, where the
//!    frontend is deliberately managing iteration aliasing). See
//!    [`Context::some_inc_rc_precedes`].
//! 3. **Per-member coverage → use-set.** Otherwise narrow the alias-set to the
//!    members that are *not* provably protected, and seed the forward walk
//!    with only those ([`Context::unprotected_aliases`]). Coverage is per
//!    member: thread the source's storage backward through the CFG
//!    ([`Context::compute_uncovered_values`]) and drop a member when, on every
//!    path where it can be the source's storage at the `array_set`, that storage
//!    is RC-bumped by an `inc_rc` or is an iteration-local fresh allocation (a
//!    `MakeArray`/`Call` re-executed each loop iteration). Dropping covered
//!    members is what stops the forward walk from flagging a read of, say, an
//!    `inc_rc`'d sibling on a path where the `array_set` never mutates that
//!    storage. An empty use-set means every member is protected — accept.
//! 4. **Protected-participant filter.** A second narrowing inside the forward
//!    walk: drop every use-set member (other than the source) that both
//!    carries its own `inc_rc` and is a loop back-edge *participant* (it
//!    flows — directly or through forward edges — into a back-edge arg
//!    position). Being in the alias-set means the value flows *into* the
//!    source, so combined with its back-edge participation the `inc_rc` is
//!    loop-carried: it runs before the value crosses the back-edge that
//!    re-binds it onto the source, so by the time the value's storage
//!    equals the source's it is RC ≥ 2 and the `array_set` copies. Reads of
//!    it therefore can't observe an in-place mutation. The gate is **per
//!    value** on that value's own `inc_rc`: a back-edge position fed by an
//!    `inc_rc`'d value on one predecessor edge and an unprotected value on
//!    another drops only the protected value, leaving the unprotected one
//!    for the walk to flag. Restricting to alias-set members is the
//!    soundness guard — a value that merely *receives* the source's storage
//!    (a forward successor, not in the source's backward set) has its
//!    `inc_rc` run *after* the `array_set` and so is not protected; it stays
//!    in the use-set. This is what lets the verifier accept the latch-block
//!    shape (an `inc_rc v` placed before `v` is threaded *forward* into the
//!    latch that then closes the loop) without the unsoundness of crediting
//!    an `inc_rc` to a value the `array_set` actually mutates first.
//! 5. **Forward walk.** Walk the CFG forward from the `array_set` with the
//!    use-set as the initial set of live aliases. At each block-parameter
//!    crossing we both **kill** params that the predecessor rebinds to a
//!    non-alias and **add** params whose arg is still an alias (so alias
//!    propagation stays accurate as the walk crosses joins and loops). The
//!    walk maintains two additional pieces of state:
//!
//!    - **`derived`**, the set of values that may share the source's storage
//!      through transitive in-place chain mutations. Seeded with the
//!      `array_set`'s own result; extended by every later `array_set` whose
//!      `array` operand is already in `derived`.
//!    - **`tainted_indices`**, the storage positions any chain link may
//!      already have written. Seeded with the `array_set`'s own write index
//!      (when constant) and widened by chain-link writes; collapsed to "all
//!      positions" if any chain link uses a dynamic index.
//!
//!    An `array_get` on a use-set member is a hazard iff its read index is
//!    covered by `tainted_indices`. An `array_set` on a use-set member is
//!    index-aware too: it produces a copy of the source with one index
//!    overwritten, so it observes (copies forward) only the indices it does
//!    *not* write — a hazard iff some `tainted_indices` position differs
//!    from its write index (a same-index write overwrites the mutation and
//!    is not a hazard; it instead extends the chain). All other uses are
//!    always hazards. An `inc_rc v` with `v ∈ use_set ∪ derived` lifts the
//!    storage's RC,
//!    so chain writes after it run on fresh storage — `derived` is cleared
//!    at that point. `tainted_indices` is *not* cleared: prior chain writes
//!    have already mutated the source's storage.
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
//!   down as documented false negatives.
//! - **Nested-array `MakeArray`**, **`IfElse` on arrays**, **non-inlined
//!   `Call` returns**, and **`Store`/`Load` on ineligible (nested-ref)
//!   allocates** are likewise not tracked.
//! - **Iteration-local fresh array read back within one iteration.** The
//!   freshness wall (step 3) credits a `MakeArray`/`Call` that shares a loop
//!   with the `array_set` as distinct per-iteration storage, which is sound
//!   only because the result is normally threaded forward rather than the
//!   freshly-allocated source being re-read after its in-place mutation. A
//!   hand-written loop body that allocates an array, mutates it in place, and
//!   then reads the *same* value back within the same iteration would not be
//!   flagged. The frontend threads the `array_set` result instead of
//!   re-reading the source, so it does not emit this shape; crediting
//!   freshness only ever *accepts* more, so this can only be a false negative,
//!   never a false positive.

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use std::collections::BTreeSet;

use acvm::FieldElement;

use crate::ssa::{
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
};

pub(crate) mod array_set;
pub(crate) mod call;

/// Pre-computed indices over a Brillig function. The verifier's per-array_set
/// checks read from these structures rather than re-scanning the function.
struct Context<'f> {
    function: &'f Function,
    /// Control-flow graph. Used by [`Context::some_inc_rc_precedes`] to walk
    /// predecessors backward when checking whether the `inc_rc`-carrying
    /// blocks collectively cover every path to an `array_set`.
    cfg: ControlFlowGraph,
    /// Dominator tree. Used by [`Context::some_inc_rc_precedes`] as a fast
    /// path: a single `inc_rc` block that dominates the `array_set` covers
    /// every path without the backward value walk.
    dom_tree: DominatorTree,
    /// Array values that allocate fresh storage at their definition
    /// (`make_array` / `Call` results). Used as a freshness wall by
    /// [`Context::compute_uncovered_values`].
    fresh_array_values: HashSet<ValueId>,
    /// The block set of each loop in the function. The freshness wall only
    /// credits a fresh allocation when its defining block and the `array_set`'s
    /// block lie in a common loop — i.e. the allocation re-executes every
    /// iteration that reaches the `array_set`, so its storage is distinct per
    /// iteration. A one-time allocation outside any shared loop is mutated in
    /// place exactly once, so reading it back is a genuine hazard.
    loop_blocks: Vec<BTreeSet<BasicBlockId>>,
    /// For each array-typed value `V`, the set of values that may share
    /// `V`'s storage **at `V`'s program point** — the source itself plus
    /// anything that flows backward into it through block-parameter →
    /// predecessor-arg chains. Computed to a fixed point.
    ///
    /// Values absent from this map have an implicit singleton class
    /// `{v}` (typically instruction results that aren't passed as args
    /// anywhere they'd matter — handled in [`Context::alias_set_for`]).
    ///
    /// The walk follows the directed `param ← arg` edges, so two values
    /// that flow into different parameters of the same block are *not*
    /// siblings of each other under backward aliasing — neither is
    /// reachable from the other along param→arg edges. This is what
    /// keeps path-merge over-approximation at join points out of the
    /// alias-set.
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
    /// Back-edge args that re-allocate distinct storage every iteration:
    /// `MakeArray` results (re-executes each iteration) and `Call` results
    /// (the callee allocates fresh). `array_set` results are excluded —
    /// they may mutate in place, so they aren't guaranteed-fresh. Two uses:
    ///
    /// - [`Context::alias_set_for`] drops such values from the alias-set:
    ///   a back-edge that puts one in a loop-header parameter's class is
    ///   conflating distinct runtime storages across iterations. (`Call`
    ///   results are also dropped there by
    ///   [`Context::non_aliasing_array_values`]; the overlap is harmless.)
    /// - the swap freshening guard in [`Context::new`] requires the
    ///   swapped-out sibling to be one of these — see
    ///   [`Context::swap_excluded_aliases`].
    ///
    /// Values that are *not* back-edge args are kept in the alias-set:
    /// they represent a one-time allocation whose storage the `array_set`
    /// may mutate in place.
    iteration_local_fresh: HashSet<ValueId>,
    /// `inc_rc value` instructions indexed by their operand. Each entry is
    /// the `(block, instruction-position-within-block)` of one `inc_rc`.
    inc_rc_locations: HashMap<ValueId, Vec<(BasicBlockId, usize)>>,
    /// Values that appear at least once as a jmp/jmpif arg on a loop
    /// back-edge. Used by both:
    ///
    /// - [`Context::iteration_local_fresh`] (the back-edge args that are
    ///   `make_array` or `Call` results, computed in [`Context::new`]); and
    /// - the **back-edge-participant relaxation** in
    ///   [`Context::some_inc_rc_precedes`]: an `inc_rc` on a non-source
    ///   alias that's also a back-edge arg is taken as a codegen
    ///   signal that the frontend is deliberately managing
    ///   iteration aliasing, regardless of program-point order.
    ///
    /// Forward-edge cases (e.g. re-seeding a value with a global on a
    /// non-loop branch) are handled precisely by [`Context::backward_aliases`]
    /// — the forward-edge value simply isn't in the source's backward
    /// set in the first place — so this signal only needs to cover
    /// loop back-edges.
    back_edge_args: HashSet<ValueId>,
    /// The backward-alias closure of [`Context::back_edge_args`]: every
    /// value that flows (directly or through forward block-parameter
    /// edges) into a loop back-edge arg position. A value `v` is a
    /// back-edge *participant* when its storage can become a loop-header
    /// parameter's storage across a back-edge — even when `v` itself is
    /// not the literal arg, but reaches one through a forward edge first
    /// (e.g. `inc_rc v; jmp latch(v)` then `latch: jmp header(v)`, where
    /// only the latch's param is the literal back-edge arg).
    ///
    /// Used by the **protected-participant filter** in
    /// [`Context::find_reachable_aliased_use`]: a value that is in the
    /// `array_set` source's alias-set (so it flows *into* the source),
    /// is a back-edge participant, and carries its own `inc_rc` is
    /// RC ≥ 2 whenever it equals the source's storage (the `inc_rc` is
    /// loop-carried — it runs before the value crosses the back-edge onto
    /// the source), so reads of it can't observe an in-place mutation and
    /// it is dropped from the forward walk's use-set.
    ///
    /// The filter is gated **per value** on that value's own `inc_rc`:
    /// membership in this set alone never exonerates anything. This is
    /// what keeps it sound where widening the relaxation to "some
    /// participant has an `inc_rc`" would not — a back-edge position fed
    /// by an `inc_rc`'d value on one predecessor edge and an unprotected
    /// value on another only drops the protected value, leaving the
    /// unprotected one for the forward walk to flag.
    back_edge_participants: HashSet<ValueId>,
    /// **Swap exclusions.** For a loop-header parameter `P`, the set of
    /// sibling parameters `Q` that `P` is rebound to on the loop
    /// back-edge (`P ← Q`, the array-variable swap `c4 = c3`) while `Q`
    /// is simultaneously rebound to a freshly-allocated, iteration-local
    /// value — a `make_array` (`c3 = [..]`) or a `Call` result
    /// (`c3 = f()`), but never an `array_set` result (which may be an
    /// in-place mutation, not a fresh allocation). Such a `Q` is a *distinct
    /// per-iteration storage* that `P` only *becomes* in a later
    /// iteration — never the storage this iteration's `array_set` on `P`
    /// mutates (the swap rotates a fresh allocation into `P` while the
    /// mutated storage is left behind, dead). `Q` is dropped from `P`'s
    /// alias-set in [`Context::alias_set_for`], which makes the forward
    /// walk both **kill** `P` where the back-edge rebinds it to `Q` and
    /// decline to **add** a fresh alias of `Q` past any later edge (the
    /// inclusive-range *peel* re-runs the same swap on a *forward* exit
    /// edge — excluding `Q` at the seed covers that edge too, which a
    /// kill keyed on the back-edge alone could not).
    ///
    /// Guarded by a **back-edge-only** check: if `Q` also flows into `P`
    /// on a forward edge, `P` and `Q` can already alias *within* an
    /// iteration (the `array_set` would mutate `Q`'s live storage), so the
    /// exclusion would be unsound and is not recorded. The freshening
    /// requirement on `Q` is the second guard — a loop-invariant `Q`
    /// swapped into `P` would make `P_k = Q_{k-1} = Q_k` and genuinely
    /// alias.
    ///
    /// **Forward propagation.** After recording, exclusions are pushed
    /// forward across non-back-edge parameter edges to a fixed point: on a
    /// forward edge `R ← P`, `R` *is* `P` within the same iteration, so `R`
    /// inherits `P`'s exclusions. This carries an exclusion from a swapped
    /// loop-header parameter onto a successor parameter seeded from it — e.g.
    /// an inner-loop header (the `array_set`'s source) fed from the swapped
    /// outer-loop parameter — so `Q` is dropped for the inner source too.
    /// Back-edges are excluded from propagation: across one, `R` is a *prior*
    /// iteration's `P`, a distinct storage.
    swap_excluded_aliases: HashMap<ValueId, HashSet<ValueId>>,
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
        let loops = Loops::find_all(function, LoopOrder::InsideOut);
        let back_edges: HashSet<(BasicBlockId, BasicBlockId)> =
            loops.yet_to_unroll.iter().map(|l| (l.back_edge_start, l.header)).collect();
        // The block set of each loop, used by the freshness wall in
        // [`Context::compute_uncovered_values`] to decide whether a fresh
        // allocation re-executes on every iteration that reaches an array_set.
        let loop_blocks: Vec<BTreeSet<BasicBlockId>> =
            loops.yet_to_unroll.iter().map(|l| l.blocks.clone()).collect();

        let mut inc_rc_locations: HashMap<ValueId, Vec<(BasicBlockId, usize)>> = HashMap::default();
        let mut array_value_defs: HashMap<ValueId, (BasicBlockId, usize)> = HashMap::default();
        let mut non_aliasing_array_values: HashSet<ValueId> = HashSet::default();
        let mut make_array_values: HashSet<ValueId> = HashSet::default();
        let mut call_result_values: HashSet<ValueId> = HashSet::default();
        let mut back_edge_args: HashSet<ValueId> = HashSet::default();
        // For each destination block, the list of `(predecessor, args)`
        // pairs collected from terminators. Used to drive the backward
        // alias-set fixed-point: each block parameter's set is the union
        // of its arg at that position from every incoming edge.
        let mut incoming_edges: HashMap<BasicBlockId, Vec<(BasicBlockId, Vec<ValueId>)>> =
            HashMap::default();

        // Single pass over every reachable block to populate
        // per-instruction indices and per-block incoming-edge tables.
        for &block_id in &rpo {
            for (idx, instruction_id) in function.dfg[block_id].instructions().iter().enumerate() {
                let instruction = &function.dfg[*instruction_id];

                if let Instruction::IncrementRc { value } = instruction {
                    inc_rc_locations.entry(*value).or_default().push((block_id, idx));
                }

                let is_call = matches!(instruction, Instruction::Call { .. });
                let is_non_aliasing =
                    is_call || matches!(instruction, Instruction::ArraySet { .. });
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
                        if is_call {
                            call_result_values.insert(result);
                        }
                    }
                }
            }

            if let Some(terminator) = function.dfg[block_id].terminator() {
                match terminator {
                    TerminatorInstruction::Jmp { destination, arguments, .. } => {
                        incoming_edges
                            .entry(*destination)
                            .or_default()
                            .push((block_id, arguments.clone()));
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
                            .push((block_id, then_arguments.clone()));
                        incoming_edges
                            .entry(*else_destination)
                            .or_default()
                            .push((block_id, else_arguments.clone()));
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

        let backward_aliases = compute_backward_aliases(function, &rpo, &incoming_edges);

        // Backward-alias closure of the literal back-edge args: every
        // value that can become a loop-header parameter's storage across
        // a back-edge, including those that only reach the back-edge arg
        // position through a forward edge first. A literal back-edge arg
        // that is a block parameter contributes its whole backward set;
        // an instruction result or function parameter contributes only
        // itself (its backward set is the singleton).
        let mut back_edge_participants: HashSet<ValueId> = HashSet::default();
        for &arg in &back_edge_args {
            back_edge_participants.insert(arg);
            if let Some(arg_set) = backward_aliases.get(&arg) {
                back_edge_participants.extend(arg_set.iter().copied());
            }
        }

        // A back-edge arg is iteration-local *fresh* if it re-allocates
        // distinct storage every iteration: a `make_array` (re-executes)
        // or a `Call` result (the callee allocates fresh — the same
        // assumption [`Context::non_aliasing_array_values`] makes).
        // `array_set` results are deliberately *excluded*: an `array_set`
        // may mutate its source in place, so its result can be the *same*
        // storage as a prior iteration's, breaking the distinct-generation
        // guarantee the swap exclusion relies on.
        let iteration_local_fresh: HashSet<ValueId> = back_edge_args
            .iter()
            .copied()
            .filter(|v| make_array_values.contains(v) || call_result_values.contains(v))
            .collect();

        // Array values that allocate **fresh** storage at their definition:
        // `make_array` (a new array) and `Call` results (the callee allocates;
        // the same heuristic [`Context::non_aliasing_array_values`] makes).
        // `array_set` results are excluded — they may mutate in place rather
        // than allocate. Used by the freshness wall in
        // [`Context::compute_uncovered_values`]: storage that is freshly
        // allocated on a path has no other live alias, so an in-place mutation
        // there cannot be observed through one.
        let fresh_array_values: HashSet<ValueId> =
            make_array_values.union(&call_result_values).copied().collect();

        // Swap exclusions. For every loop back-edge `be_start → header`,
        // inspect each array-typed header parameter at `source_pos`: if
        // the back-edge rebinds it to a *sibling* header parameter
        // (`source ← sibling`) whose own back-edge arg is an
        // iteration-local fresh allocation, and the source and sibling
        // receive distinct storage on every forward edge into the header,
        // record the sibling as excluded from the source's alias-set. See
        // [`Context::swap_excluded_aliases`].
        let mut swap_excluded_aliases: HashMap<ValueId, HashSet<ValueId>> = HashMap::default();
        for &(be_start, header) in &back_edges {
            let Some(edges) = incoming_edges.get(&header) else { continue };
            let Some(be_args) =
                edges.iter().find(|(pred, _)| *pred == be_start).map(|(_, args)| args)
            else {
                continue;
            };
            let params = function.dfg.block_parameters(header);
            for (source_pos, &source_param) in params.iter().enumerate() {
                if !function.dfg.type_of_value(source_param).contains_an_array() {
                    continue;
                }
                let Some(&sibling) = be_args.get(source_pos) else { continue };
                // `source_param ← sibling` must be a genuine swap to a
                // *different* sibling header parameter (not self-threading,
                // not a result).
                if sibling == source_param {
                    continue;
                }
                let Some(sibling_pos) = params.iter().position(|&pp| pp == sibling) else {
                    continue;
                };
                // The sibling's own back-edge arg must be an
                // iteration-local fresh allocation (the `c3 = [..]` or
                // `c3 = f()` half of the swap).
                if !be_args.get(sibling_pos).is_some_and(|a| iteration_local_fresh.contains(a)) {
                    continue;
                }
                // Loop-entry guard. On every *forward* edge into the
                // header, the source param and the sibling must receive
                // **distinct** storage, or they can already alias at the
                // array_set in the entry iteration (`source_0 = sibling_0`)
                // and the exclusion would mask a real hazard. Two ways
                // that can happen:
                //
                // - the sibling flows into the source's forward arg
                //   (`sibling ∈ backward(source_forward_arg)`) — the
                //   source *is* the sibling from the start.
                // - the source's and sibling's forward args share a
                //   backward-set member — e.g. `jmp header(v, v)` feeds the
                //   same `v` to both, so they're runtime-equal even though
                //   the directed backward walk keeps them in separate sets.
                let backward_set = |v: ValueId| -> im::HashSet<ValueId> {
                    backward_aliases.get(&v).cloned().unwrap_or_else(|| im::HashSet::unit(v))
                };
                let entry_aliased = edges
                    .iter()
                    .filter(|(pred, _)| !back_edges.contains(&(*pred, header)))
                    .any(|(_, args)| {
                        let Some(&source_forward_arg) = args.get(source_pos) else {
                            return false;
                        };
                        let source_forward_aliases = backward_set(source_forward_arg);
                        if source_forward_aliases.contains(&sibling) {
                            return true;
                        }
                        args.get(sibling_pos).is_some_and(|&sibling_forward_arg| {
                            let sibling_forward_aliases = backward_set(sibling_forward_arg);
                            source_forward_aliases
                                .iter()
                                .any(|x| sibling_forward_aliases.contains(x))
                        })
                    });
                if entry_aliased {
                    continue;
                }
                swap_excluded_aliases.entry(source_param).or_default().insert(sibling);
            }
        }

        // Propagate swap exclusions forward across non-back-edge
        // block-parameter edges. On a forward edge `P ← A`, `P` is `A` within
        // the same iteration, so `P`'s storage is `A`'s storage; a sibling
        // distinct from `A`'s per-iteration storage is therefore distinct from
        // `P`'s too. This carries an exclusion recorded on a loop-header
        // parameter to a successor parameter seeded from it — e.g. an
        // inner-loop header (the array_set source) fed from the swapped
        // outer-loop parameter — so `alias_set_for` drops the sibling for the
        // inner source as well. Back-edges are excluded: across one, `P` is a
        // *prior* iteration's `A`, a distinct storage.
        let mut changed = true;
        while changed {
            changed = false;
            for (dest, edges) in &incoming_edges {
                let params = function.dfg.block_parameters(*dest);
                for (i, &param) in params.iter().enumerate() {
                    for (pred, args) in edges {
                        if back_edges.contains(&(*pred, *dest)) {
                            continue;
                        }
                        let Some(&arg) = args.get(i) else { continue };
                        let Some(arg_excl) = swap_excluded_aliases.get(&arg).cloned() else {
                            continue;
                        };
                        let entry = swap_excluded_aliases.entry(param).or_default();
                        for x in arg_excl {
                            if x != param && entry.insert(x) {
                                changed = true;
                            }
                        }
                    }
                }
            }
        }

        Self {
            function,
            cfg,
            dom_tree,
            fresh_array_values,
            loop_blocks,
            backward_aliases,
            array_value_defs,
            non_aliasing_array_values,
            iteration_local_fresh,
            inc_rc_locations,
            back_edge_args,
            back_edge_participants,
            swap_excluded_aliases,
        }
    }

    /// Look up `source`'s backward alias-set and filter out values that
    /// can't represent the pre-mutation storage of the `array_set` at
    /// `(array_set_block, array_set_idx)`. The source itself is always
    /// kept.
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
    ///   callee, so it isn't a real alias of the `array_set`'s source.
    ///   This is a heuristic — a function that returns its input *would*
    ///   create a real alias, and we'd miss that. In practice the
    ///   frontend's array-returning functions allocate fresh storage.
    ///
    /// Also drop **iteration-local fresh results** — `MakeArray` (or
    /// `Call`) results that appear on at least one loop back-edge
    /// ([`Context::iteration_local_fresh`]). Such a value re-allocates
    /// fresh storage each iteration, so the loop-header parameter it
    /// feeds on the back-edge holds a *different* allocation in the next
    /// iteration than the one this iteration's `array_set` may have
    /// mutated. **Non-back-edge `MakeArray` results stay in the
    /// alias-set** — they represent a one-time allocation whose storage
    /// the `array_set` can mutate in place.
    ///
    /// **Post-array_set-in-same-block filter.** Drop instruction
    /// results whose defining position is in `array_set_block` at an
    /// index *greater than* `array_set_idx`. Such a value doesn't exist
    /// at the `array_set`'s program point yet — it's literally about to
    /// be allocated on a later instruction — so it can't represent the
    /// storage the `array_set` might mutate. It can land in the backward
    /// set through a round-trip: `make_array → forward-arg → block
    /// param → back-edge → source's param`. Dropping it at lookup time
    /// makes the per-arg kill in [`Context::succ_use_set`] correctly
    /// fire on the forward edge where the future-value is passed.
    ///
    /// The source itself is kept even when it happens to be filtered
    /// (e.g. a chain `v_a = array_set _ ; v_b = array_set v_a`): `v_a`
    /// is *this* check's source, so its forward uses are exactly what
    /// we want to look for. The post-array_set filter can't fire on the
    /// source because the source must be defined before the `array_set`
    /// that uses it.
    fn alias_set_for(
        &self,
        source: ValueId,
        array_set_block: BasicBlockId,
        array_set_idx: usize,
    ) -> im::HashSet<ValueId> {
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
                if self.iteration_local_fresh.contains(&v) {
                    return false;
                }
                if self.swap_excluded_aliases.get(&source).is_some_and(|qs| qs.contains(&v)) {
                    return false;
                }
                if let Some(&(def_block, def_idx)) = self.array_value_defs.get(&v)
                    && def_block == array_set_block
                    && def_idx > array_set_idx
                {
                    return false;
                }
                true
            })
            .collect()
    }

    /// The subset of `alias_set` whose members are **not** provably protected
    /// on every path to the `array_set` — the use-set the forward walk must
    /// check. An empty result means every member is protected, so the
    /// `array_set` can be accepted without the forward walk.
    ///
    /// A member is dropped (provably protected) when, on every runtime path,
    /// the storage it represents is either RC-bumped by an `inc_rc` (so the
    /// `array_set` copies) or an iteration-local fresh allocation (so it has no
    /// other live alias). Dropping covered members is what stops the forward
    /// walk from flagging a read of an `inc_rc`'d sibling on a path where the
    /// `array_set` never mutates that storage (the alias-set-merging
    /// false-positive). Keeping the uncovered members preserves real hazards,
    /// such as the issue SSA where the source itself is uncovered and read
    /// after the `array_set`.
    fn unprotected_aliases(
        &self,
        alias_set: &im::HashSet<ValueId>,
        source: ValueId,
        array_set_block: BasicBlockId,
        array_set_idx: usize,
    ) -> im::HashSet<ValueId> {
        // Fast full-accept short-circuits cover the whole array_set at once.
        if self.some_inc_rc_precedes(alias_set, source, array_set_block, array_set_idx) {
            return im::HashSet::new();
        }
        // Otherwise compute per-member coverage and keep only uncovered members.
        let uncovered = self.compute_uncovered_values(source, array_set_block, array_set_idx);
        alias_set.iter().copied().filter(|m| uncovered.contains(m)).collect()
    }

    /// Fast, whole-array_set acceptance predicate: returns `true` when some
    /// `inc_rc` on an alias-set member protects the `array_set` on **every**
    /// path. This is the cheap common case; per-path coverage that this misses
    /// (e.g. `inc_rc` on one arm and a fresh allocation on the other) is
    /// handled by [`Context::compute_uncovered_values`].
    ///
    /// Three acceptances, each sound on every path:
    ///
    /// - **Same-block, before the `array_set`.** The block is on every path to
    ///   its own `array_set`, so an earlier `inc_rc` always runs first.
    /// - **Dominating block.** An `inc_rc` whose block dominates the `array_set`
    ///   is on every path to it (e.g. an `inc_rc` in a loop pre-header).
    /// - **Back-edge-participant relaxation.** An `inc_rc` on a *non-source*
    ///   alias that appears as a jmp/jmpif arg on a loop **back-edge** is taken
    ///   as a codegen signal regardless of program-point order: the frontend
    ///   is deliberately managing iteration aliasing, and the bump's program
    ///   point can come *after* the `array_set` — it is the back-edge thread-back
    ///   that delivers the protection to the next iteration. Forward-edge cases
    ///   don't need this because [`Context::backward_aliases`] keeps the
    ///   forward-edge value out of the source's alias-set in the first place.
    ///
    /// Mere RPO precedence is *not* one of these: a sibling-branch `inc_rc`
    /// precedes in RPO yet the other branch reaches the `array_set` without it
    /// (`end_to_end_branch_local_inc_rc_does_not_protect_array_set_in_join_is_rejected`).
    ///
    /// Well-formed SSA contains no `DecrementRc`, so we don't need to worry
    /// about a `dec_rc` intervening between the `inc_rc` and the `array_set`.
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
            if *value != source && self.back_edge_args.contains(value) {
                return true;
            }
            for &(inc_block, inc_idx) in locations {
                if inc_block == array_set_block {
                    if inc_idx < array_set_idx {
                        return true;
                    }
                } else if self.dom_tree.dominates(inc_block, array_set_block) {
                    return true;
                }
            }
        }
        false
    }

    /// Whether `value` is a fresh allocation whose storage is distinct on
    /// every iteration that reaches the `array_set` in `array_set_block` — i.e.
    /// a `make_array`/`Call` result whose defining block shares a loop with
    /// `array_set_block`, so it re-executes each iteration. A fresh allocation
    /// outside any shared loop is a one-time storage that the `array_set`
    /// mutates exactly once; reading it back through the same name would
    /// observe that mutation, so it does **not** count as protection.
    fn is_iteration_local_fresh(&self, value: ValueId, array_set_block: BasicBlockId) -> bool {
        if !self.fresh_array_values.contains(&value) {
            return false;
        }
        let Some(&(def_block, _)) = self.array_value_defs.get(&value) else {
            return false;
        };
        self.loop_blocks
            .iter()
            .any(|blocks| blocks.contains(&def_block) && blocks.contains(&array_set_block))
    }

    /// The set of values that lie on at least one **uncovered** backward path
    /// from the `array_set` — a path on which the threaded source storage reaches
    /// a function parameter / unthreadable root without crossing an `inc_rc` or
    /// iteration-local-fresh wall. The caller intersects this with the
    /// alias-set to obtain the forward walk's use-set; an empty intersection
    /// means every member is protected, so the `array_set` is accepted.
    ///
    /// The walk threads the *exact* source storage backward: across a
    /// block-parameter edge it follows the predecessor's argument, and at an
    /// `array_set` result it continues with that `array_set`'s `array` operand
    /// (the result shares the operand's storage). This recognizes path-specific
    /// protection a block-level cut cannot — `inc_rc` on one arm of a diamond
    /// and a fresh `make_array` on the other, threaded through a loop (the
    /// `func_1`/`v20` fuzzer shape).
    ///
    /// It is computed over the `(block, value)` threading graph in two phases,
    /// iteratively (the predecessor chain can be thousands of blocks deep on
    /// large functions, which recursion would risk overflowing):
    ///
    /// 1. **Build.** From the seed `(array_set_block, source)`, expand each
    ///    state to its backward successors. A wall (`inc_rc` on the value, or
    ///    an iteration-local fresh allocation) is a covered sink. Reaching a
    ///    block with no predecessors, or a value defined by an unthreadable
    ///    instruction (a non-iteration-local allocation, or an `array_get` of a
    ///    nested array), or an edge whose argument can't be resolved, is an
    ///    *uncovered terminal*. `seed_idx` limits the `inc_rc` wall to bumps
    ///    *before* the `array_set` in its own block; any other block on a path
    ///    executes fully before the `array_set`, so an `inc_rc` anywhere in it
    ///    counts.
    /// 2. **Propagate.** A state is uncovered if it is an uncovered terminal or
    ///    has an uncovered successor; iterate to a fixed point. A cycle with no
    ///    uncovered terminal stays covered (a back-edge introduces no new
    ///    entry-originating path).
    ///
    /// Crediting freshness only ever shrinks the use-set, so it can add
    /// false-negatives (a freshly-allocated array read back through its own
    /// name after an in-place mutation — a shape the frontend does not emit)
    /// but never false-positives.
    fn compute_uncovered_values(
        &self,
        source: ValueId,
        array_set_block: BasicBlockId,
        array_set_idx: usize,
    ) -> HashSet<ValueId> {
        struct Node {
            successors: Vec<(BasicBlockId, ValueId)>,
            uncovered_terminal: bool,
        }

        // Phase 1: build the backward threading graph.
        let mut graph: HashMap<(BasicBlockId, ValueId), Node> = HashMap::default();
        let mut worklist: Vec<(BasicBlockId, ValueId, Option<usize>)> =
            vec![(array_set_block, source, Some(array_set_idx))];

        while let Some((block, value, seed_idx)) = worklist.pop() {
            if graph.contains_key(&(block, value)) {
                continue;
            }

            // Wall: an iteration-local `make_array`/`Call` result is newly
            // allocated storage on this path, distinct each iteration.
            if self.is_iteration_local_fresh(value, array_set_block) {
                graph
                    .insert((block, value), Node { successors: vec![], uncovered_terminal: false });
                continue;
            }

            // Wall: an `inc_rc` on the threaded value in this block.
            if let Some(locations) = self.inc_rc_locations.get(&value)
                && locations
                    .iter()
                    .any(|&(b, i)| b == block && seed_idx.is_none_or(|limit| i < limit))
            {
                graph
                    .insert((block, value), Node { successors: vec![], uncovered_terminal: false });
                continue;
            }

            // If `value` is defined in this block, the walk stops here.
            if let Some(&(def_block, def_idx)) = self.array_value_defs.get(&value)
                && def_block == block
            {
                let inst_id = self.function.dfg[block].instructions()[def_idx];
                // An `array_set` result shares its operand's storage; continue
                // threading with the operand.
                if let Instruction::ArraySet { array, .. } = self.function.dfg[inst_id] {
                    graph.insert(
                        (block, value),
                        Node { successors: vec![(block, array)], uncovered_terminal: false },
                    );
                    worklist.push((block, array, seed_idx));
                    continue;
                }
                // A non-iteration-local allocation or other instruction we
                // cannot thread: an uncovered terminal.
                graph.insert((block, value), Node { successors: vec![], uncovered_terminal: true });
                continue;
            }

            // Flows in from predecessors.
            let preds: Vec<BasicBlockId> = self.cfg.predecessors(block).collect();
            if preds.is_empty() {
                // Reached the entry / a source-less block unwalled.
                graph.insert((block, value), Node { successors: vec![], uncovered_terminal: true });
                continue;
            }
            // If `value` is a parameter of this block, follow each predecessor's
            // argument at that position; otherwise the same value flows in from
            // every predecessor.
            let params = self.function.dfg.block_parameters(block);
            let param_pos = params.iter().position(|&p| p == value);
            let mut successors = Vec::new();
            let mut uncovered_terminal = false;
            for &pred in &preds {
                let next = match param_pos {
                    Some(i) => match self.edge_arg(pred, block, i) {
                        Some(arg) => arg,
                        // An unresolvable edge argument: conservatively treat
                        // as uncovered rather than silently dropping the path.
                        None => {
                            uncovered_terminal = true;
                            continue;
                        }
                    },
                    None => value,
                };
                successors.push((pred, next));
                worklist.push((pred, next, None));
            }
            graph.insert((block, value), Node { successors, uncovered_terminal });
        }

        // Phase 2: propagate "uncovered" from terminals to a fixed point.
        let mut uncovered: HashSet<(BasicBlockId, ValueId)> =
            graph.iter().filter(|(_, n)| n.uncovered_terminal).map(|(k, _)| *k).collect();
        let mut changed = true;
        while changed {
            changed = false;
            for (state, node) in &graph {
                if !uncovered.contains(state)
                    && node.successors.iter().any(|s| uncovered.contains(s))
                {
                    uncovered.insert(*state);
                    changed = true;
                }
            }
        }

        uncovered.into_iter().map(|(_, value)| value).collect()
    }

    /// The argument passed to parameter position `i` of `dest` on the edge
    /// from `pred`. `None` if `pred`'s terminator does not target `dest` or has
    /// no argument at that position.
    fn edge_arg(&self, pred: BasicBlockId, dest: BasicBlockId, i: usize) -> Option<ValueId> {
        match self.function.dfg[pred].terminator()? {
            TerminatorInstruction::Jmp { destination, arguments, .. } if *destination == dest => {
                arguments.get(i).copied()
            }
            TerminatorInstruction::JmpIf {
                then_destination,
                then_arguments,
                else_destination,
                else_arguments,
                ..
            } => {
                if *then_destination == dest {
                    then_arguments.get(i).copied()
                } else if *else_destination == dest {
                    else_arguments.get(i).copied()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Forward CFG walk from after the `array_set` looking for a
    /// non-terminator instruction that reads a value still in the alias
    /// use-set.
    ///
    /// **Use-set evolution.** Starts as `alias_set` — which is **already
    /// narrowed** to the uncovered members by [`Context::unprotected_aliases`]
    /// before this is called, so provably-protected aliases never enter the
    /// walk — and only shrinks from there. Kills are applied **during
    /// propagation** to each successor — see [`Context::succ_use_set`] for the
    /// kill rules.
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
    /// **Chain-aware index filter.** Two auxiliary sets evolve alongside the
    /// `use_set` to make the filter sound in the presence of `array_set`
    /// chains:
    ///
    /// - **`derived`** tracks values that may share the `array_set`'s source
    ///   storage at runtime through transitive in-place mutations. Seeded
    ///   with the `array_set`'s own result; grown on every later `array_set`
    ///   whose `array` operand is already in `derived`; propagated across
    ///   block-param edges the same way the alias use-set is.
    /// - **`tainted_indices`** tracks the set of storage positions any
    ///   chain link may have already written. `Some(set)` accumulates
    ///   constant write indices (seeded with `write_index_const`); set to
    ///   `None` (= "all positions") as soon as any chain write uses a
    ///   dynamic index.
    ///
    /// When we encounter `array_get v, idx` with `v ∈ use_set`, the read
    /// is a hazard iff `tainted_indices` covers `idx`. With both indices
    /// constant, that's a set-membership test; otherwise (either side
    /// dynamic, or `tainted_indices == None`) the verifier conservatively
    /// flags. An `array_set v, idx, _` with `v ∈ use_set` is index-aware in
    /// the same way: it copies forward every position except `idx`, so it
    /// is a hazard iff some `tainted_indices` position differs from `idx`
    /// (a same-index write overwrites the mutation and extends the chain
    /// instead). All other uses on a `use_set` member are always flagged —
    /// the SSA-vs-runtime divergence isn't index-local for them.
    ///
    /// **`IncrementRc` clears `derived`.** A program-point `inc_rc v` with
    /// `v ∈ use_set ∪ derived` lifts the storage's RC ≥ 2, so any
    /// subsequent `array_set` on a chain participant runs on fresh
    /// storage and can no longer clobber the source. We clear `derived`
    /// at that point. `tainted_indices` is *not* cleared — past damage
    /// to the shared storage is still observable through `use_set` reads.
    ///
    /// **Cycle detection.** Re-entering a block with a state that's a
    /// (`use_set`, derived, tainted_indices)-subset of the frontier we've
    /// already explored adds no new information. Each component is
    /// unioned into the frontier on entry; for `tainted_indices`, `None`
    /// is the absorbing element (a previously-`None` frontier covers any
    /// re-entry).
    fn find_reachable_aliased_use(
        &self,
        alias_set: &im::HashSet<ValueId>,
        source: ValueId,
        array_set_id: InstructionId,
        array_set_block: BasicBlockId,
        array_set_idx: usize,
        write_index_const: Option<FieldElement>,
    ) -> Option<AliasedUse> {
        let mut visited: HashMap<BasicBlockId, WalkState> = HashMap::default();

        // Protected-participant set. Exclude a non-source alias `v` when
        // all three hold:
        //
        // 1. `v ∈ alias_set` — `v` is in `source`'s backward set, i.e. it
        //    actually *flows into* `source`. This is the soundness guard:
        //    a value that only *receives* `source`'s storage (a forward
        //    successor) is not in the backward set, so its `inc_rc` runs
        //    *after* the array_set and cannot protect it. Requiring
        //    membership keeps such values in the use-set to be flagged.
        // 2. `v` carries its own `inc_rc` — combined with (3), the bump
        //    runs before `v` crosses the back-edge that re-binds it onto
        //    `source`'s loop-header parameter, so by the time `v`'s
        //    storage equals `source`'s it is RC ≥ 2 and the array_set
        //    copies rather than mutating in place.
        // 3. `v` is a loop back-edge participant
        //    ([`Context::back_edge_participants`]) — its storage reaches
        //    `source` through a back-edge (possibly via a forward edge
        //    into the back-edge arg first), which is what makes the
        //    `inc_rc` loop-carried.
        //
        // The gate is **per value** on that value's own `inc_rc` — a
        // sibling's `inc_rc` never exonerates an unprotected value. A
        // back-edge position fed by an `inc_rc`'d value on one predecessor
        // edge and an unprotected value on another protects only the
        // former; the latter stays in the use-set for the walk to flag.
        //
        // The removal is **sticky**: protected members are kept out of the
        // use-set for the whole walk, not just the initial seed. A
        // protected participant that is itself a loop-header parameter
        // would otherwise be re-introduced the moment another in-use alias
        // flows into its position (the add-rule in
        // [`Context::succ_use_set`]); excluding it there too keeps it out.
        // Dropping it from the seed also lets the per-arg kill rule drop
        // the loop-header parameter on the back-edge, since the value
        // threaded back is this protected participant rather than a
        // still-live alias.
        let protected: im::HashSet<ValueId> = alias_set
            .iter()
            .copied()
            .filter(|&v| {
                v != source
                    && self.inc_rc_locations.contains_key(&v)
                    && self.back_edge_participants.contains(&v)
            })
            .collect();
        let use_set: im::HashSet<ValueId> =
            alias_set.iter().copied().filter(|v| !protected.contains(v)).collect();

        let array_set_result = self.function.dfg.instruction_results(array_set_id)[0];
        let initial_state = WalkState {
            use_set,
            derived: im::HashSet::unit(array_set_result),
            tainted: write_index_const.map(im::HashSet::unit),
        };
        let mut worklist: Vec<WalkFrame> = vec![WalkFrame {
            block: array_set_block,
            start_idx: array_set_idx + 1,
            state: initial_state,
        }];

        while let Some(WalkFrame { block, start_idx, mut state }) = worklist.pop() {
            // No alias-set members in use means no read of this array_set's
            // source's storage can surface a hazard from here on.
            if state.use_set.is_empty() {
                continue;
            }

            // Cycle/redundancy check + bookkeeping only applies to *full*
            // block entries (start_idx == 0). The very first frame of the
            // walk starts mid-block and only covers a suffix of the block —
            // recording it here would incorrectly let a later back-edge
            // entry to the same block skip the unexplored prefix.
            if start_idx == 0 {
                if let Some(prev) = visited.get(&block)
                    && state.is_covered_by(prev)
                {
                    continue;
                }
                let new_frontier = match visited.get(&block) {
                    Some(prev) => state.merge(prev),
                    None => state.clone(),
                };
                visited.insert(block, new_frontier);
            }

            let instructions = self.function.dfg[block].instructions();
            for &inst_id in instructions.iter().skip(start_idx) {
                if inst_id == array_set_id {
                    continue;
                }
                let inst = &self.function.dfg[inst_id];

                match inst {
                    // `inc_rc` on an alias or chain-derived value lifts the
                    // shared storage's RC; subsequent chain writes run on
                    // fresh storage and stop tainting the source. Past
                    // tainted indices survive — the damage is already done.
                    Instruction::IncrementRc { value } => {
                        if state.derived.contains(value) || state.use_set.contains(value) {
                            state.derived.clear();
                        }
                    }
                    // Well-formed Brillig SSA shouldn't contain dec_rc;
                    // skip if encountered.
                    Instruction::DecrementRc { .. } => {}
                    Instruction::ArraySet { array, index, value, .. } => {
                        let array_in_use = state.use_set.contains(array);
                        if array_in_use {
                            // Index-aware, mirroring the `array_get` rule below.
                            // `array_set v, i, x` produces a *copy* of `v` with
                            // index `i` overwritten, so it only observes (copies
                            // forward) the indices it does **not** write. It can
                            // therefore surface the source's in-place mutation
                            // only if it copies a tainted index — i.e. some
                            // tainted index differs from this write index. A
                            // write to the (sole) tainted index overwrites the
                            // mutation and observes nothing. A dynamic write or
                            // read index, or fully-tainted (`None`) storage, is
                            // flagged conservatively.
                            let write_idx = self.function.dfg.get_numeric_constant(*index);
                            let observes_tainted = match (&state.tainted, write_idx) {
                                (None, _) | (_, None) => true,
                                (Some(t), Some(c)) => t.iter().any(|i| *i != c),
                            };
                            if observes_tainted {
                                return Some(AliasedUse { instruction: inst_id, value: *array });
                            }
                        }
                        if self.function.dfg.type_of_value(*value).contains_an_array()
                            && state.use_set.contains(value)
                        {
                            return Some(AliasedUse { instruction: inst_id, value: *value });
                        }
                        // Chain extension: a write through a value that may
                        // share the source's storage — a `derived` member, or a
                        // `use_set` member whose write we just cleared as
                        // non-observing — keeps mutating that storage, so record
                        // its write index and track the result.
                        if state.derived.contains(array) || array_in_use {
                            match (
                                state.tainted.as_mut(),
                                self.function.dfg.get_numeric_constant(*index),
                            ) {
                                (Some(t), Some(c)) => {
                                    t.insert(c);
                                }
                                _ => {
                                    state.tainted = None;
                                }
                            }
                            let [result] = self.function.dfg.instruction_result(inst_id);
                            state.derived.insert(result);
                        }
                    }
                    Instruction::ArrayGet { array, index, .. } => {
                        if state.use_set.contains(array) {
                            let read_idx = self.function.dfg.get_numeric_constant(*index);
                            let hazard = match (&state.tainted, read_idx) {
                                (None, _) | (_, None) => true,
                                (Some(set), Some(c)) => set.contains(&c),
                            };
                            if hazard {
                                return Some(AliasedUse { instruction: inst_id, value: *array });
                            }
                        }
                    }
                    other => {
                        // Any other instruction reading an alias-set member
                        // through an array-typed operand is a hazard. The
                        // SSA-vs-runtime divergence isn't index-local for
                        // these instruction kinds.
                        let mut hit: Option<ValueId> = None;
                        other.for_each_value(|v| {
                            if hit.is_some() {
                                return;
                            }
                            if self.function.dfg.type_of_value(v).contains_an_array()
                                && state.use_set.contains(&v)
                            {
                                hit = Some(v);
                            }
                        });
                        if let Some(v) = hit {
                            return Some(AliasedUse { instruction: inst_id, value: v });
                        }
                    }
                }
            }

            let Some(terminator) = self.function.dfg[block].terminator() else { continue };
            match terminator {
                TerminatorInstruction::Jmp { destination, arguments, .. } => {
                    let next = self.succ_walk_state(*destination, arguments, &state, &protected);
                    worklist.push(WalkFrame { block: *destination, start_idx: 0, state: next });
                }
                TerminatorInstruction::JmpIf {
                    then_destination,
                    then_arguments,
                    else_destination,
                    else_arguments,
                    ..
                } => {
                    let then_state =
                        self.succ_walk_state(*then_destination, then_arguments, &state, &protected);
                    worklist.push(WalkFrame {
                        block: *then_destination,
                        start_idx: 0,
                        state: then_state,
                    });
                    let else_state =
                        self.succ_walk_state(*else_destination, else_arguments, &state, &protected);
                    worklist.push(WalkFrame {
                        block: *else_destination,
                        start_idx: 0,
                        state: else_state,
                    });
                }
                TerminatorInstruction::Return { .. }
                | TerminatorInstruction::Unreachable { .. } => (),
            }
        }

        None
    }

    /// Propagate the walk state across a block-parameter edge. `use_set`
    /// and `derived` follow the same kill/add rules ([`Context::succ_use_set`]);
    /// `tainted` is carried unchanged, since it tracks storage positions
    /// (not SSA values).
    fn succ_walk_state(
        &self,
        dest: BasicBlockId,
        arguments: &[ValueId],
        state: &WalkState,
        protected: &im::HashSet<ValueId>,
    ) -> WalkState {
        WalkState {
            use_set: self.succ_use_set(dest, arguments, &state.use_set, protected),
            derived: self.succ_use_set(dest, arguments, &state.derived, protected),
            tainted: state.tainted.clone(),
        }
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
    ///      `array_set`'s source (most commonly the `array_set`'s own
    ///      result, excluded at lookup time): drop it.
    ///    - **Add.** If the param is not in `use_set` but the arg is,
    ///      this edge introduces a fresh alias: the param at `dest`'s
    ///      entry shares storage with an alias-set member, so add it —
    ///      *unless* the param is a protected back-edge participant
    ///      ([`Context::find_reachable_aliased_use`]), which must stay out
    ///      of the use-set for the whole walk (its own `inc_rc` keeps it
    ///      RC ≥ 2). Without this exclusion a protected participant that
    ///      is itself a loop-header parameter would be re-introduced the
    ///      moment any in-use alias flowed into its position.
    ///    - Otherwise (both in or both out), no change.
    ///
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
        protected: &im::HashSet<ValueId>,
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
                (false, true) if !protected.contains(&param) => {
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
/// # Why the walk is directed (param ← arg), not symmetric
///
/// A symmetric closure over `(param, arg)` pairs would put two sibling
/// args passed to the same block in the same class via the shared
/// parameter, even though at runtime they never refer to the same
/// storage (each runtime path takes only one of them). Following the
/// directed `param ← arg` edges instead means sibling parameters don't
/// co-mingle in each other's sets — only the predecessors that flow
/// *into* a given parameter do.
///
/// # Why we deliberately do **not** chase past `array_set` results
///
/// At runtime the result may or may not alias the source depending on
/// RC. The verifier's job is to flag the cases where it *might*. We
/// model this by *excluding* the result from each alias-set at lookup
/// time (in [`Context::alias_set_for`]) rather than refusing to chase
/// through it here. Otherwise a chain
/// `v1 = array_set v0 ; v2 = array_set v1 ; v3 = array_set v2` would
/// pull `v0` into the source of every later `array_set`, and an
/// `inc_rc v0` that legitimately protects only `v1`'s `array_set` would
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

/// Per-frame state of the forward reachable-use walk: which alias-set
/// members are live, which chain-derived values share storage with the
/// source, and which storage positions any chain link may already have
/// clobbered. Also serves as the per-block visited frontier for cycle
/// detection — see [`WalkState::is_covered_by`] / [`WalkState::merge`].
#[derive(Clone)]
struct WalkState {
    /// Alias-set members live at this point: values that may share the
    /// `array_set`'s source storage *at the `array_set`'s program point*. A
    /// non-terminator read of one of these is the hazard the walk is
    /// looking for.
    use_set: im::HashSet<ValueId>,
    /// Values that may share the source's storage through transitive
    /// in-place chain mutations. Seeded with the `array_set`'s own result;
    /// extended by every later `array_set` whose `array` operand is
    /// already in `derived`.
    derived: im::HashSet<ValueId>,
    /// Storage positions any chain link may already have written. `None`
    /// (= "all positions") absorbs a dynamic chain write — once we lose
    /// precise tracking we can't recover it.
    tainted: Option<im::HashSet<FieldElement>>,
}

impl WalkState {
    /// `self` is covered by `prev` when every potential hazard `self`
    /// could surface is also reachable from `prev`. For `use_set` and
    /// `derived` that's the subset relation; for `tainted`, `None`
    /// (= "all positions") absorbs as the upper bound — a previously
    /// `None` frontier covers any re-entry, but a current `None`
    /// requires `prev` also to be `None`.
    fn is_covered_by(&self, prev: &Self) -> bool {
        if !self.use_set.is_subset(&prev.use_set) || !self.derived.is_subset(&prev.derived) {
            return false;
        }
        match (&prev.tainted, &self.tainted) {
            (None, _) => true,
            (_, None) => false,
            (Some(p), Some(c)) => c.is_subset(p),
        }
    }

    /// Component-wise union — the frontier we record after visiting a
    /// block, so future re-entries can be cycle-checked against
    /// everything we've already explored from that block.
    fn merge(&self, other: &Self) -> Self {
        Self {
            use_set: self.use_set.clone().union(other.use_set.clone()),
            derived: self.derived.clone().union(other.derived.clone()),
            tainted: match (&self.tainted, &other.tainted) {
                (None, _) | (_, None) => None,
                (Some(a), Some(b)) => Some(a.clone().union(b.clone())),
            },
        }
    }
}

/// One entry on the forward reachable-use walk's worklist: a block to
/// enter, the instruction index to start at within that block, and the
/// walk's evolving state. `start_idx > 0` denotes the very first frame,
/// which continues inside the `array_set`'s own block past the `array_set`
/// instruction itself; all later frames enter at block start.
struct WalkFrame {
    block: BasicBlockId,
    start_idx: usize,
    state: WalkState,
}

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

/// Test support shared by both submodules' `tests`. The reject helpers stay
/// per-module (each asserts its own error variant), but acceptance is the same
/// property for both — *neither* verifier rejects — so it lives here.
#[cfg(test)]
pub(crate) mod tests {
    use super::{array_set, call};
    use crate::{errors::RtResult, ssa::ssa_gen::Ssa};

    /// Run the full `rc_invariant` check — every submodule verifier — over
    /// `ssa`, returning the first violation.
    pub(crate) fn verify_all(ssa: &Ssa) -> RtResult<()> {
        array_set::verify(ssa)?;
        call::verify(ssa)?;
        Ok(())
    }

    /// Assert the full `rc_invariant` check ([`verify_all`]) accepts `src`.
    pub(crate) fn assert_verifier_accepts(src: &str) {
        assert_verifier_accepts_because(src, "");
    }

    /// Same as [`assert_verifier_accepts`] but includes `reason` in the panic
    /// message — useful for documenting why the SSA is *expected* to be
    /// accepted (e.g. "loop exit reads a rebound block-param").
    pub(crate) fn assert_verifier_accepts_because(src: &str, reason: &str) {
        let ssa = Ssa::from_str(src).expect("SSA parses");
        if let Err(err) = verify_all(&ssa) {
            if reason.is_empty() {
                panic!("expected the verifier to accept, but it rejected: {err:?}");
            } else {
                panic!("expected the verifier to accept ({reason}), but it rejected: {err:?}");
            }
        }
    }
}
