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
        let post_order = PostOrder::with_cfg(&cfg);
        let mut dom_tree = DominatorTree::with_cfg_and_post_order(&cfg, &post_order);
        let inc_rc_locations = collect_inc_rc_locations(self);
        let array_params_by_block = collect_array_params_by_block(self);

        for block_id in self.reachable_blocks() {
            for (idx, instruction_id) in self.dfg[block_id].instructions().iter().enumerate() {
                let Instruction::ArraySet { array, .. } = self.dfg[*instruction_id] else {
                    continue;
                };
                let [result] = self.dfg.instruction_result(*instruction_id);

                let alias_set = compute_alias_set(self, &cfg, array, result);

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
                    &array_params_by_block,
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

/// For each block, the set of its array-typed parameters. The reachable-use
/// walk uses this to "kill" alias-set members from the live use-set when the
/// walk crosses into a block where the member is a parameter — the parameter
/// is re-bound by the predecessor's terminator argument, so the *name* refers
/// to a different value in this block (and its dominated descendants) than to
/// the array_set's pre-mutation source.
fn collect_array_params_by_block(
    function: &Function,
) -> HashMap<BasicBlockId, im::HashSet<ValueId>> {
    let mut out: HashMap<BasicBlockId, im::HashSet<ValueId>> = HashMap::default();
    for block_id in function.reachable_blocks() {
        let mut set = im::HashSet::new();
        for &param in function.dfg.block_parameters(block_id) {
            if function.dfg.type_of_value(param).contains_an_array() {
                set.insert(param);
            }
        }
        if !set.is_empty() {
            out.insert(block_id, set);
        }
    }
    out
}

/// Forward CFG walk from after the `array_set` looking for a non-terminator
/// instruction that reads a value still in the alias use-set.
///
/// **Use-set evolution.** Starts as `alias_set` and only shrinks: when the
/// walk crosses *into* a block where an alias-set member is a parameter, that
/// member is dropped from the use-set for this path onward. This models the
/// re-binding that happens at block boundaries — a `jmp b(v_new)` to a block
/// whose parameter is in our alias-set means the parameter's name now refers
/// to `v_new` in that block (and its dominated descendants), not to the
/// array_set's source.
///
/// **What counts as a use.** Only non-terminator operands. Terminator
/// arguments are the legitimate threading mechanism the invariant relies on:
/// `jmp b(v_alias)` is how the post-mutation value reaches the next block
/// where it is re-bound to that block's parameter. Flagging this would defeat
/// the invariant's whole purpose.
///
/// The original `array_set` itself is also skipped, in case a cycle re-enters
/// its block — it is, by construction, a use of its own source, not a hazard.
///
/// **Cycle detection.** Re-visiting a block with a use-set that is a subset
/// of what we already explored at that block adds no new information. We
/// record the *largest* use-set seen per block and skip on subset matches.
fn has_reachable_aliased_use(
    function: &Function,
    array_params_by_block: &HashMap<BasicBlockId, im::HashSet<ValueId>>,
    alias_set: &im::HashSet<ValueId>,
    array_set_id: InstructionId,
    array_set_block: BasicBlockId,
    array_set_idx: usize,
) -> bool {
    let mut visited: HashMap<BasicBlockId, im::HashSet<ValueId>> = HashMap::default();

    // (block, start_idx, use_set_on_entry)
    //
    // `start_idx > 0` denotes the very first frame, where we continue inside
    // the array_set's own block past the array_set instruction itself. Block
    // kills are not applied on that frame because we are mid-block (kills
    // would have already been applied when the array_set's block was first
    // entered from a predecessor; we are picking up after that point).
    let mut worklist: Vec<(BasicBlockId, usize, im::HashSet<ValueId>)> =
        vec![(array_set_block, array_set_idx + 1, alias_set.clone())];

    while let Some((block, start_idx, mut use_set)) = worklist.pop() {
        if start_idx == 0
            && let Some(kills) = array_params_by_block.get(&block)
        {
            use_set = use_set.relative_complement(kills.clone());
        }

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

        for inst_id in function.dfg[block].instructions().iter().skip(start_idx) {
            if *inst_id == array_set_id {
                continue;
            }
            let mut hit = false;
            function.dfg[*inst_id].for_each_value(|v| {
                hit |= use_set.contains(&v);
            });
            if hit {
                return true;
            }
        }

        for succ in function.dfg[block].successors() {
            worklist.push((succ, 0, use_set.clone()));
        }
    }
    false
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

        let entry_params = function.dfg.block_parameters(function.entry_block());
        let v0 = entry_params[0];
        let v1 = entry_params[1];
        let v2 = entry_params[2];
        let v3 = entry_params[3];
        let v4 = entry_params[4];

        let array_sets = find_array_sets(function, &cfg);
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
        let cfg = ControlFlowGraph::with_function(function);
        let array_params_by_block = super::collect_array_params_by_block(function);
        let (block, idx, _source, alias_set, inst_id) =
            find_array_set_with_id(function, &cfg).expect("array_set present");

        let has_use = super::has_reachable_aliased_use(
            function,
            &array_params_by_block,
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
        let cfg = ControlFlowGraph::with_function(function);
        let array_params_by_block = super::collect_array_params_by_block(function);
        let (block, idx, _source, alias_set, inst_id) =
            find_array_set_with_id(function, &cfg).expect("array_set present");

        let has_use = super::has_reachable_aliased_use(
            function,
            &array_params_by_block,
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
        let cfg = ControlFlowGraph::with_function(function);
        let array_params_by_block = super::collect_array_params_by_block(function);
        let (block, idx, _source, alias_set, inst_id) =
            find_array_set_with_id(function, &cfg).expect("array_set present");

        let has_use = super::has_reachable_aliased_use(
            function,
            &array_params_by_block,
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

    fn find_array_set_with_id(
        function: &Function,
        cfg: &super::ControlFlowGraph,
    ) -> Option<(super::BasicBlockId, usize, ValueId, im::HashSet<ValueId>, super::InstructionId)>
    {
        for block_id in function.reachable_blocks() {
            for (idx, instruction_id) in function.dfg[block_id].instructions().iter().enumerate() {
                if let Instruction::ArraySet { array, .. } = function.dfg[*instruction_id] {
                    let [result] = function.dfg.instruction_result(*instruction_id);
                    let alias_set = super::compute_alias_set(function, cfg, array, result);
                    return Some((block_id, idx, array, alias_set, *instruction_id));
                }
            }
        }
        None
    }

    fn find_array_sets(
        function: &Function,
        cfg: &super::ControlFlowGraph,
    ) -> Vec<(super::BasicBlockId, usize, ValueId, im::HashSet<ValueId>)> {
        let mut out = Vec::new();
        for block_id in function.reachable_blocks() {
            for (idx, instruction_id) in function.dfg[block_id].instructions().iter().enumerate() {
                if let Instruction::ArraySet { array, .. } = function.dfg[*instruction_id] {
                    let [result] = function.dfg.instruction_result(*instruction_id);
                    let alias_set = super::compute_alias_set(function, cfg, array, result);
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
