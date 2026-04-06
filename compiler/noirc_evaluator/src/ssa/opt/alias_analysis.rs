//! Standalone alias analysis for SSA functions.
//!
//! Following LLVM's layered design, this module provides:
//!
//! 1. **Alias queries** — "can two addresses refer to the same memory?"
//!    Based on allocation identity: different `allocate` results → `NoAlias`.
//!
//! 2. **Per-block known values** — "what value is stored at address X at the
//!    start of block B?" Computed via forward dataflow using predecessor
//!    intersection and alias queries for safety.
//!
//! LSF and other passes consume these results instead of maintaining their own
//! ad-hoc alias tracking.
//!
//! A future follow-up can split the known-value propagation into its own module
//! (analogous to LLVM's MemorySSA) when DSE or other passes need it.
//!
//! See also: <https://github.com/noir-lang/noir/issues/12005>

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        function::Function,
        instruction::{Instruction, TerminatorInstruction},
        value::ValueId,
    },
    opt::{LoopOrder, Loops},
};

/// Alias analysis results for a single SSA function.
///
/// Provides alias queries and per-block known address→value maps.
pub(crate) struct AliasAnalysis {
    /// Addresses originating from `allocate` instructions.
    /// Two different allocate results never alias each other.
    allocations: HashSet<ValueId>,

    /// Addresses involved in loop-carried alias patterns.
    /// Stores to these addresses conservatively invalidate all known values.
    loop_aliases: HashSet<ValueId>,

    /// Per-block known address→value at the ENTRY of each block.
    /// Computed via forward dataflow using predecessor intersection.
    known_at_entry: HashMap<BasicBlockId, HashMap<ValueId, ValueId>>,
}

impl AliasAnalysis {
    /// Build the full analysis for a function: alias facts, loop aliases,
    /// and per-block known values.
    pub(crate) fn new(function: &Function) -> Self {
        use crate::ssa::ir::{cfg::ControlFlowGraph, post_order::PostOrder};

        let cfg = ControlFlowGraph::with_function(function);
        let blocks = PostOrder::with_cfg(&cfg).into_vec_reverse(); // RPO order
        let loops = Loops::find_all(function, LoopOrder::OutsideIn);

        let allocations = collect_allocations(function, &blocks);
        let loop_aliases = analyze_loop_aliases(function, &loops);
        let known_at_entry = compute_known_values_at_entry(
            function,
            &cfg,
            &blocks,
            &allocations,
            &loop_aliases,
            &loops,
        );

        Self { allocations, loop_aliases, known_at_entry }
    }

    /// Returns true if the two addresses might refer to the same memory.
    /// Conservative: returns true (MayAlias) when uncertain.
    pub(crate) fn may_alias(&self, addr_a: ValueId, addr_b: ValueId) -> bool {
        if addr_a == addr_b {
            return true;
        }
        // Two different allocations provably don't alias.
        let a_is_alloc = self.allocations.contains(&addr_a);
        let b_is_alloc = self.allocations.contains(&addr_b);
        if a_is_alloc && b_is_alloc {
            return false;
        }
        true // Unknown derivation → conservative MayAlias
    }

    /// Returns true if this address originates from an `allocate` instruction.
    pub(crate) fn is_allocation(&self, addr: ValueId) -> bool {
        self.allocations.contains(&addr)
    }

    /// Returns true if this address is involved in a loop-carried alias pattern.
    pub(crate) fn is_loop_aliased(&self, addr: ValueId) -> bool {
        self.loop_aliases.contains(&addr)
    }

    /// Known address→value pairs at the entry of a block (intersection of
    /// predecessor exit states).
    pub(crate) fn get_known_at_entry(
        &self,
        block: BasicBlockId,
    ) -> Option<&HashMap<ValueId, ValueId>> {
        self.known_at_entry.get(&block)
    }

    /// Returns the set of reference addresses potentially modified by a call,
    /// or `None` if the call could modify any address (e.g. nested references
    /// or containers holding references are passed).
    pub(crate) fn addresses_modified_by_call(
        &self,
        instruction: &Instruction,
        dfg: &DataFlowGraph,
        resolver: impl Fn(ValueId) -> ValueId,
    ) -> Option<Vec<ValueId>> {
        compute_addresses_modified_by_call(instruction, dfg, resolver)
    }
}

/// Returns the set of reference addresses potentially modified by a call,
/// or `None` if the call could modify any address.
///
/// A simple reference argument (`&mut T` where `T` has no references) means
/// only that specific address may be modified. A nested reference (`&mut &mut T`)
/// or a container holding references means the callee can reach additional
/// addresses, so we return `None` (could modify anything).
fn compute_addresses_modified_by_call(
    instruction: &Instruction,
    dfg: &DataFlowGraph,
    resolver: impl Fn(ValueId) -> ValueId,
) -> Option<Vec<ValueId>> {
    let mut modified = Vec::new();
    let mut clear_all = false;

    instruction.for_each_value(|value| {
        if clear_all {
            return;
        }
        let value = resolver(value);
        if dfg.value_is_reference(value) {
            modified.push(value);
            let typ = dfg.type_of_value(value);
            if typ.reference_element_type().is_some_and(|e| e.contains_reference()) {
                clear_all = true;
            }
        } else if dfg.type_of_value(value).contains_reference() {
            clear_all = true;
        }
    });

    if clear_all { None } else { Some(modified) }
}

/// Collect all addresses from `allocate` instructions across reachable blocks.
fn collect_allocations(function: &Function, blocks: &[BasicBlockId]) -> HashSet<ValueId> {
    let mut allocations = HashSet::default();
    for &block_id in blocks {
        for instruction_id in function.dfg[block_id].instructions() {
            if matches!(function.dfg[*instruction_id], Instruction::Allocate) {
                let result = function.dfg.instruction_results(*instruction_id)[0];
                allocations.insert(result);
            }
        }
    }
    allocations
}

/// Identify addresses involved in loop-carried alias patterns.
///
/// A loop-carried alias occurs in two forms:
///
/// 1. **Store of a reference inside a loop** (`store ref_value at ref_address`):
///    The stored reference can alias a local variable that gets re-initialized
///    in a later iteration, making the "clear-on-unknown-store" heuristic
///    insufficient.
///
/// 2. **Reference-typed block parameters on loop headers**: If `mem2reg_simple`
///    has already promoted a `store ref at ref` to a block parameter, the
///    aliasing is implicit. A reference-typed loop header parameter and the
///    corresponding jmp arguments from within the loop body create the same
///    cross-iteration aliasing.
fn analyze_loop_aliases(function: &Function, loops: &Loops) -> HashSet<ValueId> {
    let mut aliases: HashSet<ValueId> = HashSet::default();
    for loop_info in &loops.yet_to_unroll {
        // Form 1: store of a reference inside a loop block.
        for block_id in &loop_info.blocks {
            let block = &function.dfg[*block_id];
            for instruction_id in block.instructions() {
                if let Instruction::Store { address, value } = &function.dfg[*instruction_id]
                    && function.dfg.value_is_reference(*value)
                {
                    aliases.insert(*address);
                    aliases.insert(*value);
                }
            }
        }

        // Form 2: reference-typed block parameters on the loop header.
        let header = loop_info.header;
        let header_params = function.dfg[header].parameters();
        let ref_param_indices: Vec<usize> = header_params
            .iter()
            .enumerate()
            .filter(|(_, param)| function.dfg.value_is_reference(**param))
            .map(|(idx, param)| {
                aliases.insert(*param);
                idx
            })
            .collect();

        if !ref_param_indices.is_empty() {
            for block_id in &loop_info.blocks {
                let block = &function.dfg[*block_id];
                match block.terminator() {
                    Some(TerminatorInstruction::Jmp { destination, arguments, .. })
                        if *destination == header =>
                    {
                        for &idx in &ref_param_indices {
                            if let Some(arg) = arguments.get(idx) {
                                aliases.insert(*arg);
                            }
                        }
                    }
                    Some(TerminatorInstruction::JmpIf {
                        then_destination,
                        then_arguments,
                        else_destination,
                        else_arguments,
                        ..
                    }) => {
                        if *then_destination == header {
                            for &idx in &ref_param_indices {
                                if let Some(arg) = then_arguments.get(idx) {
                                    aliases.insert(*arg);
                                }
                            }
                        }
                        if *else_destination == header {
                            for &idx in &ref_param_indices {
                                if let Some(arg) = else_arguments.get(idx) {
                                    aliases.insert(*arg);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    aliases
}

/// Compute per-block known address→value maps using forward dataflow analysis.
///
/// Uses **predecessor intersection** as the meet operation: a block's initial
/// known values are the intersection of all predecessors' exit states. An address
/// is only forwarded if ALL predecessors agree on its value.
///
/// This is sound for both acyclic control flow (diamonds/merge points) and loops
/// (back-edge predecessors are skipped since they haven't been visited in RPO).
///
/// Within each block, stores update known values using conservative alias heuristics:
/// - Loop-aliased addresses → clear all known values
/// - Unknown addresses (not an allocation, not already tracked) → clear all
/// - Known/allocation addresses → update only that entry
///
/// Calls invalidate known values for reference arguments.
fn compute_known_values_at_entry(
    function: &Function,
    cfg: &crate::ssa::ir::cfg::ControlFlowGraph,
    blocks: &[BasicBlockId],
    allocations: &HashSet<ValueId>,
    loop_aliases: &HashSet<ValueId>,
    loops: &Loops,
) -> HashMap<BasicBlockId, HashMap<ValueId, ValueId>> {
    // Collect addresses stored within each loop body, keyed by loop header.
    // These must be invalidated at loop headers to prevent the pre-loop value
    // from being forwarded through the loop exit (the loop body overwrites it).
    let loop_stored_addresses = collect_loop_stored_addresses(function, allocations, loops);

    // Track which blocks have been visited (for skipping back-edge predecessors in loops).
    let mut visited: HashSet<BasicBlockId> = HashSet::default();
    let mut exit_states: HashMap<BasicBlockId, HashMap<ValueId, ValueId>> = HashMap::default();
    let mut entry_states: HashMap<BasicBlockId, HashMap<ValueId, ValueId>> = HashMap::default();

    for &block in blocks {
        // Compute initial known values from predecessors.
        // Only consider predecessors already visited in RPO (skip back-edges).
        let mut known = {
            let mut forward_preds =
                cfg.predecessors(block).filter(|pred| visited.contains(pred)).peekable();

            if forward_preds.peek().is_none() {
                // Entry block or unreachable — start empty
                HashMap::default()
            } else {
                // Intersect all forward predecessors' exit states:
                // keep only addresses where ALL predecessors agree on the value.
                let mut first = true;
                let mut result: HashMap<ValueId, ValueId> = HashMap::default();

                for pred in forward_preds {
                    if let Some(pred_exit) = exit_states.get(&pred) {
                        if first {
                            result = pred_exit.clone();
                            first = false;
                        } else {
                            // Keep only entries present in BOTH with the SAME value
                            result.retain(|addr, value| pred_exit.get(addr) == Some(value));
                        }
                    } else if !first {
                        // Predecessor has no exit state → no known values from it
                        result.clear();
                    }
                    // If first is still true and pred has no exit state,
                    // result stays empty (correct).
                }
                result
            }
        };

        // If this block is a loop header, invalidate addresses stored in the loop body.
        // Without this, the pre-loop value would be forwarded through the loop exit,
        // ignoring writes from the loop body that execute before the exit is reached.
        if let Some(stored_addrs) = loop_stored_addresses.get(&block) {
            for addr in stored_addrs {
                known.remove(addr);
            }
        }

        visited.insert(block);
        entry_states.insert(block, known.clone());

        // Process instructions within the block to compute exit state.
        for instruction_id in function.dfg[block].instructions() {
            let instruction = &function.dfg[*instruction_id];
            match instruction {
                Instruction::Store { address, value } => {
                    if loop_aliases.contains(address) {
                        known.clear();
                    } else if !known.contains_key(address) && !allocations.contains(address) {
                        // Unknown address may alias tracked addresses → clear
                        known.clear();
                    } else {
                        // Known/allocation address. Invalidate entries that may alias it.
                        // Two distinct allocations don't alias (NoAlias).
                        let addr_is_alloc = allocations.contains(address);
                        known.retain(|k, _| {
                            k == address || (addr_is_alloc && allocations.contains(k))
                        });
                    }
                    known.insert(*address, *value);
                }
                Instruction::Call { .. } => {
                    match compute_addresses_modified_by_call(instruction, &function.dfg, |v| v) {
                        Some(addrs) => {
                            for addr in &addrs {
                                known.remove(addr);
                            }
                        }
                        None => known.clear(),
                    }
                }
                _ => {}
            }
        }

        exit_states.insert(block, known);
    }

    entry_states
}

/// For each loop header, collect the set of addresses stored to within the loop body.
fn collect_loop_stored_addresses(
    function: &Function,
    allocations: &HashSet<ValueId>,
    loops: &Loops,
) -> HashMap<BasicBlockId, HashSet<ValueId>> {
    let mut result: HashMap<BasicBlockId, HashSet<ValueId>> = HashMap::default();

    for loop_info in &loops.yet_to_unroll {
        let mut stored_addresses = HashSet::default();
        for block_id in &loop_info.blocks {
            for instruction_id in function.dfg[*block_id].instructions() {
                let instruction = &function.dfg[*instruction_id];
                match instruction {
                    Instruction::Store { address, .. } => {
                        stored_addresses.insert(*address);
                        // If the store address is NOT a known allocation, it could
                        // be a loaded reference that aliases any allocation.
                        // Conservatively mark all allocations as potentially modified.
                        if !allocations.contains(address) {
                            stored_addresses.extend(allocations.iter().copied());
                        }
                    }
                    // A call with reference arguments may store through those
                    // references. Treat them as potentially modified addresses.
                    // For containers holding references, we can't know which
                    // addresses they hold, so we add all known allocations.
                    Instruction::Call { .. } => {
                        match compute_addresses_modified_by_call(instruction, &function.dfg, |v| v)
                        {
                            Some(addrs) => {
                                stored_addresses.extend(addrs);
                            }
                            None => {
                                stored_addresses.extend(allocations.iter().copied());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        if !stored_addresses.is_empty() {
            result.insert(loop_info.header, stored_addresses);
        }
    }

    result
}
