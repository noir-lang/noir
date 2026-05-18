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

use rustc_hash::FxHashMap as HashMap;

use crate::{
    errors::RtResult,
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

        let cfg = ControlFlowGraph::with_function(self);
        let post_order = PostOrder::with_cfg(&cfg);
        let mut dom_tree = DominatorTree::with_cfg_and_post_order(&cfg, &post_order);
        let inc_rc_locations = collect_inc_rc_locations(self);
        let value_aliases = collect_value_aliases(self);
        let array_operand_uses = collect_array_operand_uses(self);

        for block_id in self.reachable_blocks() {
            for (idx, instruction_id) in self.dfg[block_id].instructions().iter().enumerate() {
                let Instruction::ArraySet { array, .. } = self.dfg[*instruction_id] else {
                    continue;
                };
                let [result] = self.dfg.instruction_result(*instruction_id);

                let alias_set = alias_set_for(&value_aliases, array, result);

                // Cheap check first: if any `inc_rc` on an alias-set member dominates
                // this `array_set`, the in-place mutation is bounded by RC ≥ 2 and
                // the SSA is safe regardless of downstream uses.
                if some_inc_rc_dominates(
                    &mut dom_tree,
                    &inc_rc_locations,
                    &alias_set,
                    block_id,
                    idx,
                ) {
                    continue;
                }

                // Expensive: forward CFG walk looking for an aliased read.
                // Step 5 will gate `Err` on this; for now we just exercise the
                // function.
                let _has_use = has_reachable_aliased_use(
                    self,
                    &array_operand_uses,
                    &alias_set,
                    *instruction_id,
                    block_id,
                    idx,
                );
            }
        }
        Ok(())
    }
}

/// A non-terminator instruction's reference to an array-typed value.
///
/// One entry per (instruction, array-typed operand) pair — an instruction
/// with two array operands contributes two entries. Tuples are
/// `(instruction-index-within-block, instruction-id, operand-value)`.
type ArrayOperandUse = (usize, InstructionId, ValueId);

/// For each block, list every (non-terminator) array-typed operand used by
/// some instruction in the block, in source order. The reachable-use walk
/// uses this to skip over instructions that can't be hits (most arithmetic
/// and control-flow instructions don't reference any array value).
///
/// Terminator operands are deliberately excluded — they are the legitimate
/// threading mechanism the invariant relies on and are handled by the
/// per-arg kill rule in [`succ_use_set`].
fn collect_array_operand_uses(function: &Function) -> HashMap<BasicBlockId, Vec<ArrayOperandUse>> {
    let mut out: HashMap<BasicBlockId, Vec<ArrayOperandUse>> = HashMap::default();
    for block_id in function.reachable_blocks() {
        let mut uses: Vec<ArrayOperandUse> = Vec::new();
        for (idx, instruction_id) in function.dfg[block_id].instructions().iter().enumerate() {
            function.dfg[*instruction_id].for_each_value(|v| {
                if function.dfg.type_of_value(v).contains_an_array() {
                    uses.push((idx, *instruction_id, v));
                }
            });
        }
        if !uses.is_empty() {
            out.insert(block_id, uses);
        }
    }
    out
}

/// Indexes every `inc_rc v` instruction in the function by `v`, recording its
/// `(block, position-in-block)`. Used for the dominance check at each
/// `array_set` site.
fn collect_inc_rc_locations(function: &Function) -> HashMap<ValueId, Vec<(BasicBlockId, usize)>> {
    let mut locations: HashMap<ValueId, Vec<(BasicBlockId, usize)>> = HashMap::default();
    for block_id in function.reachable_blocks() {
        for (idx, instruction_id) in function.dfg[block_id].instructions().iter().enumerate() {
            if let Instruction::IncrementRc { value } = function.dfg[*instruction_id] {
                locations.entry(value).or_default().push((block_id, idx));
            }
        }
    }
    locations
}

/// Returns `true` if some `inc_rc r` for an `r ∈ alias_set` dominates the
/// `array_set` located at `(array_set_block, array_set_idx)`.
///
/// "Dominates" means the `inc_rc` is guaranteed to execute before the
/// `array_set` on every CFG path — either in a strictly-earlier position
/// within the same block, or in a block that strictly or improperly dominates
/// the `array_set`'s block (improper dominance suffices when the position
/// within the block is earlier).
///
/// Well-formed SSA contains no `DecrementRc` (the existing SSA validator
/// panics on any), so we don't need to worry about a `dec_rc` intervening
/// between the `inc_rc` and the `array_set`.
fn some_inc_rc_dominates(
    dom_tree: &mut DominatorTree,
    inc_rc_locations: &HashMap<ValueId, Vec<(BasicBlockId, usize)>>,
    alias_set: &im::HashSet<ValueId>,
    array_set_block: BasicBlockId,
    array_set_idx: usize,
) -> bool {
    for value in alias_set {
        let Some(locations) = inc_rc_locations.get(value) else {
            continue;
        };
        for &(inc_block, inc_idx) in locations {
            if inc_block == array_set_block {
                if inc_idx < array_set_idx {
                    return true;
                }
            } else if dom_tree.dominates(inc_block, array_set_block) {
                return true;
            }
        }
    }
    false
}

/// Forward CFG walk from after the `array_set` looking for a non-terminator
/// instruction that reads a value still in the alias use-set.
///
/// **Use-set evolution.** Starts as `alias_set` and only shrinks. Kills are
/// applied **during propagation** to each successor — see
/// [`succ_use_set`] for the per-arg kill rule. The intuition: a parameter
/// of the successor is in the alias-set means it shares storage with the
/// array_set's source. After the jump, the parameter is rebound to the
/// arg passed by this terminator. If the arg is *also* in the use-set
/// (still an alias), the parameter stays in the use-set; if the arg is
/// a *fresh* value (not in the use-set, e.g. the array_set's own result),
/// the parameter is killed.
///
/// **What counts as a use.** Only non-terminator operands. Terminator
/// arguments are the legitimate threading mechanism the invariant relies
/// on; `jmp b(v_alias)` is how the post-mutation value reaches the next
/// block where it is re-bound to that block's parameter. The kill logic
/// above already accounts for these args.
///
/// The original `array_set` itself is also skipped, in case a cycle
/// re-enters its block — it is, by construction, a use of its own source,
/// not a hazard.
///
/// **Cycle detection.** Re-visiting a block with a use-set that is a
/// subset of what we already explored at that block adds no new
/// information. We record the *union* of use-sets seen per block and skip
/// on subset matches.
fn has_reachable_aliased_use(
    function: &Function,
    array_operand_uses: &HashMap<BasicBlockId, Vec<ArrayOperandUse>>,
    alias_set: &im::HashSet<ValueId>,
    array_set_id: InstructionId,
    array_set_block: BasicBlockId,
    array_set_idx: usize,
) -> bool {
    let mut visited: HashMap<BasicBlockId, im::HashSet<ValueId>> = HashMap::default();

    // (block, start_idx, use_set_on_entry)
    //
    // `start_idx > 0` denotes the very first frame, which continues inside
    // the array_set's own block past the array_set instruction itself.
    let mut worklist: Vec<(BasicBlockId, usize, im::HashSet<ValueId>)> =
        vec![(array_set_block, array_set_idx + 1, alias_set.clone())];

    while let Some((block, start_idx, use_set)) = worklist.pop() {
        // No live aliases on this path: the instruction scan can't hit and
        // propagating the empty set to successors is wasted work.
        if use_set.is_empty() {
            continue;
        }

        // Cycle/redundancy check + bookkeeping only applies to *full* block
        // entries (start_idx == 0). The very first frame of the walk starts
        // mid-block (just after the array_set) and only covers a suffix of
        // the block — recording it here would incorrectly let a later
        // back-edge entry to the same block skip the unexplored prefix.
        if start_idx == 0 {
            if let Some(prev) = visited.get(&block)
                && use_set.is_subset(prev)
            {
                continue;
            }
            // Track the union so a later "subset of prev" check covers
            // everything we've ever explored this block with.
            let merged = visited.get(&block).cloned().unwrap_or_default().union(use_set.clone());
            visited.insert(block, merged);
        }

        // Only iterate non-terminator instructions that have an array-typed
        // operand — most arithmetic / branching instructions can't be a hit,
        // so iterating them is wasted work. Entries are pre-sorted by `idx`.
        if let Some(uses) = array_operand_uses.get(&block) {
            for &(idx, inst_id, operand) in uses.iter().skip_while(|(idx, _, _)| *idx < start_idx) {
                if inst_id == array_set_id {
                    continue;
                }
                if use_set.contains(&operand) {
                    return true;
                }
            }
        }

        // Propagate to each successor with per-arg kills applied.
        let Some(terminator) = function.dfg[block].terminator() else { continue };
        match terminator {
            TerminatorInstruction::Jmp { destination, arguments, .. } => {
                let next = succ_use_set(function, *destination, arguments, &use_set);
                worklist.push((*destination, 0, next));
            }
            TerminatorInstruction::JmpIf {
                then_destination,
                then_arguments,
                else_destination,
                else_arguments,
                ..
            } => {
                let then_next = succ_use_set(function, *then_destination, then_arguments, &use_set);
                worklist.push((*then_destination, 0, then_next));
                let else_next = succ_use_set(function, *else_destination, else_arguments, &use_set);
                worklist.push((*else_destination, 0, else_next));
            }
            TerminatorInstruction::Return { .. } | TerminatorInstruction::Unreachable { .. } => (),
        }
    }
    false
}

/// Compute the use-set carried into `dest` when its predecessor jumps with
/// `arguments`.
///
/// For each `dest.params[i]` that is currently in `use_set`, look at the
/// corresponding `arguments[i]`:
/// - If the arg is *also* in `use_set`, the parameter is rebound to a value
///   that still aliases the array_set's source; keep the parameter in the
///   use-set.
/// - Otherwise (arg is a fresh value — most commonly the array_set's own
///   result, which was excluded from the alias-set at lookup time), the
///   parameter is rebound to a non-aliased value; drop it from the use-set.
fn succ_use_set(
    function: &Function,
    dest: BasicBlockId,
    arguments: &[ValueId],
    use_set: &im::HashSet<ValueId>,
) -> im::HashSet<ValueId> {
    let mut result = use_set.clone();
    let params = function.dfg.block_parameters(dest);
    for (i, &param) in params.iter().enumerate() {
        if !use_set.contains(&param) {
            continue;
        }
        let arg_is_alias = arguments.get(i).is_some_and(|arg| use_set.contains(arg));
        if !arg_is_alias {
            result.remove(&param);
        }
    }
    result
}

/// Compute aliasing equivalence classes for every value that participates in
/// block-parameter threading.
///
/// Two values are in the same class iff they may share storage at runtime
/// through any chain of `jmp` / `jmpif` argument-to-parameter bindings. This
/// is the transitive closure of:
/// - `param[i] ≈ arg[i]` for every `jmp b(args)` (and each arm of a `jmpif`).
///
/// Implemented as union-find over [`ValueId`]s. The returned map sends each
/// touched value to its full equivalence class. Values that don't participate
/// in any union are absent from the map; their class is implicitly `{v}`.
///
/// # Why this covers the sibling-arg case
///
/// A `jmp b(v0, v0)` unifies both of `b`'s parameters with `v0` (and so with
/// each other). The earlier backward-only walk added `v0` to each parameter's
/// alias-set but treated the two parameters as independent — missing that
/// they alias each other. The union-find formulation captures this directly:
/// both parameters land in the same class as `v0`.
///
/// # What this still does **not** track
///
/// Aliasing introduced by `MakeArray` of nested arrays, `IfElse` on arrays,
/// non-inlined `Call` returns, or `Store`/`Load` on ineligible (nested-ref)
/// allocates. Those are listed in the module-level docs as known gaps; this
/// pass only formalizes block-parameter aliasing.
///
/// # Why we deliberately do **not** unify `array_set`'s source and result
///
/// At runtime the result may or may not alias the source depending on RC.
/// The verifier's job is to flag the cases where it *might*. We model this
/// by *excluding* the result from the alias-set at lookup time (see callers)
/// rather than unifying it into the equivalence class. Otherwise a chain
/// `v1 = array_set v0 ; v2 = array_set v1 ; v3 = array_set v2` would put
/// `{v0, v1, v2, v3}` in `v3`'s class, and an `inc_rc v0` that legitimately
/// protects only `v1`'s array_set would falsely appear to protect `v3` as
/// well. See `alias_set_does_not_walk_array_set_chains` for a worked example.
fn collect_value_aliases(function: &Function) -> HashMap<ValueId, im::HashSet<ValueId>> {
    let mut uf: UnionFind<ValueId> = UnionFind::new();

    for block_id in function.reachable_blocks() {
        let Some(terminator) = function.dfg[block_id].terminator() else {
            continue;
        };
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

    // Materialize the keys so the subsequent `find` calls can apply path
    // compression (`find` is `&mut self`; iterating `uf.keys()` borrows
    // `&self`, so the two cannot interleave directly).
    let keys: Vec<ValueId> = uf.keys().collect();

    // Group every value in the union-find by its class representative.
    let mut class_of_rep: HashMap<ValueId, im::HashSet<ValueId>> = HashMap::default();
    for &v in &keys {
        let rep = uf.find(v);
        class_of_rep.entry(rep).or_default().insert(v);
    }

    // Map each value directly to its class for O(1) per-array_set lookups.
    // The trees were just flattened by the loop above, so this pass's
    // `find` calls are amortized O(1).
    let mut result: HashMap<ValueId, im::HashSet<ValueId>> = HashMap::default();
    for v in keys {
        let rep = uf.find(v);
        if let Some(class) = class_of_rep.get(&rep) {
            result.insert(v, class.clone());
        }
    }
    result
}

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

/// Look up the alias-set for `source` in the precomputed equivalence classes
/// and remove `exclude` (the `array_set`'s own result) from it. See the
/// rationale in [`collect_value_aliases`] for why the result is excluded.
fn alias_set_for(
    value_aliases: &HashMap<ValueId, im::HashSet<ValueId>>,
    source: ValueId,
    exclude: ValueId,
) -> im::HashSet<ValueId> {
    let class = value_aliases.get(&source).cloned().unwrap_or_else(|| im::HashSet::unit(source));
    class.without(&exclude)
}

#[cfg(test)]
mod tests {
    use super::{ControlFlowGraph, alias_set_for, collect_value_aliases};
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
        let value_aliases = collect_value_aliases(function);

        let (source, result) = last_array_set(function).expect("array_set present");

        let alias_set = alias_set_for(&value_aliases, source, result);

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
        let value_aliases = collect_value_aliases(function);

        let (source, result) = last_array_set(function).expect("array_set present");

        let alias_set = alias_set_for(&value_aliases, source, result);

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

    /// Five dominance shapes for `inc_rc`, each isolated on its own array
    /// parameter so the inc_rcs don't accidentally cover for each other:
    ///   - `v0`: same-block, inc_rc *earlier* than the array_set → **dominates**.
    ///   - `v1`: inc_rc in a strictly-dominating block (entry) → **dominates**.
    ///   - `v3`: same-block, inc_rc *later* than the array_set → does **not**
    ///     dominate.
    ///   - `v4`: no inc_rc anywhere → does **not** dominate.
    ///   - `v2`: inc_rc in a sibling branch that does not dominate the
    ///     array_set's block → does **not** dominate.
    #[test]
    fn inc_rc_dominance_recognizes_dominating_and_local_positions() {
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
        let cfg = ControlFlowGraph::with_function(function);
        let post_order = super::PostOrder::with_cfg(&cfg);
        let mut dom_tree = super::DominatorTree::with_cfg_and_post_order(&cfg, &post_order);
        let inc_rc_locations = super::collect_inc_rc_locations(function);
        let value_aliases = collect_value_aliases(function);

        let entry_params = function.dfg.block_parameters(function.entry_block());
        let v0 = entry_params[0];
        let v1 = entry_params[1];
        let v2 = entry_params[2];
        let v3 = entry_params[3];
        let v4 = entry_params[4];

        let array_sets = find_array_sets(function, &value_aliases);
        assert_eq!(array_sets.len(), 5, "five array_set instructions expected");

        for (block_id, idx, source, alias_set) in &array_sets {
            let dominates = super::some_inc_rc_dominates(
                &mut dom_tree,
                &inc_rc_locations,
                alias_set,
                *block_id,
                *idx,
            );
            let (expected, label) = if *source == v0 {
                (true, "v0: same-block earlier inc_rc")
            } else if *source == v1 {
                (true, "v1: cross-block dominator inc_rc")
            } else if *source == v2 {
                (false, "v2: inc_rc in sibling branch")
            } else if *source == v3 {
                (false, "v3: same-block later inc_rc")
            } else if *source == v4 {
                (false, "v4: no inc_rc")
            } else {
                panic!("unexpected array_set source {source:?}");
            };
            assert_eq!(
                dominates, expected,
                "{label}: expected dominates={expected}, got {dominates}"
            );
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
        let value_aliases = collect_value_aliases(function);
        let (block, idx, _source, alias_set, inst_id) =
            find_array_set_with_id(function, &value_aliases).expect("array_set present");

        let array_operand_uses = super::collect_array_operand_uses(function);
        let has_use = super::has_reachable_aliased_use(
            function,
            &array_operand_uses,
            &alias_set,
            inst_id,
            block,
            idx,
        );
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
        let value_aliases = collect_value_aliases(function);
        let (block, idx, _source, alias_set, inst_id) =
            find_array_set_with_id(function, &value_aliases).expect("array_set present");

        let array_operand_uses = super::collect_array_operand_uses(function);
        let has_use = super::has_reachable_aliased_use(
            function,
            &array_operand_uses,
            &alias_set,
            inst_id,
            block,
            idx,
        );
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
        let value_aliases = collect_value_aliases(function);
        let (block, idx, _source, alias_set, inst_id) =
            find_array_set_with_id(function, &value_aliases).expect("array_set present");

        let array_operand_uses = super::collect_array_operand_uses(function);
        let has_use = super::has_reachable_aliased_use(
            function,
            &array_operand_uses,
            &alias_set,
            inst_id,
            block,
            idx,
        );
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
        let value_aliases = collect_value_aliases(function);
        let (block, idx, source, alias_set, inst_id) =
            find_array_set_with_id(function, &value_aliases).expect("array_set present");

        // alias_set should include v0 (function param), v1 (= source), and
        // v2 (sibling param threaded the same v0 by b0's jmp).
        let v0 = function.dfg.block_parameters(function.entry_block())[0];
        let v2_param = function.dfg.block_parameters(block)[1];
        assert!(alias_set.contains(&v0), "alias-set must include v0");
        assert!(alias_set.contains(&source), "alias-set must include the source v1");
        assert!(alias_set.contains(&v2_param), "alias-set must include the sibling param v2");

        // The forward walk catches `array_get v2` as an aliased read.
        let array_operand_uses = super::collect_array_operand_uses(function);
        let has_use = super::has_reachable_aliased_use(
            function,
            &array_operand_uses,
            &alias_set,
            inst_id,
            block,
            idx,
        );
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
        let value_aliases = collect_value_aliases(function);
        let (block, idx, _source, alias_set, inst_id) =
            find_array_set_with_id(function, &value_aliases).expect("array_set present");

        let array_operand_uses = super::collect_array_operand_uses(function);
        let has_use = super::has_reachable_aliased_use(
            function,
            &array_operand_uses,
            &alias_set,
            inst_id,
            block,
            idx,
        );
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
        let value_aliases = collect_value_aliases(function);
        let (block, idx, _source, alias_set, inst_id) =
            find_array_set_with_id(function, &value_aliases).expect("array_set present");

        let array_operand_uses = super::collect_array_operand_uses(function);
        let has_use = super::has_reachable_aliased_use(
            function,
            &array_operand_uses,
            &alias_set,
            inst_id,
            block,
            idx,
        );
        assert!(
            !has_use,
            "b2's v5 is rebound to v4 (the array_set's result, excluded from alias-set), so it is killed and array_get v5 is not aliased"
        );
    }

    fn find_array_set_with_id(
        function: &Function,
        value_aliases: &super::HashMap<ValueId, im::HashSet<ValueId>>,
    ) -> Option<(super::BasicBlockId, usize, ValueId, im::HashSet<ValueId>, super::InstructionId)>
    {
        for block_id in function.reachable_blocks() {
            for (idx, instruction_id) in function.dfg[block_id].instructions().iter().enumerate() {
                if let Instruction::ArraySet { array, .. } = function.dfg[*instruction_id] {
                    let [result] = function.dfg.instruction_result(*instruction_id);
                    let alias_set = alias_set_for(value_aliases, array, result);
                    return Some((block_id, idx, array, alias_set, *instruction_id));
                }
            }
        }
        None
    }

    fn find_array_sets(
        function: &Function,
        value_aliases: &super::HashMap<ValueId, im::HashSet<ValueId>>,
    ) -> Vec<(super::BasicBlockId, usize, ValueId, im::HashSet<ValueId>)> {
        let mut out = Vec::new();
        for block_id in function.reachable_blocks() {
            for (idx, instruction_id) in function.dfg[block_id].instructions().iter().enumerate() {
                if let Instruction::ArraySet { array, .. } = function.dfg[*instruction_id] {
                    let [result] = function.dfg.instruction_result(*instruction_id);
                    let alias_set = alias_set_for(value_aliases, array, result);
                    out.push((block_id, idx, array, alias_set));
                }
            }
        }
        out
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
