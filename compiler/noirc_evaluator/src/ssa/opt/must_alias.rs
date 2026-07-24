//! Must-alias analysis based on a standard dataflow analysis over allocation sites.
//!
//! Consumes the result of [crate::ssa::opt::alias_analysis::AliasAnalysis]
//! may-alias analysis and computes, for every reference-typed SSA value,
//! the static `Allocate` instruction it originates from. Two values that
//! provably originate from the same single-firing site must-alias.
//!
//! ACIR/Brillig differences within this pass:
//!   - No impact, it works equally on ACIR or Brillig
//!
//! Conditions:
//!   - Precondition: May-Alias analysis must have been run before. Typically the must-alias analysis
//!     will be trigger by the may-alias analysis itself.
//!
//! Relevance to other passes:
//!   - Defunctionalize removes first-class function values from the program which skip conservative approach
//!     on unresolved function. It should be done before, but is not mandatory.
//!
//! Implementation details:
//!
//! The analysis is inter-procedural, context-insensitive and field-insensitive.
//!
//! 1. **May-alias** — [`AliasAnalysis::analyze`] builds the union-find of
//!    alias classes and the points-to relation. It must be done outside this module.
//!    It is consumed read-only here.
//!
//! 2. **Preprocess** — see [`preprocess_multiple_allocations`].
//!    Identifies allocation sites and functions that may be executed multiple times per execution.
//!
//! 3. **Joint dataflow** — monotone fixed point over:
//!    - per-value allocation site ([`MustAliasAnalysis::value_sites`]),
//!    - per-alias-class pointee site ([`MustAliasAnalysis::points_to_sites`]),
//!    - per-function summary ([`Summary`]) with `param_sites` and `return_sites`.
//!
//!    The analysis is designed over a two-level work list:
//!    - an inner loop over blocks within one function, converge when per-value allocation sites stabilize.
//!    - an outer loop over functions, converge when function summaries and pointee sites stabilize.
//!
//!    Indeed, a function modifies only the sites of it own values, and blocks naturally propagate value sites to their successors,
//!    so a work list of blocks for computing the per-value allocation sites works well.
//!    When a pointee information is modified, the whole function needs to be re-processed because
//!    a store may impact a load in any other block.
//!
//! 4. **Query** — `MustAliasAnalysis::must_alias` is true iff both values
//!    carry the same `Known(s)` (single-firing site).
//!    `cannot_equal` queries benefit from `Multiple(s)`
//!
//! # Lattice
//!
//! We use a basic and shallow lattice: Bottom -> Allocation Site -> Top, that we extend with:
//! - External: entry point inputs, which cannot mix with local allocations.
//! - Multiple: allocations done by a single site, but potentially executed multiple times.
//!   A `Multiple(site)` cannot mix with another `site` (either `Known` or `Multiple`).
//!
//! ```text
//!                NoAllocation                (top, conservative)
//!               /     |       \
//!     Multiple(s)  Multiple(t)  External    (multi-firing sites, opaque external cells)
//!         |            |
//!      Known(s)     Known(t)                (single-firing sites)
//!               \     |     /
//!                  Undef                    (bottom, unobserved)
//! ```
//!
//! `Known(s) ⊔ Known(s) = Known(s)`. Two distinct sites join to `NoAllocation`.
//! `Known(s) ⊔ Multiple(s) = Multiple(s)`. `External` is incomparable to any
//! `Known/Multiple` site (joins to `NoAllocation`).

use std::collections::VecDeque;

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        call_graph::CallGraph,
        cfg::ControlFlowGraph,
        function::{Function, FunctionId},
        instruction::{Instruction, Intrinsic, TerminatorInstruction},
        post_order::PostOrder,
        value::{Value, ValueId},
    },
    opt::alias_analysis::{AliasAnalysis, GlobalValueId},
    opt::unrolling::{LoopOrder, Loops},
    ssa_gen::Ssa,
};

// =========================================================================
// Lattice
// =========================================================================

/// Allocation-site lattice. See the module-level docs for the partial order.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub(crate) enum AllocationLattice {
    /// Bottom: this value has not been touched yet by the dataflow.
    Undef,
    /// The value originates from outside the analyzed SSA (e.g. an entry
    /// point's reference parameter). It cannot interfere with local allocations.
    External,
    /// The value originates from a single-firing static `Allocate` instruction.
    /// Two values both `Known(s)` with the same `s` must-alias.
    Known(GlobalValueId),
    /// The value originates from an `Allocate` instruction that may fire
    /// more than once per execution (e.g in a loop or in a recursive function).
    Multiple(GlobalValueId),
    /// Top: the value may originate from any site or none.
    NoAllocation,
}

impl AllocationLattice {
    /// Monotone and commutative join operator over the lattice.
    pub(crate) fn join(self, other: Self) -> Self {
        use AllocationLattice::*;
        match (self, other) {
            // `Undef` is bottom: identity for join.
            (Undef, x) | (x, Undef) => x,
            // `NoAllocation` is top: absorbing for join.
            (NoAllocation, _) | (_, NoAllocation) => NoAllocation,
            // `External` only joins with itself.
            (External, External) => External,
            (External, _) | (_, External) => NoAllocation,
            // Same single-firing site.
            (Known(s), Known(t)) if s == t => Known(s),
            // Same multi-firing site.
            (Multiple(s), Multiple(t)) if s == t => Multiple(s),
            // Same site, one single-firing one multi-firing: `Multiple` dominates.
            (Known(s), Multiple(t)) | (Multiple(s), Known(t)) if s == t => Multiple(s),
            // Different sites across `Known`/`Multiple`: precision lost.
            _ => NoAllocation,
        }
    }

    /// True if `self` and `other` cannot refer to the same runtime cell.
    /// Used by may-alias as a cheap site-identity filter (distinct sites
    /// can never alias; `External` and any `Known/Multiple` cannot alias).
    ///
    /// Sound: when this returns `true` the two values are guaranteed to
    /// never alias at runtime.
    /// Conservative: `Undef` and `NoAllocation` carry no site information,
    /// so any pair involving them returns `false`.
    pub(crate) fn cannot_equal(self, other: Self) -> bool {
        use AllocationLattice::*;
        match (self, other) {
            // Bottom or top on either side: no site identity to compare.
            (Undef, _) | (_, Undef) => false,
            (NoAllocation, _) | (_, NoAllocation) => false,
            // External is incomparable to any tracked site, but
            // two `External` values may refer to the same external cell.
            (External, External) => false,
            (External, Known(_) | Multiple(_)) | (Known(_) | Multiple(_), External) => true,
            // Same static site (single or multi-firing): may share a cell.
            (Known(s), Known(t)) if s == t => false,
            (Multiple(s), Multiple(t)) if s == t => false,
            (Known(s), Multiple(t)) | (Multiple(s), Known(t)) if s == t => false,
            // Distinct static sites: distinct cells.
            _ => true,
        }
    }
}

// =========================================================================
// Per-function summary
// =========================================================================

/// Inter-procedural interface of one function.
///
/// `param_sites[i]` is the join of `value_sites[arg_i]` at every call site
/// of this function.
/// `return_sites[i]` is the join of `value_sites[ret_i]` at every `Return` point.
#[derive(Default, Clone)]
pub(crate) struct Summary {
    pub(crate) param_sites: Vec<AllocationLattice>,
    pub(crate) return_sites: Vec<AllocationLattice>,
}

// =========================================================================
// Analysis state
// =========================================================================

pub(crate) struct MustAliasAnalysis {
    /// Allocation site, per value. Not in the map means `Undef`.
    value_sites: HashMap<GlobalValueId, AllocationLattice>,

    /// Allocation sites of Points-to alias-class root
    /// It is the join of any site ever stored at an address.
    /// The alias-class root represents all the values ever stored at a same address
    /// and is computed during the may-alias analysis.
    points_to_sites: HashMap<GlobalValueId, AllocationLattice>,

    /// Per-function summary.
    summaries: HashMap<FunctionId, Summary>,

    /// Allocates defined inside a loop block.
    loop_allocates: HashSet<GlobalValueId>,

    /// Functions called multiple times; in a recursion context, in a loop, or via multiple call sites
    untrusted_functions: HashSet<FunctionId>,

    /// Callees whose body is available to this analysis.
    /// A `Call` to a `Value::Function` outside this set (single-function scope,
    /// or a callee absent in a partial SSA) cannot get a summary and is
    /// conservatively handled
    in_scope_functions: HashSet<FunctionId>,

    /// For each alias-class representative, gives the set of functions
    /// that own a value in that class. When a `points_to_sites` entry grows,
    /// only these functions can observe it through a `Load`, and are re-queued.
    /// Precomputed once from the may-alias input.
    class_functions: HashMap<GlobalValueId, HashSet<FunctionId>>,
}

/// Track inter-procedural propagation changes in [`MustAliasAnalysis::analyze_function`].
#[derive(Default)]
struct InterProceduralChanges {
    /// Functions to re-queue after this pass: callees whose summary
    /// `param_sites` changed, and functions that can observe an update of
    /// `points_to_sites` entry.
    updated_functions: HashSet<FunctionId>,
    /// True if the analyzed function's `return_sites` was updated.
    return_sites: bool,
}

impl MustAliasAnalysis {
    // ---------------------------------------------------------------------
    // Public API
    // ---------------------------------------------------------------------

    /// Run the must-alias analysis over the whole program.
    /// It requires a previously computed `aliases` as input.
    pub(crate) fn analyze(ssa: &Ssa, aliases: &AliasAnalysis) -> Self {
        let MultipleSite { loop_allocates, untrusted_functions } =
            preprocess_multiple_allocations(ssa);

        let mut analysis = Self {
            value_sites: HashMap::default(),
            points_to_sites: HashMap::default(),
            summaries: HashMap::default(),
            loop_allocates,
            untrusted_functions,
            in_scope_functions: ssa.functions.keys().copied().collect(),
            class_functions: build_class_functions(aliases),
        };

        // Entry points are reached from outside the SSA, so their reference
        // parameters (and everything behind them) are `External`. Seeding runs
        // before the work list is built, so the re-queue flags are discarded.
        let mut seed_changes = InterProceduralChanges::default();
        for function in ssa.functions.values() {
            if ssa.is_entry_point(function.id()) {
                analysis.seed_entry_externals(function, aliases, &mut seed_changes);
            }
        }

        analysis.run_dataflow(ssa, aliases);
        analysis
    }

    /// Run the must-alias analysis over a single function in isolation, with
    /// every call treated as an opaque sink. Less precise than [`Self::analyze`]
    /// but usable when the whole SSA is unavailable. `aliases` must already be
    /// built for the same single function.
    pub(crate) fn analyze_single_function(function: &Function, aliases: &AliasAnalysis) -> Self {
        let MultipleSite { loop_allocates, untrusted_functions } =
            preprocess_single_function(function);

        let mut analysis = Self {
            value_sites: HashMap::default(),
            points_to_sites: HashMap::default(),
            summaries: HashMap::default(),
            loop_allocates,
            untrusted_functions,
            // No callee bodies are in scope: every `Call` is an opaque sink.
            in_scope_functions: HashSet::default(),
            class_functions: build_class_functions(aliases),
        };

        // A single analyzed function is always its own entry point. Seeding
        // runs before the dataflow loop, so its re-queue flags are discarded.
        analysis.seed_entry_externals(function, aliases, &mut InterProceduralChanges::default());
        analysis.run_dataflow_single(function, aliases);
        analysis
    }

    // The query methods below (`must_alias`, `cannot_equal`, `known_site`) are
    // the natural public interface of this analysis.
    // They are exercised only by the unit tests for now.

    /// True iff `a` and `b` definitely refer to the same runtime cell.
    /// Holds only when both values share the same `Known(s)`
    #[cfg(test)]
    pub(crate) fn must_alias(&self, a: GlobalValueId, b: GlobalValueId) -> bool {
        if a == b {
            return true;
        }
        match (self.known_site(a), self.known_site(b)) {
            (Some(sa), Some(sb)) => sa == sb,
            _ => false,
        }
    }

    #[cfg(test)]
    pub(crate) fn cannot_equal(&self, a: GlobalValueId, b: GlobalValueId) -> bool {
        if a == b {
            return false;
        }
        let site_a = self.get_site(a);
        let site_b = self.get_site(b);
        site_a.cannot_equal(site_b)
    }

    /// Returns the known allocation site for `value`, if any.
    #[cfg(test)]
    pub(crate) fn known_site(&self, value: GlobalValueId) -> Option<GlobalValueId> {
        match self.get_site(value) {
            AllocationLattice::Known(site) => Some(site),
            _ => None,
        }
    }

    /// Read `value_sites[value]`, defaulting to `Undef` if absent.
    pub(crate) fn get_site(&self, value: GlobalValueId) -> AllocationLattice {
        self.value_sites.get(&value).copied().unwrap_or(AllocationLattice::Undef)
    }

    /// Returns the allocation sites built by the must_alias analysis.
    pub(crate) fn allocation_sites(self) -> HashMap<GlobalValueId, AllocationLattice> {
        self.value_sites
    }

    // ---------------------------------------------------------------------
    // Dataflow analysis
    // ---------------------------------------------------------------------

    /// Record that `points_to_sites[root]` grew: flag every function owning a
    /// value in that alias class for re-queue, since each can observe the new
    /// pointee through a `Load`. Includes the function that grew it, covering
    /// loads on its own loop back-edges.
    fn enqueue_class_readers(&self, root: GlobalValueId, changes: &mut InterProceduralChanges) {
        if let Some(functions) = self.class_functions.get(&root) {
            changes.updated_functions.extend(functions.iter().copied());
        }
    }

    /// Outer work list over `FunctionId`. Repeatedly invokes
    /// [`Self::analyze_function`] until no summary or global state changes.
    fn run_dataflow(&mut self, ssa: &Ssa, aliases: &AliasAnalysis) {
        let call_graph = CallGraph::from_ssa_partial(ssa);
        let (sccs, _) = call_graph.sccs();
        let callers = call_graph.callers();

        // `sccs()` lists functions in reverse topological order:
        // Callees before  their callers, so that callee summaries
        // are computed before their callers consume them.
        let mut work_list: VecDeque<FunctionId> = sccs.iter().flatten().copied().collect();
        let mut in_work_list: HashSet<FunctionId> = work_list.iter().copied().collect();

        while let Some(fid) = work_list.pop_front() {
            in_work_list.remove(&fid);
            let Some(function) = ssa.functions.get(&fid) else { continue };

            let changes = self.analyze_function(function, aliases);
            // Re-queue every function flagged by the pass
            for updated_function in changes.updated_functions {
                if in_work_list.insert(updated_function) {
                    work_list.push_back(updated_function);
                }
            }

            // This function's `return_sites` was updated; re-queue every caller
            // so they can update the results at their call sites.
            if changes.return_sites
                && let Some(my_callers) = callers.get(&fid)
            {
                for &caller in my_callers.keys() {
                    if in_work_list.insert(caller) {
                        work_list.push_back(caller);
                    }
                }
            }
        }
    }

    /// Single-function counterpart to [`Self::run_dataflow`]. With no in-scope
    /// callees or callers, the only inter-block feedback is `points_to_sites`
    /// growth, so re-run the intra-procedural fixed point until global memory
    /// state stops growing.
    fn run_dataflow_single(&mut self, function: &Function, aliases: &AliasAnalysis) {
        let mut changes = InterProceduralChanges {
            updated_functions: std::iter::once(function.id()).collect(),
            return_sites: false,
        };
        while !changes.updated_functions.is_empty() {
            changes = self.analyze_function(function, aliases);
        }
    }

    /// Run an intra-procedural fixed point dataflow analysis over `function`'s blocks.
    ///
    /// Standard fixed point analysis on function's blocks:
    /// 1. Meets predecessors' terminator sites into the block's params.
    /// 2. Applies per-instruction transfers in the block body.
    /// 3. Enqueues successors when either step grew lattice state.
    ///
    /// Returns what changed at the inter-procedural boundary so the outer
    /// driver can re-queue affected callers and callees.
    fn analyze_function(
        &mut self,
        function: &Function,
        aliases: &AliasAnalysis,
    ) -> InterProceduralChanges {
        // Inter-procedural changes
        let mut changes = InterProceduralChanges::default();
        let cfg = ControlFlowGraph::with_function(function);

        // Seed the entry block's parameters from the current `summaries[function].param_sites`.
        self.seed_entry_block_params(function);
        // Seed the work list with reverse post order for better efficiency (compute predecessors before successors).
        let mut work_list: VecDeque<BasicBlockId> =
            PostOrder::with_cfg(&cfg).into_vec_reverse().into();
        let mut in_work_list: HashSet<BasicBlockId> = work_list.iter().copied().collect();

        while let Some(block_id) = work_list.pop_front() {
            in_work_list.remove(&block_id);
            let mut site_update = self.meet_predecessors(function, block_id, &cfg);
            site_update |= self.transfer_block(function, block_id, aliases, &mut changes);

            if site_update {
                for successor in cfg.successors(block_id) {
                    if in_work_list.insert(successor) {
                        work_list.push_back(successor);
                    }
                }
            }
        }

        changes
    }

    /// Apply the transfer function of every instruction in the block,
    /// mutating `value_sites` / `points_to_sites` in place and joining
    /// callee summaries at `Call` sites.
    ///
    /// Returns `true` if any block-local `value_sites` entry grew.
    /// Global growth (in `points_to_sites` or in any function summary)
    /// is recorded via `changes` instead, so the outer work list
    /// can re-queue the affected functions.
    fn transfer_block(
        &mut self,
        function: &Function,
        block_id: BasicBlockId,
        aliases: &AliasAnalysis,
        changes: &mut InterProceduralChanges,
    ) -> bool {
        // Track whether `value_sites` is updated.
        let mut update_sites = false;

        let instructions: Vec<_> = function.dfg[block_id].instructions().to_vec();

        for inst_id in instructions {
            match &function.dfg[inst_id] {
                Instruction::Allocate => {
                    let result = function.dfg.instruction_result::<1>(inst_id)[0];
                    update_sites |= self.allocate(function, GlobalValueId::new(function, result));
                }
                Instruction::Load { address } => {
                    let address = *address;
                    let result = function.dfg.instruction_result::<1>(inst_id)[0];
                    // Join `result` site with the site of the values that the `address`
                    // may points to (field-insensitive)
                    update_sites |= self.load_from_reference(
                        GlobalValueId::new(function, address),
                        &[GlobalValueId::new(function, result)],
                        aliases,
                    );
                }
                Instruction::Store { address, value } => {
                    let address = *address;
                    let value = *value;
                    // `value_sites` is not updated here, so we do not modify `update_sites`
                    // `value_sites` will be indirectly updated through the points-to sites that are tracked in the `changes`
                    let address_root = aliases.class_root(GlobalValueId::new(function, address));
                    // Join `address_root` points-to site with `value` site.
                    self.write_to_pointee(
                        address_root,
                        &[GlobalValueId::new(function, value)],
                        changes,
                    );
                }
                Instruction::Call { func, arguments } => {
                    let callee_value_id = *func;
                    let arguments: Vec<_> =
                        arguments.iter().map(|&v| GlobalValueId::new(function, v)).collect();
                    let results: Vec<_> = function
                        .dfg
                        .instruction_results(inst_id)
                        .iter()
                        .map(|&v| GlobalValueId::new(function, v))
                        .collect();
                    update_sites |= self.transfer_call(
                        function,
                        callee_value_id,
                        &arguments,
                        &results,
                        aliases,
                        changes,
                    );
                }
                Instruction::MakeArray { elements, .. } => {
                    let array = function.dfg.instruction_result::<1>(inst_id)[0];
                    let array_id = GlobalValueId::new(function, array);
                    let elements: Vec<_> =
                        elements.iter().map(|&v| GlobalValueId::new(function, v)).collect();
                    // Same as Store: join `array_root` points-to site with all the `elements` sites.
                    let array_root = aliases.class_root(array_id);
                    self.write_to_pointee(array_root, &elements, changes);
                }
                Instruction::ArraySet { array, value, .. } => {
                    let array_id = GlobalValueId::new(function, *array);
                    let value_id = GlobalValueId::new(function, *value);
                    let new_array = function.dfg.instruction_result::<1>(inst_id)[0];
                    let new_array_id = GlobalValueId::new(function, new_array);
                    let new_array_root = aliases.class_root(new_array_id);
                    let array_root = aliases.class_root(array_id);

                    // The new array's pointee site is the join of the
                    // old container's pointee site (unchanged slots) and
                    // `value`'s site (the changed slot).
                    let old_pointee = self
                        .points_to_sites
                        .get(&array_root)
                        .copied()
                        .unwrap_or(AllocationLattice::Undef);
                    // The new_array inherits the old array site
                    self.join_into_pointee_site(new_array_root, old_pointee, changes);
                    // Same as Store: join `new_array_root` points-to site with `value_id` site.
                    self.write_to_pointee(new_array_root, &[value_id], changes);
                }
                Instruction::ArrayGet { array, .. } => {
                    let array_id = GlobalValueId::new(function, *array);
                    let result = function.dfg.instruction_result::<1>(inst_id)[0];
                    let result_id = GlobalValueId::new(function, result);
                    // Same as Load: join with the site of array's pointee class
                    update_sites |= self.load_from_reference(array_id, &[result_id], aliases);
                }
                Instruction::IfElse { then_value, else_value, .. } => {
                    // Join with the sites from both branches.
                    if function.dfg.type_of_value(*then_value).contains_reference() {
                        let then_id = GlobalValueId::new(function, *then_value);
                        let else_id = GlobalValueId::new(function, *else_value);
                        let result = function.dfg.instruction_result::<1>(inst_id)[0];
                        let result_id = GlobalValueId::new(function, result);
                        let merged = self.get_site(then_id).join(self.get_site(else_id));
                        update_sites |= self.join_into_site(result_id, merged);
                    }
                }

                // Instructions that do not modify references.
                Instruction::Binary(_)
                | Instruction::Cast(_, _)
                | Instruction::Not(_)
                | Instruction::Truncate { .. }
                | Instruction::Constrain(_, _, _)
                | Instruction::ConstrainNotEqual(_, _, _)
                | Instruction::RangeCheck { .. }
                | Instruction::EnableSideEffectsIf { .. }
                | Instruction::IncrementRc { .. }
                | Instruction::DecrementRc { .. }
                | Instruction::Noop => {}
            }
        }

        // `Return` updates this function's summary, recorded in `changes.return_sites`.
        // `Unreachable` has no effect.
        if let Some(TerminatorInstruction::Return { return_values, .. }) =
            function.dfg[block_id].terminator()
        {
            let return_values: Vec<_> =
                return_values.iter().map(|&v| GlobalValueId::new(function, v)).collect();
            self.transfer_return(function, &return_values, changes);
        }

        update_sites
    }

    /// Meet over predecessors at the entry of `block_id`: for every
    /// predecessor `P` of `block_id`, identify the terminator arguments
    /// flowing to `block_id` and join each `value_sites[arg]` into the
    /// corresponding `value_sites[block_id.param]`.
    ///
    /// Returns `true` if any block-parameter site is updated.
    fn meet_predecessors(
        &mut self,
        function: &Function,
        block_id: BasicBlockId,
        cfg: &ControlFlowGraph,
    ) -> bool {
        let params = function.dfg[block_id].parameters();
        let mut update_param = false;

        for pred in cfg.predecessors(block_id) {
            let terminator =
                function.dfg[pred].terminator().expect("ICE: predecessor block has no terminator");
            let args: &[ValueId] = match terminator {
                TerminatorInstruction::Jmp { destination, arguments, .. } => {
                    debug_assert_eq!(*destination, block_id);
                    arguments
                }
                TerminatorInstruction::JmpIf {
                    then_destination,
                    then_arguments,
                    else_destination,
                    else_arguments,
                    ..
                } => {
                    if *then_destination == block_id {
                        then_arguments
                    } else {
                        debug_assert_eq!(*else_destination, block_id);
                        else_arguments
                    }
                }
                TerminatorInstruction::Return { .. }
                | TerminatorInstruction::Unreachable { .. } => {
                    unreachable!("ICE: non-branching terminator as predecessor of a block")
                }
            };

            debug_assert_eq!(args.len(), params.len());
            for (&arg, &param) in args.iter().zip(params) {
                let arg_site = self.get_site(GlobalValueId::new(function, arg));
                update_param |= self.join_into_site(GlobalValueId::new(function, param), arg_site);
            }
        }

        update_param
    }

    /// Join the current `summaries[function].param_sites` into
    /// `value_sites` for the entry block's parameters.
    /// No-op if function has no summary yet (i.e. no caller has been processed).
    fn seed_entry_block_params(&mut self, function: &Function) {
        let Some(summary) = self.summaries.get(&function.id()) else { return };
        if summary.param_sites.is_empty() {
            return;
        }
        let param_sites = summary.param_sites.clone();
        let params = function.dfg[function.entry_block()].parameters().to_vec();
        debug_assert_eq!(param_sites.len(), params.len());
        for (param, site) in params.into_iter().zip(param_sites) {
            self.join_into_site(GlobalValueId::new(function, param), site);
        }
    }

    /// Seed `External` for an entry-point function's reference parameters and
    /// everything reachable through their points-to chains. Runs before the
    /// dataflow work list exists, so the re-queue flags it produces are
    /// discarded by the caller.
    fn seed_entry_externals(
        &mut self,
        function: &Function,
        aliases: &AliasAnalysis,
        changes: &mut InterProceduralChanges,
    ) {
        let params = function.dfg[function.entry_block()].parameters().to_vec();
        for param in params {
            if !function.dfg.type_of_value(param).contains_reference() {
                continue;
            }
            let param_g = GlobalValueId::new(function, param);
            self.join_into_site(param_g, AllocationLattice::External);
            // Forward it through their points-to, recursively.
            self.mark_pointees_external(param_g, aliases, changes);
        }
    }

    /// Mark `External` the `points_to_sites` entry of `value`'s class and of
    /// every class reachable by dereferencing it.
    fn mark_pointees_external(
        &mut self,
        value: GlobalValueId,
        aliases: &AliasAnalysis,
        changes: &mut InterProceduralChanges,
    ) {
        self.poison_pointee_chain(value, AllocationLattice::External, aliases, changes);
    }

    /// Join `site` into the `points_to_sites` entry of `value`'s class and of
    /// every class reachable by dereferencing it.
    ///
    /// `points_to_sites` is keyed by the *pointer's* class root: a load through
    /// a pointer in class `K` reads `points_to_sites[K]`. To poison every load
    /// reachable from `value` at any depth (e.g. an entry-point parameter, or
    /// an argument escaping into an opaque callee that may mutate any cell it
    /// can reach), we poison `value`'s own class and then walk the may-alias
    /// points-to chain, poisoning each successive level. Stopping at the first
    /// level is unsound: a deeper pointee would keep a stale site even though
    /// the callee could have overwritten it.
    fn poison_pointee_chain(
        &mut self,
        value: GlobalValueId,
        site: AllocationLattice,
        aliases: &AliasAnalysis,
        changes: &mut InterProceduralChanges,
    ) {
        let mut current = aliases.class_root(value);
        let mut seen: HashSet<GlobalValueId> = HashSet::default();
        while seen.insert(current) {
            self.join_into_pointee_site(current, site, changes);
            let Some(pointee) = aliases.pointee(current) else { break };
            current = aliases.class_root(pointee);
        }
    }

    /// Join `site` into `value_sites[value]`.
    /// Returns `true` if the entry was modified.
    /// `Undef` is never inserted into the map.
    fn join_into_site(&mut self, value: GlobalValueId, mut site: AllocationLattice) -> bool {
        if site == AllocationLattice::Undef {
            return false;
        }
        if let Some(previous_site) = self.value_sites.get(&value) {
            site = previous_site.join(site);
            if site == *previous_site {
                return false;
            }
        }
        self.value_sites.insert(value, site);
        true
    }

    /// Join `site` into `points_to_sites[class_root]`. On growth, flag every
    /// function that can observe the new pointee through a `Load` for re-queue.
    /// Pointee-side mirror of [`Self::join_into_site`].
    fn join_into_pointee_site(
        &mut self,
        class_root: GlobalValueId,
        mut site: AllocationLattice,
        changes: &mut InterProceduralChanges,
    ) {
        if site == AllocationLattice::Undef {
            return;
        }
        if let Some(previous_site) = self.points_to_sites.get(&class_root) {
            site = previous_site.join(site);
            if site == *previous_site {
                return;
            }
        }
        self.points_to_sites.insert(class_root, site);
        self.enqueue_class_readers(class_root, changes);
    }

    /// Transfer for `Allocate`: `Known(s)` or `Multiple(s)` depending on the context (loop or recursion)
    /// Returns `true` if it updated `value_sites[result]`.
    fn allocate(&mut self, function: &Function, result: GlobalValueId) -> bool {
        let multiple = self.untrusted_functions.contains(&function.id())
            || self.loop_allocates.contains(&result);
        let site = if multiple {
            AllocationLattice::Multiple(result)
        } else {
            AllocationLattice::Known(result)
        };
        self.join_into_site(result, site)
    }

    /// Transfer for `Call(callee, args) -> results`. Dispatches on the kind
    /// of `callee`:
    ///   - `Value::Function(g)` with `g` in scope: see [`Self::transfer_resolved_call`].
    ///   - `Value::Intrinsic(_)`: see [`Self::transfer_intrinsic_call`].
    ///   - `Value::ForeignFunction { .. }`: no-op (cannot modify references).
    ///   - Anything else: conservative `NoAllocation`.
    ///
    /// Returns `true` if any `value_sites[result]` grew.
    /// Indicates in `changes` whether function summary was updated.
    fn transfer_call(
        &mut self,
        function: &Function,
        callee_value_id: ValueId,
        arguments: &[GlobalValueId],
        results: &[GlobalValueId],
        aliases: &AliasAnalysis,
        changes: &mut InterProceduralChanges,
    ) -> bool {
        match &function.dfg[callee_value_id] {
            Value::Function(callee_id) if self.in_scope_functions.contains(callee_id) => {
                self.transfer_resolved_call(*callee_id, arguments, results, changes)
            }
            Value::Intrinsic(intrinsic) => {
                // Clone `intrinsic` because of the borrow-checker.
                let intrinsic = *intrinsic;
                self.transfer_intrinsic_call(&intrinsic, arguments, results, aliases, changes)
            }
            // Foreign calls cannot receive or return references in Noir, so
            // they have no effect on the must-alias state.
            Value::ForeignFunction { .. } => false,
            // Conservatively mark arguments and results to `NoAllocation`.
            _ => {
                for &arg in arguments {
                    if !function.dfg.type_of_value(arg.value_id()).contains_reference() {
                        continue;
                    }
                    // Mark the argument as `NoAllocation`, but also any other reference
                    // it may points-to, recursively.
                    self.poison_pointee_chain(
                        arg,
                        AllocationLattice::NoAllocation,
                        aliases,
                        changes,
                    );
                }
                let mut update = false;
                for &result in results {
                    if !function.dfg.type_of_value(result.value_id()).contains_reference() {
                        continue;
                    }
                    update |= self.join_into_site(result, AllocationLattice::NoAllocation);
                }
                update
            }
        }
    }

    /// Transfer for a resolved Noir-function call `Call(g, args) -> results`:
    /// join argument sites into `summaries[g].param_sites`,
    /// join `summaries[g].return_sites` into the call's result sites.
    /// Returns `true` if any `value_sites[result]` was updated.
    fn transfer_resolved_call(
        &mut self,
        callee: FunctionId,
        arguments: &[GlobalValueId],
        results: &[GlobalValueId],
        changes: &mut InterProceduralChanges,
    ) -> bool {
        let arg_sites: Vec<_> = arguments.iter().map(|&a| self.get_site(a)).collect();
        let summary = self.summaries.entry(callee).or_default();

        if Self::join_sites_into_summary(&mut summary.param_sites, &arg_sites) {
            changes.updated_functions.insert(callee);
        }

        if summary.return_sites.is_empty() && !results.is_empty() {
            summary.return_sites = vec![AllocationLattice::Undef; results.len()];
        }
        let return_sites = summary.return_sites.clone();

        let mut update = false;
        for (&result, &return_site) in results.iter().zip(&return_sites) {
            update |= self.join_into_site(result, return_site);
        }
        update
    }

    /// Transfer for `Call(intrinsic, args) -> results`.
    ///
    /// - Vector operations might involve references,
    ///   we conservatively assign no allocation site to the output vector.
    /// - `Hint` is identity: the output inherit the input site.
    /// - Other intrinsics do not involve reference.
    ///
    /// Returns `true` if any `value_sites[result]` grew.
    fn transfer_intrinsic_call(
        &mut self,
        intrinsic: &Intrinsic,
        arguments: &[GlobalValueId],
        results: &[GlobalValueId],
        aliases: &AliasAnalysis,
        changes: &mut InterProceduralChanges,
    ) -> bool {
        use Intrinsic::*;
        let mut update_sites = false;
        // Vector intrinsics are creating new vectors (expect for `AsVector`), we conservatively treat them as `NoAllocation`.
        match intrinsic {
            AsVector => {
                let input_site = self.get_site(arguments[0]);
                update_sites |= self.join_into_site(results[1], input_site);
            }
            // (len, vec, elems...) -> (new_len, new_vec).
            VectorPushBack | VectorPushFront => {
                let vector_root = aliases.class_root(arguments[1]);
                self.write_to_pointee(vector_root, &arguments[2..], changes);
                update_sites |= self.join_into_site(results[1], AllocationLattice::NoAllocation);
            }
            // (len, vec, idx, elems...) -> (new_len, new_vec).
            VectorInsert => {
                let vector_root = aliases.class_root(arguments[1]);
                self.write_to_pointee(vector_root, &arguments[3..], changes);
                update_sites |= self.join_into_site(results[1], AllocationLattice::NoAllocation);
            }
            // (len, vec, [idx]) -> (new_len, new_vec, elems...).
            VectorPopBack | VectorRemove => {
                update_sites |= self.load_from_reference(arguments[1], &results[2..], aliases);
                update_sites |= self.join_into_site(results[1], AllocationLattice::NoAllocation);
            }
            // (len, vec) -> (elems..., new_len, new_vec).
            VectorPopFront => {
                let n = results.len();
                update_sites |= self.load_from_reference(arguments[1], &results[..n - 2], aliases);
                update_sites |=
                    self.join_into_site(results[n - 1], AllocationLattice::NoAllocation);
            }
            // `Hint` is an identity function, hiding the input from optimizations.
            Hint(_) => {
                debug_assert_eq!(
                    arguments.len(),
                    results.len(),
                    "ICE: Hint intrinsic must have equal input/output arity"
                );
                for (&arg, &result) in arguments.iter().zip(results) {
                    let input_site = self.get_site(arg);
                    update_sites |= self.join_into_site(result, input_site);
                }
            }
            // Other intrinsics cannot create or transport references.
            _ => {}
        }

        update_sites
    }

    /// Update the allocation site of the pointee class by joining with the sites of `elements`
    fn write_to_pointee(
        &mut self,
        pointee_root: GlobalValueId,
        elements: &[GlobalValueId],
        changes: &mut InterProceduralChanges,
    ) {
        for &elem in elements {
            let elem_site = self.get_site(elem);
            self.join_into_pointee_site(pointee_root, elem_site, changes);
        }
    }

    /// Update allocation site of `elements` by joining with the site of the pointee class
    fn load_from_reference(
        &mut self,
        pointer: GlobalValueId,
        elements: &[GlobalValueId],
        aliases: &AliasAnalysis,
    ) -> bool {
        let class_root = aliases.class_root(pointer);
        let pointee_site =
            self.points_to_sites.get(&class_root).copied().unwrap_or(AllocationLattice::Undef);
        let mut updated = false;
        for &elem in elements {
            updated |= self.join_into_site(elem, pointee_site);
        }
        updated
    }

    /// Transfer for `Return(values)`: join `value_sites[values[i]]` into
    /// `summaries[function.id()].return_sites[i]`. Sets
    /// `changes.return_sites` so the outer worklist re-queues this
    /// function's callers, which will then re-pull the grown return sites
    /// in their `transfer_resolved_call`.
    fn transfer_return(
        &mut self,
        function: &Function,
        values: &[GlobalValueId],
        changes: &mut InterProceduralChanges,
    ) {
        let value_sites: Vec<_> = values.iter().map(|&v| self.get_site(v)).collect();
        let summary = self.summaries.entry(function.id()).or_default();
        if Self::join_sites_into_summary(&mut summary.return_sites, &value_sites) {
            changes.return_sites = true;
        }
    }

    /// Join each `sites[i]` into `summary[i]`.
    /// Returns `true` if any summary is updated.
    fn join_sites_into_summary(
        summary: &mut Vec<AllocationLattice>,
        sites: &[AllocationLattice],
    ) -> bool {
        // A summary can be empty, which means `Undef`.
        if summary.is_empty() && !sites.is_empty() {
            *summary = vec![AllocationLattice::Undef; sites.len()];
        }
        debug_assert_eq!(
            summary.len(),
            sites.len(),
            "ICE: summary slot arity inconsistent with sites"
        );
        let mut update = false;
        for (slot, &site) in summary.iter_mut().zip(sites) {
            let new = slot.join(site);
            if new != *slot {
                *slot = new;
                update = true;
            }
        }
        update
    }
}

// =========================================================================
// Preprocessing
// =========================================================================

/// Outputs of [`preprocess_multiple_allocations`].
/// A 'multiple' allocation site means it can be executed multiple times per program execution.
/// This can happens:
/// - if the allocations is inside a loop, or
/// - if the allocation is inside a function that can be called multiple times
///
/// Note that we do not take into account a function called multiple times in different call sites.
/// This will be handled by a context-sensitive dataflow analysis, but precomputing loops and recursive
/// functions allows us to keep the context-sensitive state small.
pub(crate) struct MultipleSite {
    /// Allocation sites defined in a loop block
    pub(crate) loop_allocates: HashSet<GlobalValueId>,

    /// Transitive closure of functions called in a loop or in a recursion cycle.
    pub(crate) untrusted_functions: HashSet<FunctionId>,
}

/// Compute allocation sites and functions who can be executed multiple times
/// because:
/// - they are part of a loop, or
/// - they belong to a recursion call cycle, or
/// - they are reachable from two or more static call sites (the body then runs
///   more than once per execution, so each of its `Allocate`s yields a distinct
///   cell per call — the same multi-firing situation as a loop).
pub(crate) fn preprocess_multiple_allocations(ssa: &Ssa) -> MultipleSite {
    let call_graph = CallGraph::from_ssa_partial(ssa);
    let (_, recursive) = call_graph.sccs();

    let mut loop_allocates = HashSet::default();
    let mut untrusted_functions = recursive;

    // Static call-site count per callee. Counted directly from the
    // instructions rather than via the call graph because `from_ssa_partial`
    // deduplicates edges, which would miss two call sites in the same caller.
    let mut call_site_counts: HashMap<FunctionId, usize> = HashMap::default();

    // Single walk over each function's loop blocks:
    // collect Allocate results and Call targets
    for function in ssa.functions.values() {
        if untrusted_functions.contains(&function.id()) {
            // The function is already untrusted, as well as its callee and
            // allocation sites, so there is no need to identify its loop or count the calls.
            continue;
        }

        for block_id in loop_blocks(function) {
            for inst_id in function.dfg[block_id].instructions() {
                match &function.dfg[*inst_id] {
                    Instruction::Allocate => {
                        let result = function.dfg.instruction_result::<1>(*inst_id)[0];
                        loop_allocates.insert(GlobalValueId::new(function, result));
                    }
                    Instruction::Call { func, .. } => {
                        if let Value::Function(callee) = &function.dfg[*func] {
                            untrusted_functions.insert(*callee);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Count every static call site across all blocks of this function.
        // A callee already untrusted stays untrusted, so it needs no count.
        for block_id in function.reachable_blocks() {
            for inst_id in function.dfg[block_id].instructions() {
                if let Instruction::Call { func, .. } = &function.dfg[*inst_id]
                    && let Value::Function(callee) = &function.dfg[*func]
                    && !untrusted_functions.contains(callee)
                {
                    *call_site_counts.entry(*callee).or_default() += 1;
                }
            }
        }
    }

    // A callee reached from two or more static call sites runs more than once
    // per execution, so its sites cannot be trusted as single-firing.
    for (callee, count) in call_site_counts {
        if count >= 2 {
            untrusted_functions.insert(callee);
        }
    }

    // Transitive closure: any callee of an 'untrusted' function is de-facto untrusted.
    let untrusted_functions = call_graph.reachable_from(untrusted_functions);

    MultipleSite { loop_allocates, untrusted_functions }
}

/// Single-function counterpart to [`preprocess_multiple_allocations`].
///
/// The calling context is unknown, so the function may itself run more than
/// once per execution, and any callee it invokes is opaque and could
/// transitively re-enter it (recursion through an unknown path). We therefore
/// take the conservative stance: if the function makes any call to a Noir
/// function — resolved or indirect — none of its allocation sites can be
/// trusted as single-firing, so the whole function is untrusted. Otherwise
/// only its loop-resident allocates are multi-firing.
//
// A future refinement (needs call-graph reachability not available here) could
// exempt calls to callees that provably cannot reach this function.
fn preprocess_single_function(function: &Function) -> MultipleSite {
    let mut loop_allocates = HashSet::default();
    let mut untrusted_functions = HashSet::default();

    if makes_any_call(function) {
        untrusted_functions.insert(function.id());
    } else {
        for block_id in loop_blocks(function) {
            for inst_id in function.dfg[block_id].instructions() {
                if matches!(&function.dfg[*inst_id], Instruction::Allocate) {
                    let result = function.dfg.instruction_result::<1>(*inst_id)[0];
                    loop_allocates.insert(GlobalValueId::new(function, result));
                }
            }
        }
    }

    MultipleSite { loop_allocates, untrusted_functions }
}

/// Map alias class roots to the functions of the class.
/// An alias class root represent a set of GlobalValueId that may alias,
/// and a GlobalValueId is a ValueId living in a function.
/// `build_class_functions()` maps the class root to all functions appearing in the class.
fn build_class_functions(aliases: &AliasAnalysis) -> HashMap<GlobalValueId, HashSet<FunctionId>> {
    let mut class_functions: HashMap<GlobalValueId, HashSet<FunctionId>> = HashMap::default();
    for (root, members) in aliases.class_sets() {
        let functions = class_functions.entry(root).or_default();
        for member in members {
            functions.insert(member.function_id());
        }
    }
    class_functions
}

/// True if `function` calls any `Value::Function`
fn makes_any_call(function: &Function) -> bool {
    for block_id in function.reachable_blocks() {
        for inst_id in function.dfg[block_id].instructions() {
            if let Instruction::Call { func, .. } = &function.dfg[*inst_id] {
                match &function.dfg[*func] {
                    Value::Intrinsic(_) | Value::ForeignFunction { .. } => {}
                    _ => return true,
                }
            }
        }
    }
    false
}

/// Set of basic blocks inside a loop body of `function`. An allocate in any
/// of these blocks fires once per iteration, producing a distinct runtime
/// cell each time.
fn loop_blocks(function: &Function) -> HashSet<BasicBlockId> {
    let loops = Loops::find_all(function, LoopOrder::InsideOut);
    loops.yet_to_unroll.into_iter().flat_map(|l| l.blocks.into_iter()).collect()
}

#[cfg(test)]
impl MustAliasAnalysis {
    /// Empty analysis, used by the query tests to seed `value_sites`
    /// directly without going through the dataflow driver.
    fn empty() -> Self {
        Self {
            value_sites: HashMap::default(),
            points_to_sites: HashMap::default(),
            summaries: HashMap::default(),
            loop_allocates: HashSet::default(),
            untrusted_functions: HashSet::default(),
            in_scope_functions: HashSet::default(),
            class_functions: HashMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssa::ssa_gen::Ssa;

    /// Parse a small SSA with `n` allocates, return the parsed SSA and the
    /// `n` `GlobalValueId`s of the allocate results in declaration order.
    /// Used as a cheap source of fresh, distinct `GlobalValueId`s.
    fn ssa_with_allocates(n: usize) -> (Ssa, Vec<GlobalValueId>) {
        let mut src = String::from("acir(inline) fn main f0 {\n  b0():\n");
        for i in 0..n {
            src.push_str(&format!("    v{i} = allocate -> &mut Field\n"));
        }
        src.push_str("    return\n}\n");
        let ssa = Ssa::from_str(&src).unwrap();
        let func = ssa.main();
        let mut allocs = Vec::new();
        for block_id in func.reachable_blocks() {
            for inst_id in func.dfg[block_id].instructions() {
                if matches!(&func.dfg[*inst_id], Instruction::Allocate) {
                    let v = func.dfg.instruction_result::<1>(*inst_id)[0];
                    allocs.push(GlobalValueId::new(func, v));
                }
            }
        }
        assert_eq!(allocs.len(), n);
        (ssa, allocs)
    }

    /// Every lattice point parametrized over two distinct sites `s` and `t`.
    fn lattice_points(s: GlobalValueId, t: GlobalValueId) -> [AllocationLattice; 7] {
        use AllocationLattice::*;
        [Undef, External, NoAllocation, Known(s), Known(t), Multiple(s), Multiple(t)]
    }

    // -----------------------------------------------------------------------
    // Lattice: join properties
    // -----------------------------------------------------------------------

    #[test]
    fn join_is_idempotent() {
        let (_ssa, ids) = ssa_with_allocates(2);
        for x in lattice_points(ids[0], ids[1]) {
            assert_eq!(x.join(x), x, "idempotent at {x:?}");
        }
    }

    #[test]
    fn join_is_commutative() {
        let (_ssa, ids) = ssa_with_allocates(2);
        let points = lattice_points(ids[0], ids[1]);
        for a in points {
            for b in points {
                assert_eq!(a.join(b), b.join(a), "commute {a:?} {b:?}");
            }
        }
    }

    #[test]
    fn join_is_associative() {
        let (_ssa, ids) = ssa_with_allocates(2);
        let points = lattice_points(ids[0], ids[1]);
        for a in points {
            for b in points {
                for c in points {
                    assert_eq!(
                        a.join(b).join(c),
                        a.join(b.join(c)),
                        "associativity at {a:?} {b:?} {c:?}",
                    );
                }
            }
        }
    }

    #[test]
    fn undef_is_join_identity() {
        let (_ssa, ids) = ssa_with_allocates(2);
        for x in lattice_points(ids[0], ids[1]) {
            assert_eq!(AllocationLattice::Undef.join(x), x);
            assert_eq!(x.join(AllocationLattice::Undef), x);
        }
    }

    #[test]
    fn no_allocation_is_join_absorbing() {
        let (_ssa, ids) = ssa_with_allocates(2);
        for x in lattice_points(ids[0], ids[1]) {
            assert_eq!(AllocationLattice::NoAllocation.join(x), AllocationLattice::NoAllocation);
            assert_eq!(x.join(AllocationLattice::NoAllocation), AllocationLattice::NoAllocation);
        }
    }

    #[test]
    fn join_specific_pairs() {
        use AllocationLattice::*;
        let (_ssa, ids) = ssa_with_allocates(2);
        let (s, t) = (ids[0], ids[1]);

        assert_eq!(Known(s).join(Known(s)), Known(s));
        assert_eq!(Known(s).join(Known(t)), NoAllocation);
        assert_eq!(Multiple(s).join(Multiple(s)), Multiple(s));
        assert_eq!(Multiple(s).join(Multiple(t)), NoAllocation);
        assert_eq!(Known(s).join(Multiple(s)), Multiple(s));
        assert_eq!(Known(s).join(Multiple(t)), NoAllocation);
        assert_eq!(External.join(External), External);
        assert_eq!(External.join(Known(s)), NoAllocation);
        assert_eq!(External.join(Multiple(s)), NoAllocation);
    }

    // -----------------------------------------------------------------------
    // Lattice: cannot_equal
    // -----------------------------------------------------------------------

    #[test]
    fn cannot_equal_is_commutative() {
        let (_ssa, ids) = ssa_with_allocates(2);
        let points = lattice_points(ids[0], ids[1]);
        for a in points {
            for b in points {
                assert_eq!(a.cannot_equal(b), b.cannot_equal(a), "{a:?} {b:?}");
            }
        }
    }

    #[test]
    fn cannot_equal_distinct_sites() {
        use AllocationLattice::*;
        let (_ssa, ids) = ssa_with_allocates(2);
        let (s, t) = (ids[0], ids[1]);

        assert!(Known(s).cannot_equal(Known(t)));
        assert!(Multiple(s).cannot_equal(Multiple(t)));
        assert!(Known(s).cannot_equal(Multiple(t)));
        assert!(External.cannot_equal(Known(s)));
        assert!(External.cannot_equal(Multiple(s)));
    }

    #[test]
    fn cannot_equal_same_site_or_unknown() {
        use AllocationLattice::*;
        let (_ssa, ids) = ssa_with_allocates(1);
        let s = ids[0];

        // Same site may share a cell.
        assert!(!Known(s).cannot_equal(Known(s)));
        assert!(!Multiple(s).cannot_equal(Multiple(s)));
        assert!(!Known(s).cannot_equal(Multiple(s)));

        // Two External values may refer to the same external cell.
        assert!(!External.cannot_equal(External));

        // Undef / NoAllocation carry no site information.
        assert!(!Undef.cannot_equal(Known(s)));
        assert!(!Undef.cannot_equal(Undef));
        assert!(!NoAllocation.cannot_equal(Known(s)));
        assert!(!NoAllocation.cannot_equal(NoAllocation));
    }

    // -----------------------------------------------------------------------
    // Queries: must_alias and trusted_site
    // -----------------------------------------------------------------------

    #[test]
    fn must_alias_is_reflexive_even_when_undef() {
        let (_ssa, ids) = ssa_with_allocates(1);
        let analysis = MustAliasAnalysis::empty();
        assert!(analysis.must_alias(ids[0], ids[0]));
    }

    #[test]
    fn must_alias_holds_for_same_known_site() {
        let (_ssa, ids) = ssa_with_allocates(3);
        let (site, a, b) = (ids[0], ids[1], ids[2]);
        let mut analysis = MustAliasAnalysis::empty();
        analysis.value_sites.insert(a, AllocationLattice::Known(site));
        analysis.value_sites.insert(b, AllocationLattice::Known(site));
        assert!(analysis.must_alias(a, b));
    }

    #[test]
    fn must_alias_rejects_distinct_known_sites() {
        let (_ssa, ids) = ssa_with_allocates(4);
        let (site_s, site_t, a, b) = (ids[0], ids[1], ids[2], ids[3]);
        let mut analysis = MustAliasAnalysis::empty();
        analysis.value_sites.insert(a, AllocationLattice::Known(site_s));
        analysis.value_sites.insert(b, AllocationLattice::Known(site_t));
        assert!(!analysis.must_alias(a, b));
    }

    #[test]
    fn must_alias_rejects_multiple_even_at_same_site() {
        let (_ssa, ids) = ssa_with_allocates(3);
        let (site, a, b) = (ids[0], ids[1], ids[2]);
        let mut analysis = MustAliasAnalysis::empty();
        analysis.value_sites.insert(a, AllocationLattice::Multiple(site));
        analysis.value_sites.insert(b, AllocationLattice::Multiple(site));
        assert!(!analysis.must_alias(a, b));
    }

    #[test]
    fn must_alias_rejects_top_or_external() {
        let (_ssa, ids) = ssa_with_allocates(2);
        let (a, b) = (ids[0], ids[1]);
        let mut analysis = MustAliasAnalysis::empty();
        analysis.value_sites.insert(a, AllocationLattice::NoAllocation);
        analysis.value_sites.insert(b, AllocationLattice::NoAllocation);
        assert!(!analysis.must_alias(a, b));

        analysis.value_sites.insert(a, AllocationLattice::External);
        analysis.value_sites.insert(b, AllocationLattice::External);
        assert!(!analysis.must_alias(a, b));
    }

    #[test]
    fn trusted_site_returns_known_site() {
        let (_ssa, ids) = ssa_with_allocates(2);
        let (site, a) = (ids[0], ids[1]);
        let mut analysis = MustAliasAnalysis::empty();
        analysis.value_sites.insert(a, AllocationLattice::Known(site));
        assert_eq!(analysis.known_site(a), Some(site));
    }

    #[test]
    fn trusted_site_rejects_multiple() {
        let (_ssa, ids) = ssa_with_allocates(2);
        let (site, a) = (ids[0], ids[1]);
        let mut analysis = MustAliasAnalysis::empty();
        analysis.value_sites.insert(a, AllocationLattice::Multiple(site));
        assert_eq!(analysis.known_site(a), None);
    }

    #[test]
    fn trusted_site_rejects_undef_or_top_or_external() {
        let (_ssa, ids) = ssa_with_allocates(1);
        let a = ids[0];
        let mut analysis = MustAliasAnalysis::empty();
        // Absent → Undef.
        assert_eq!(analysis.known_site(a), None);
        analysis.value_sites.insert(a, AllocationLattice::NoAllocation);
        assert_eq!(analysis.known_site(a), None);
        analysis.value_sites.insert(a, AllocationLattice::External);
        assert_eq!(analysis.known_site(a), None);
    }

    // -----------------------------------------------------------------------
    // End-to-end: full dataflow via `MustAliasAnalysis::analyze`
    // -----------------------------------------------------------------------

    /// Run the full pipeline (Steensgaard + must-alias) on the given SSA
    /// source, returning the parsed SSA and the resulting analysis.
    fn analyze_source(src: &str) -> (Ssa, MustAliasAnalysis) {
        let ssa = Ssa::from_str(src).unwrap();
        let aliases = AliasAnalysis::analyze(&ssa);
        let analysis = MustAliasAnalysis::analyze(&ssa, &aliases);
        (ssa, analysis)
    }

    fn collect_allocates_main(ssa: &Ssa) -> Vec<GlobalValueId> {
        let func = ssa.main();
        let mut out = Vec::new();
        for block_id in func.reachable_blocks() {
            for inst_id in func.dfg[block_id].instructions() {
                if matches!(&func.dfg[*inst_id], Instruction::Allocate) {
                    let v = func.dfg.instruction_result::<1>(*inst_id)[0];
                    out.push(GlobalValueId::new(func, v));
                }
            }
        }
        out
    }

    fn collect_loads_main(ssa: &Ssa) -> Vec<GlobalValueId> {
        let func = ssa.main();
        let mut out = Vec::new();
        for block_id in func.reachable_blocks() {
            for inst_id in func.dfg[block_id].instructions() {
                if matches!(&func.dfg[*inst_id], Instruction::Load { .. }) {
                    let v = func.dfg.instruction_result::<1>(*inst_id)[0];
                    out.push(GlobalValueId::new(func, v));
                }
            }
        }
        out
    }

    #[test]
    fn end_to_end_single_allocate_has_trusted_site() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            return
        }
        ";
        let (ssa, analysis) = analyze_source(src);
        let allocs = collect_allocates_main(&ssa);
        assert_eq!(analysis.known_site(allocs[0]), Some(allocs[0]));
    }

    #[test]
    fn end_to_end_distinct_allocates_dont_must_alias() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            return
        }
        ";
        let (ssa, analysis) = analyze_source(src);
        let allocs = collect_allocates_main(&ssa);
        assert!(!analysis.must_alias(allocs[0], allocs[1]));
        assert_eq!(analysis.known_site(allocs[0]), Some(allocs[0]));
        assert_eq!(analysis.known_site(allocs[1]), Some(allocs[1]));
    }

    #[test]
    fn end_to_end_load_after_store_recovers_must_alias() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut &mut Field
            store v0 at v1
            v2 = load v1 -> &mut Field
            return
        }
        ";
        let (ssa, analysis) = analyze_source(src);
        let allocs = collect_allocates_main(&ssa);
        let loads = collect_loads_main(&ssa);
        // After store v0 at v1, the cell at class(v1) holds Known(v0); the
        // load through v1 must-aliases v0.
        assert!(analysis.must_alias(allocs[0], loads[0]));
        assert_eq!(analysis.known_site(loads[0]), Some(allocs[0]));
    }

    #[test]
    fn end_to_end_loop_allocate_is_multiple_not_trusted() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1):
            jmp b1(v0)
          b1(v1: u1):
            v2 = allocate -> &mut Field
            jmpif v1 then: b1(v1), else: b2()
          b2():
            return
        }
        ";
        let (ssa, analysis) = analyze_source(src);
        let allocs = collect_allocates_main(&ssa);
        // The allocate sits in a loop block so its result is `Multiple(s)`,
        // not `Known(s)` — no trusted site, no must-alias even with itself
        // across iterations (here we only have one allocate ID, but the
        // lattice value is what later consumers see).
        assert_eq!(analysis.known_site(allocs[0]), None);
    }

    #[test]
    fn unresolved_call_poisons_deep_pointee_chain() {
        // An indirect (unresolved) call receives `v1: &mut &mut &mut Field`.
        // Before the call, `**v1` is stored to a single-firing allocation
        // `v3`, giving the level-2 pointee class `Known(v3)`. The opaque
        // callee can mutate any cell reachable from `v1` at any depth — in
        // particular it can overwrite `**v1`. So after the call a load of
        // `**v1` (here `v5`) must NOT keep the `Known(v3)` site, otherwise a
        // consumer such as `load_store_forwarding` would forward a stale
        // value. Poisoning only the first pointee level (`*v1`) is unsound.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: function):
            v1 = allocate -> &mut &mut &mut Field
            v2 = load v1 -> &mut &mut Field
            v3 = allocate -> &mut Field
            store v3 at v2
            call v0(v1)
            v4 = load v1 -> &mut &mut Field
            v5 = load v4 -> &mut Field
            return
        }
        ";
        let (ssa, analysis) = analyze_source(src);
        let allocs = collect_allocates_main(&ssa);
        let loads = collect_loads_main(&ssa);
        // allocs[1] = v3 (the deep stored allocation); loads[2] = v5 (the
        // load of `**v1` after the unresolved call).
        assert_eq!(
            analysis.known_site(loads[2]),
            None,
            "deep pointee site must be poisoned across an unresolved call",
        );
        assert!(!analysis.must_alias(allocs[1], loads[2]));
    }

    #[test]
    fn end_to_end_distinct_sites_filter_via_cannot_equal() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            return
        }
        ";
        let (ssa, analysis) = analyze_source(src);
        let allocs = collect_allocates_main(&ssa);
        // Both are `Known(_)` with distinct sites → cannot alias.
        assert!(analysis.cannot_equal(allocs[0], allocs[1]));
    }

    // -----------------------------------------------------------------------
    // AsVector identity propagation
    // -----------------------------------------------------------------------

    #[test]
    fn end_to_end_as_vector_does_not_panic_and_is_sound() {
        // Regression test for the precompute that previously indexed
        // `arguments[1]` unconditionally — `AsVector` has only one
        // argument, so that path panicked. The analysis must now run
        // through cleanly. The result's site is `NoAllocation` because
        // the parameter input has no tracked site (Undef-fallback case).
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [(); 3]):
            v1, v2 = call as_vector(v0) -> (u32, [()])
            return v1, v2
        }
        ";
        let (_ssa, _analysis) = analyze_source(src);
        // No panic; reaching here is the assertion.
    }

    #[test]
    fn as_vector_propagates_known_input_site_through_transfer() {
        // Direct test of the AsVector identity-propagation contract:
        // when the input's `value_sites` is `Known(s)`, the result must
        // carry the same `Known(s)`. Constructed by seeding analysis
        // state and calling `transfer_intrinsic_call` directly, because
        // standard Noir SSA flow rarely produces a `Known` site on an
        // array value — the contract is what's under test, not the
        // upstream flow.
        let (ssa, ids) = ssa_with_allocates(3);
        let aliases = AliasAnalysis::analyze(&ssa);
        let mut analysis = MustAliasAnalysis::empty();

        let (site, input_array, new_vec) = (ids[0], ids[1], ids[2]);
        analysis.value_sites.insert(input_array, AllocationLattice::Known(site));

        let mut changes = InterProceduralChanges::default();
        let len_result = ids[0]; // length result; AsVector ignores its site.
        let _ = analysis.transfer_intrinsic_call(
            &Intrinsic::AsVector,
            &[input_array],
            &[len_result, new_vec],
            &aliases,
            &mut changes,
        );

        assert_eq!(
            analysis.value_sites.get(&new_vec).copied(),
            Some(AllocationLattice::Known(site)),
            "AsVector should propagate the input's Known site to the new_vec result",
        );
        // And the corresponding must-alias query holds.
        assert!(analysis.must_alias(input_array, new_vec));
    }

    #[test]
    fn as_vector_undef_input_leaves_result_untracked() {
        // The companion to the propagation test: `AsVector` is pure identity
        // propagation, so an untracked input (Undef) fabricates no site and
        // the result is left untracked. This is sound — the queries treat a
        // missing/Undef site conservatively (no must-alias, no cannot_equal) —
        // and strictly more precise than topping out to NoAllocation, since
        // the result still resolves to the input's site if it is learned later.
        let (ssa, ids) = ssa_with_allocates(3);
        let aliases = AliasAnalysis::analyze(&ssa);
        let mut analysis = MustAliasAnalysis::empty();

        let (input_array, new_vec) = (ids[1], ids[2]);
        // No seed for input_array → its site is Undef.

        let mut changes = InterProceduralChanges::default();
        let len_result = ids[0];
        let _ = analysis.transfer_intrinsic_call(
            &Intrinsic::AsVector,
            &[input_array],
            &[len_result, new_vec],
            &aliases,
            &mut changes,
        );

        assert_eq!(analysis.value_sites.get(&new_vec).copied(), None);
    }
}
