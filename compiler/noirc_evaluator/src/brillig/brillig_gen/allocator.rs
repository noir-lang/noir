//! The pluggable register-allocation seam for Brillig codegen.
//!
//! Brillig has no hardware register file; "registers" are the low slots of a function's stack
//! frame. Allocation decides which SSA value lives in which slot and what to do when a frame runs
//! out of slots (spill to the heap-backed spill region). Historically those decisions were made
//! inline by [`BrilligBlock`](super::brillig_block::BrilligBlock), interleaved with opcode
//! emission. This module hosts the seam that lifts those decisions behind an interface so codegen
//! becomes a pure consumer.
//!
//! **Information flows one way: allocator → shadow.** The allocator owns the authoritative state —
//! which value is in which register, what is spilled, the free list. The driver keeps only a
//! *register shadow* (`value → MemoryAddress`), which it seeds from [`Allocator::begin_block`] and
//! updates by applying the [`Action`]s the allocator returns (each carries its `value`) plus the
//! addresses returned by `define_variable`/`use_variable`. Values enter the shadow at their
//! definition/reload and leave it on a [`Action::Spill`] or, when they die, a [`Action::Prune`], so
//! the shadow's registers are exactly the occupied ones at every point. The driver never tells the
//! allocator what is resident. Because the shadow is a pure function of the allocator's output,
//! swapping the greedy allocator for the linear-scan one requires no change to the driver — proving
//! that is the point of this phase.
//!
//! See `design/register_allocation.md` (Phase 0.5) for the full plan.

use std::cell::RefCell;
use std::rc::Rc;

use acvm::acir::brillig::MemoryAddress;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::brillig_block_variables::{BlockVariables, compute_array_length};
use super::coalescing::CoalescingMap;
use super::spill_manager::SpillManager;
use super::variable_liveness::VariableLiveness;
use crate::brillig::brillig_ir::brillig_variable::{
    BrilligArray, BrilligVariable, BrilligVector, SingleAddrVariable, get_bit_size_from_ssa_type,
};
use crate::brillig::brillig_ir::registers::RegisterAllocator;
use crate::ssa::ir::basic_block::BasicBlockId;
use crate::ssa::ir::dfg::DataFlowGraph;
use crate::ssa::ir::instruction::InstructionId;
use crate::ssa::ir::types::Type;
use crate::ssa::ir::value::ValueId;

/// A slot in the heap-backed spill region.
///
/// Addressed by offset from the per-frame spill base pointer (`sp[1]`); offset `0` is the base
/// pointer itself. The allocator owns slot assignment and hands codegen a resolved `SpillSlot`
/// inside an [`Action`] — codegen never computes or tracks spill offsets itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SpillSlot(pub(crate) usize);

impl SpillSlot {
    /// The offset of this slot relative to the per-frame spill base pointer.
    pub(crate) fn offset(self) -> usize {
        self.0
    }
}

/// A single unit of register-map traffic the allocator asks the driver to apply.
///
/// Every action carries the `value` it concerns so the driver updates its register map
/// mechanically as it applies the action: `Reload`/`Move` put the value at the destination
/// register; `Spill` and `Prune` remove it. Most actions also emit an opcode; `Prune` is
/// bookkeeping-only. Register-facing endpoints are [`MemoryAddress`]; the spill region is addressed
/// through [`SpillSlot`], which the allocator resolves from its own bookkeeping.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Action {
    /// Store a register-resident value into its spill slot (the value leaves the map).
    Spill { value: ValueId, from: MemoryAddress, to: SpillSlot },
    /// Load a spilled value from its slot into a register (the value enters the map at `into`).
    Reload { value: ValueId, from: SpillSlot, into: MemoryAddress },
    /// Register-to-register move (the value moves to `to` in the map). Produced by edge
    /// resolution, which the greedy allocator does not yet drive; see `resolve_edge` in the design
    /// doc (Phase 1).
    #[allow(dead_code)]
    Move { value: ValueId, from: MemoryAddress, to: MemoryAddress },
    /// Drop a now-dead value from the map, freeing `register`. Emits no opcode: it only keeps the
    /// driver's map a faithful mirror of residency, so the map's registers are exactly the occupied
    /// ones. `register` is the value's current register; the driver asserts the map agreed, which
    /// checks the allocator and driver never drifted out of sync.
    Prune { value: ValueId, register: MemoryAddress },
}

/// The register-allocation seam: codegen asks the allocator where each value lives and what memory
/// traffic to emit, and the allocator makes those decisions and maintains the register/spill state.
///
/// The driver ([`BrilligBlock`](super::brillig_block::BrilligBlock)) owns opcode emission and the
/// one-way register shadow (see the module docs); the allocator owns the register pool, the spill
/// slots, and the SSA-value → register cache. Methods that move data return the [`Action`]s the
/// driver must emit; methods that only reserve or free registers return nothing to emit.
///
/// Methods are keyed on SSA ids (values, instructions, blocks); the allocator resolves them against
/// state it owns. `define_variable` still takes the `dfg` to read the value's type — the register
/// count is always one, but the driver needs the typed [`BrilligVariable`] wrapper. A linear-scan
/// implementation would precompute the type shapes and drop even that.
pub(crate) trait Allocator {
    /// Enter a block: reset per-block eviction state, pre-allocate the register pool with the
    /// block's register-resident live-ins, and define the block parameters this block is
    /// responsible for (its own, plus those of the blocks it immediately dominates). Returns the
    /// register-resident values (live-ins and register-homed params) so the driver can seed its
    /// register map. Values that live across a block boundary are handled by the allocator's own
    /// cross-block policy (for the greedy allocator, a permanent spill), so they are not returned.
    fn begin_block(
        &mut self,
        block: BasicBlockId,
        dfg: &DataFlowGraph,
    ) -> (Vec<(ValueId, MemoryAddress)>, Vec<Action>);

    /// Bring a value into existence at its definition point (a constant, an instruction result, or
    /// a parameter). Reserves its register — spilling LRU victims first if the frame is full — and
    /// returns the allocation plus the spills that freed the room. The driver then writes the value
    /// into the returned register.
    fn define_variable(
        &mut self,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> (BrilligVariable, Vec<Action>);

    /// Make an already-defined value register-resident and return its allocation, plus the actions
    /// to get it there: a [`Action::Reload`] if it was spilled (and any [`Action::Spill`]s to free
    /// a slot for it), or no actions if it is already resident.
    fn use_variable(&mut self, value_id: ValueId) -> (BrilligVariable, Vec<Action>);

    /// Ensure `n` registers are free for codegen to allocate scratch temporaries via its RAII path,
    /// returning the spills that freed the room. The contract is "N slots are free", not "here are
    /// the registers" — codegen owns the temporaries.
    ///
    /// This is the scratch half of the design's `before_instruction`; making the instruction's
    /// operands resident up front (the other half) is still done lazily by the driver via
    /// `use_variable`, so this takes an explicit count rather than an instruction id for now.
    fn reserve_scratch(&mut self, scratch: usize) -> Vec<Action>;

    /// Retire the values whose last use is `inst` now that it has been lowered: free their
    /// registers (returning them to the pool for reuse later in the block) and drop any bookkeeping.
    /// Returns a [`Action::Prune`] for each retired value that held a register, so the driver drops
    /// it from its map; no opcodes are emitted. Values the allocator does not track — globals and
    /// hoisted constants, which live in the driver's globals map and are reserved for the whole
    /// program — are skipped.
    fn after_instruction(&mut self, inst: InstructionId) -> Vec<Action>;

    /// Prepare for `block`'s terminator: settle the location of every value that lives across an
    /// outgoing edge, before the terminator's operands are read. Returns the memory traffic to emit
    /// (for the greedy allocator, permanent spills of the cross-block live-ins of each successor).
    fn before_terminator(&mut self, block: BasicBlockId, dfg: &DataFlowGraph) -> Vec<Action>;

    /// Where each parameter of `succ` lives, so the driver can pass a jmp's arguments to them —
    /// a register-to-register move for a register-homed param, or a store to its slot for a
    /// spill-homed one. This is per-edge because a value's home can differ per predecessor under a
    /// splitting allocator; the greedy allocator's homes are per-value, so it ignores `pred`.
    fn resolve_edge(
        &self,
        pred: BasicBlockId,
        succ: BasicBlockId,
        dfg: &DataFlowGraph,
    ) -> Vec<ParamHome>;

    /// Whether spilling is enabled for this function (there is a spill manager).
    fn spill_enabled(&self) -> bool;

    /// The high-water mark of spill slots used, for sizing the spill prologue (0 if none).
    fn max_spill_offset(&self) -> usize;
}

/// Where a block parameter lives, as reported by [`Allocator::resolve_edge`], telling the driver
/// how to pass a jmp argument to it.
#[derive(Clone, Copy, Debug)]
pub(crate) enum ParamHome {
    /// The parameter is register-homed: move the argument into this register.
    Register(MemoryAddress),
    /// The parameter is spill-homed (permanently spilled): store the argument into this slot.
    Slot(SpillSlot),
}

/// The concrete "greedy + LRU spilling" allocator.
///
/// It owns the authoritative allocation state: the SSA-value → register cache, its own residency
/// model (`resident`), the spill manager, and the coalescing map (a construction-time input, not a
/// rival pass). The pluggable seam that lets a different strategy (e.g. linear scan) take its place
/// is the [`Allocator`] trait above.
pub(crate) struct GreedyAllocator<R: RegisterAllocator> {
    /// A shared handle to the register pool (the same `Rc` the [`BrilligContext`] holds, so codegen
    /// can still allocate its RAII scratch temporaries from it). The allocator owns the allocation
    /// decisions: it allocates SSA-value registers here, frees them on retirement, and reseeds it in
    /// place at each block entry.
    ///
    /// [`BrilligContext`]: crate::brillig::brillig_ir::BrilligContext
    pool: Rc<RefCell<R>>,
    /// The allocator's authoritative residency model: which values currently hold a register. The
    /// spill manager queries this; the driver's shadow is a downstream projection of it.
    resident: BlockVariables,
    /// Map from SSA values to their allocation. Since values are defined only once in SSA form,
    /// we insert them here when we allocate them at their definition. This accumulates historical
    /// allocations (not just currently-live ones), so a block can look up the allocation of its
    /// live-in values even if a previously-visited block deemed them dead.
    ssa_value_allocations: HashMap<ValueId, BrilligVariable>,
    /// Spilling of register values to the heap spill region when register pressure exceeds the
    /// stack frame limit. Persists across blocks. Present only when the function may need spilling.
    spill_manager: Option<SpillManager>,
    /// Coalescing map for jmp argument → block parameter register sharing.
    coalescing: CoalescingMap,
    /// Per-variable liveness for the function. Immutable during codegen; the allocator reads it to
    /// know a block's live-ins and the block parameters it defines (`begin_block`) and the
    /// successors' cross-block live-ins to spill (`before_terminator`).
    liveness: VariableLiveness,
    /// For each instruction, the values whose last use it is — i.e. those that die once the
    /// instruction has been lowered. The allocator retires them in `after_instruction`, freeing
    /// their registers for reuse later in the block. Keyed by the globally-unique `InstructionId`
    /// (derived from `liveness`), so a single function-wide map suffices.
    last_uses: HashMap<InstructionId, HashSet<ValueId>>,
}

impl<R: RegisterAllocator> GreedyAllocator<R> {
    /// Build the greedy allocator with a shared handle to the register pool, plus the spill manager,
    /// coalescing map, liveness, and per-instruction last-use sets decided by
    /// [`FunctionContext::new`](super::brillig_fn::FunctionContext::new).
    pub(crate) fn new(
        pool: Rc<RefCell<R>>,
        spill_manager: Option<SpillManager>,
        coalescing: CoalescingMap,
        liveness: VariableLiveness,
        last_uses: HashMap<InstructionId, HashSet<ValueId>>,
    ) -> Self {
        Self {
            pool,
            resident: BlockVariables::default(),
            ssa_value_allocations: HashMap::default(),
            spill_manager,
            coalescing,
            liveness,
            last_uses,
        }
    }

    /// The final register allocations, consumed when handing the global allocations to the artifact.
    pub(crate) fn into_allocations(self) -> HashMap<ValueId, BrilligVariable> {
        self.ssa_value_allocations
    }

    /// The coalescing partner of `value_id`, if any. Exposed for tests that assert how the
    /// construction-time coalescing map was built.
    #[cfg(test)]
    pub(crate) fn get_coalesced(&self, value_id: &ValueId) -> Option<ValueId> {
        self.coalescing.get_coalesced(value_id)
    }

    /// The liveness the allocator was built with. Exposed for tests.
    #[cfg(test)]
    pub(crate) fn liveness(&self) -> &VariableLiveness {
        &self.liveness
    }

    /// Retire a single value, for tests that force a specific deallocation order. In production
    /// retirement is driven by `after_instruction` from the last-use sets.
    #[cfg(test)]
    pub(crate) fn retire(&mut self, value_id: &ValueId) {
        self.retire_value(value_id);
    }
}

impl<R: RegisterAllocator> Allocator for GreedyAllocator<R> {
    fn begin_block(
        &mut self,
        block: BasicBlockId,
        dfg: &DataFlowGraph,
    ) -> (Vec<(ValueId, MemoryAddress)>, Vec<Action>) {
        // The block's register-resident live-ins are the live-in values the allocator tracks
        // (globals/hoisted constants are never in `ssa_value_allocations`) minus any that are
        // permanently spilled — those are reloaded on demand, not pre-allocated.
        let mut live_in: HashSet<ValueId> = self
            .liveness
            .get_live_in(&block)
            .iter()
            .copied()
            .filter(|value_id| self.ssa_value_allocations.contains_key(value_id))
            .collect();
        if let Some(sm) = self.spill_manager.as_mut() {
            sm.begin_block(&mut live_in);
        }
        self.resident = BlockVariables::new(live_in.clone());

        // Reseed the pool in place with the live-in registers pre-allocated: a fresh free list
        // where these are already taken (they may be freed and reused if they die in this block,
        // then become pre-allocated again in a later block, depending on processing order).
        // Mutating through the shared `RefCell` (rather than swapping the `Rc`) keeps the handle
        // the `BrilligContext` holds pointing at the same pool. This is sound because nothing
        // outlives a block boundary holding an old register: cross-block allocations are detached,
        // and within-block scratch temporaries are dropped before the next block begins. The reset
        // bumps the pool's generation, so any register that *did* survive panics on drop rather
        // than corrupting the reseeded free list.
        let registers = live_in
            .iter()
            .map(|value_id| self.ssa_value_allocations[value_id].extract_register())
            .collect();
        self.pool.borrow_mut().reset_to_preallocated(registers);

        // Define the block parameters this block is responsible for: its own, plus those of the
        // blocks it immediately dominates (so a predecessor's jmp can write to them, and the
        // dominated block can read them, from a home reserved before either runs). A successor
        // param does not hold valid data until a jmp writes it, so it is eagerly given a permanent
        // spill home; the block's own params already hold data from the predecessor. Defining a
        // param may spill under pressure, so collect the resulting stores for the driver to emit.
        let mut actions = Vec::new();
        let own_params: HashSet<ValueId> = dfg[block].parameters().iter().copied().collect();
        for param_id in self.liveness.defined_block_params(&block) {
            let (_, param_actions) = self.define_variable(param_id, dfg);
            actions.extend(param_actions);
            if !own_params.contains(&param_id) && self.spill_manager.is_some() {
                actions.extend(self.spill_value(param_id, true, false));
            }
        }

        // Report the register-resident values (live-ins and register-homed params) so the driver
        // can seed its register map.
        let resident: Vec<ValueId> = self.resident.iter().copied().collect();
        let seed = resident
            .into_iter()
            .map(|value_id| (value_id, self.ssa_value_allocations[&value_id].extract_register()))
            .collect();
        (seed, actions)
    }

    fn define_variable(
        &mut self,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> (BrilligVariable, Vec<Action>) {
        // Making room here mirrors the historical `ensure_register_capacity(1)`. When coalescing is
        // active spilling is disabled (they are mutually exclusive), so this is a no-op on the
        // coalesced path below.
        let actions = self.make_room(1);

        // Check the coalescing map — reuse the block parameter's register if coalesced.
        if let Some(param) = self.coalescing.get_coalesced(&value_id) {
            let variable = *self
                .ssa_value_allocations
                .get(&param)
                .expect("ICE: Coalesced parameter not yet allocated");
            // The param must be available at this point: it must have been declared earlier, and
            // should not have been removed as dead, nor spilled. Otherwise the register could have
            // been allocated to something else in the meantime.
            assert!(
                self.resident.is_allocated(&param),
                "ICE: Coalesced parameter not currently available"
            );
            self.ssa_value_allocations.insert(value_id, variable);
            self.resident.add_available(value_id);
            return (variable, actions);
        }

        // Allocate the value's register from the pool. Registers are managed manually (freed at
        // retirement), not via RAII: a register allocated in one block may be freed in another.
        let variable = self.allocate_typed(value_id, dfg);

        if self.ssa_value_allocations.insert(value_id, variable).is_some() {
            unreachable!("ICE: ValueId {value_id:?} was already in cache");
        }
        self.resident.add_available(value_id);

        if let Some(sm) = self.spill_manager.as_mut() {
            sm.touch(value_id, &self.resident);
        }

        (variable, actions)
    }

    fn use_variable(&mut self, value_id: ValueId) -> (BrilligVariable, Vec<Action>) {
        if self.is_spilled(&value_id) {
            return self.reload_spilled_value(value_id);
        }

        // Already resident: fetch the cached allocation and mark it most-recently-used.
        assert!(
            self.resident.is_allocated(&value_id),
            "ICE: ValueId {value_id:?} is not available"
        );
        let variable = *self
            .ssa_value_allocations
            .get(&value_id)
            .unwrap_or_else(|| panic!("ICE: Value not found in cache {value_id}"));
        if let Some(sm) = self.spill_manager.as_mut() {
            sm.touch(value_id, &self.resident);
        }
        (variable, Vec::new())
    }

    fn reserve_scratch(&mut self, scratch: usize) -> Vec<Action> {
        self.make_room(scratch)
    }

    fn before_terminator(&mut self, block: BasicBlockId, dfg: &DataFlowGraph) -> Vec<Action> {
        if self.spill_manager.is_none() {
            return Vec::new();
        }
        // Permanently spill the non-param values that are live into a successor, before the
        // terminator's operands are read (the arg conversion / condition reload may otherwise
        // overwrite a register still holding a value we must store). Successor params are skipped:
        // they are given permanent homes eagerly in `begin_block` and written by the jmp itself.
        // This is greedy's blanket cross-block policy — spill everything crossing an edge and
        // reload on demand — which a global-assignment allocator would replace with per-edge
        // resolution.
        let mut to_spill = Vec::new();
        for succ in self.liveness.cfg().successors(block) {
            let params: HashSet<ValueId> = dfg[succ].parameters().iter().copied().collect();
            for value_id in self.liveness.get_live_in(&succ) {
                if !params.contains(value_id)
                    && self.ssa_value_allocations.contains_key(value_id)
                    && !to_spill.contains(value_id)
                {
                    to_spill.push(*value_id);
                }
            }
        }
        let mut actions = Vec::new();
        for value_id in to_spill {
            actions.extend(self.spill_value(value_id, true, true));
        }
        actions
    }

    fn resolve_edge(
        &self,
        _pred: BasicBlockId,
        succ: BasicBlockId,
        dfg: &DataFlowGraph,
    ) -> Vec<ParamHome> {
        dfg[succ]
            .parameters()
            .iter()
            .map(|param| match self.permanent_spill_slot(param) {
                Some(slot) => ParamHome::Slot(slot),
                None => ParamHome::Register(self.ssa_value_allocations[param].extract_register()),
            })
            .collect()
    }

    fn after_instruction(&mut self, inst: InstructionId) -> Vec<Action> {
        let Some(dead) = self.last_uses.get(&inst) else {
            return Vec::new();
        };
        // Retire the values the allocator tracks; globals/hoisted constants live in the driver's
        // globals map (never in `ssa_value_allocations`) and must not be freed.
        let dead: Vec<ValueId> = dead
            .iter()
            .copied()
            .filter(|value_id| self.ssa_value_allocations.contains_key(value_id))
            .collect();
        let mut actions = Vec::new();
        for value_id in dead {
            // A register-resident value is exactly one the driver has in its map (spilled values
            // left the map on their `Spill`). Retiring it frees the register, so tell the driver to
            // drop the entry. Read the register before retiring, since retirement frees it.
            if self.resident.is_allocated(&value_id) {
                let register = self.ssa_value_allocations[&value_id].extract_register();
                actions.push(Action::Prune { value: value_id, register });
            }
            self.retire_value(&value_id);
        }
        actions
    }

    fn spill_enabled(&self) -> bool {
        self.spill_manager.is_some()
    }

    fn max_spill_offset(&self) -> usize {
        self.spill_manager.as_ref().map_or(0, |sm| sm.max_spill_offset())
    }
}

impl<R: RegisterAllocator> GreedyAllocator<R> {
    /// Allocate a register from the pool for `value_id` and wrap it in the typed [`BrilligVariable`]
    /// its SSA type calls for. Every variant occupies exactly one register (the single-address
    /// value, or an array/vector pointer); the size / bit-size are static metadata the driver needs
    /// to emit typed opcodes, which is the only reason the type is consulted here.
    /// Spill a value to a slot, freeing its register. `permanent` slots survive the whole function
    /// (used for cross-block values); `emit_store = false` skips the store for block parameters
    /// whose register does not yet hold the final value (the jmp writes the slot later). Returns the
    /// store to emit, if any. Internal: driven by `before_terminator` and the `begin_block`
    /// eager-spill of successor params.
    fn spill_value(&mut self, value_id: ValueId, permanent: bool, emit_store: bool) -> Vec<Action> {
        // For a permanent spill, try to promote an existing record first.
        if permanent {
            // A reloaded transient value holds a register that must be freed when promoted to a
            // permanent spill. A value not currently in a register must not have its register freed
            // here — it may still be live (e.g. a value live into more than one successor when
            // `before_terminator` visits them). Capture this before promoting.
            let was_transient_reloaded = self.is_transient_reloaded(&value_id);
            let promoted = self
                .spill_manager
                .as_mut()
                .expect("ICE: spill_value called without spill manager")
                .ensure_permanent_spill(&value_id);
            if promoted {
                if was_transient_reloaded {
                    self.deallocate(&value_id);
                }
                return Vec::new();
            }
        }

        if self.is_spilled(&value_id) {
            return Vec::new();
        }

        let var = *self.ssa_value_allocations.get(&value_id).unwrap();
        let prior_offset = self.spill_manager.as_ref().unwrap().get_spill_offset(&value_id);
        let offset = match prior_offset {
            Some(offset) => offset,
            None => self.spill_manager.as_mut().unwrap().allocate_spill_offset(),
        };
        if permanent {
            self.spill_manager.as_mut().unwrap().record_permanent_spill(value_id, offset, var);
        } else {
            // `value_id` is still in a register here (we free it below), so record_spill's
            // double-spill assert sees a legitimate spill.
            self.spill_manager.as_mut().unwrap().record_spill(
                value_id,
                offset,
                var,
                &self.resident,
            );
        }

        // Only store when we've just allocated the slot. If the value already had a slot, the slot
        // still holds the correct value (SSA values are immutable) so the store would be redundant.
        let actions = if emit_store && prior_offset.is_none() {
            vec![Action::Spill {
                value: value_id,
                from: var.extract_register(),
                to: SpillSlot(offset),
            }]
        } else {
            Vec::new()
        };

        self.deallocate(&value_id);
        actions
    }

    /// The permanent spill slot of a value, if it has one. Used by `resolve_edge` to route a jmp
    /// argument into its destination parameter's slot.
    fn permanent_spill_slot(&self, value_id: &ValueId) -> Option<SpillSlot> {
        self.spill_manager
            .as_ref()
            .and_then(|sm| sm.get_permanent_spill_offset(value_id))
            .map(SpillSlot)
    }

    fn allocate_typed(&self, value_id: ValueId, dfg: &DataFlowGraph) -> BrilligVariable {
        let typ = dfg.type_of_value(value_id);
        let register = self.pool.borrow_mut().allocate_register();
        match typ.as_ref() {
            Type::Numeric(_) | Type::Reference(..) | Type::Function => BrilligVariable::SingleAddr(
                SingleAddrVariable::new(register, get_bit_size_from_ssa_type(&typ)),
            ),
            Type::Array(item_typ, elem_count) => BrilligVariable::BrilligArray(BrilligArray {
                pointer: register,
                size: compute_array_length(item_typ, *elem_count),
            }),
            Type::Vector(_) => BrilligVariable::BrilligVector(BrilligVector { pointer: register }),
        }
    }

    /// Free a dead value's register (returning it to the pool) and drop its bookkeeping. Called by
    /// `after_instruction` for each value tracked by the allocator whose last use has passed.
    fn retire_value(&mut self, value_id: &ValueId) {
        if self.is_spilled(value_id) {
            // Spilled: the register was already freed. Just clean up tracking.
            self.spill_manager.as_mut().unwrap().remove_spill(value_id);
            // Only remove from residency if it is actually there. A permanently spilled value may
            // have been filtered out at block entry and never reloaded, so it was never marked.
            if self.resident.is_allocated(value_id) {
                self.resident.mark_unavailable(value_id);
            }
        } else if self.coalescing.has_live_partner(value_id, |v| self.resident.is_allocated(v)) {
            // Shares a register with a coalescing partner that is still alive: do not free the
            // register yet; it is freed when the partner dies (or at block-boundary cleanup).
            self.resident.remove_variable_without_dealloc(value_id);
        } else {
            self.deallocate(value_id);
            if let Some(sm) = self.spill_manager.as_mut() {
                // A value can reach this branch while still owning a spill slot: a transiently
                // spilled value that was reloaded into a register is no longer spilled, yet its
                // transient slot stays reserved. Release it so the offset returns to the free list.
                // For a value with no spill record, or a permanently spilled one, this is a no-op.
                sm.remove_spill(value_id);
            }
        }
    }

    /// Whether the value has a spill slot and is not currently in a register.
    fn is_spilled(&self, value_id: &ValueId) -> bool {
        self.spill_manager.as_ref().is_some_and(|sm| sm.is_spilled(value_id, &self.resident))
    }

    /// Whether the value was transiently spilled and is currently reloaded into a register.
    fn is_transient_reloaded(&self, value_id: &ValueId) -> bool {
        self.spill_manager
            .as_ref()
            .is_some_and(|sm| sm.is_transient_reloaded(value_id, &self.resident))
    }

    /// Check whether allocating `n` more registers would exceed the stack frame limit. Always
    /// `false` when spilling is disabled (including the globals context, whose spill manager is
    /// absent), matching the historical `building_globals || !spill_enabled` guard.
    fn needs_spill_for(&self, n: usize) -> bool {
        if self.spill_manager.is_none() {
            return false;
        }
        self.pool.borrow().available_registers() < n
    }

    /// Ensure there is capacity for `n` more register allocations by spilling if necessary, and
    /// return the stores to emit. It frees registers in the pool and records the spills, but leaves
    /// emission to the driver.
    fn make_room(&mut self, n: usize) -> Vec<Action> {
        if !self.needs_spill_for(n) {
            return Vec::new();
        }
        // Spill `n - available` in a batch first (more efficient, and lets consecutive slots share
        // address computation), then fall back to single spills until capacity is met.
        let available = self.pool.borrow().available_registers();
        let mut actions = self.spill_lru_values(n.saturating_sub(available));
        while self.needs_spill_for(n) {
            actions.extend(self.spill_lru_value());
        }
        actions
    }

    /// Spill the least-recently-used value.
    fn spill_lru_value(&mut self) -> Vec<Action> {
        let victim_id = self
            .spill_manager
            .as_ref()
            .unwrap()
            .lru_victim(&self.resident)
            .expect("No values available to spill");
        self.spill_value(victim_id, false, true)
    }

    /// Spill the `k` least-recently-used values, recording all spills before freeing their
    /// registers so the stores can be emitted as one batch.
    fn spill_lru_values(&mut self, k: usize) -> Vec<Action> {
        let victims = self.spill_manager.as_ref().unwrap().lru_victims(k, &self.resident);
        if victims.is_empty() {
            return Vec::new();
        }

        // Record each spill and collect the stores. A value that already has a slot keeps it and
        // needs no store. The victims are still in registers here (freed in the loop below), so
        // record_spill's double-spill assert sees legitimate spills.
        let mut actions = Vec::with_capacity(victims.len());
        for value_id in &victims {
            let var = *self.ssa_value_allocations.get(value_id).unwrap();
            let prior_offset = self.spill_manager.as_ref().unwrap().get_spill_offset(value_id);
            let offset = match prior_offset {
                Some(offset) => offset,
                None => self.spill_manager.as_mut().unwrap().allocate_spill_offset(),
            };
            self.spill_manager.as_mut().unwrap().record_spill(
                *value_id,
                offset,
                var,
                &self.resident,
            );
            if prior_offset.is_none() {
                actions.push(Action::Spill {
                    value: *value_id,
                    from: var.extract_register(),
                    to: SpillSlot(offset),
                });
            }
        }

        for value_id in &victims {
            self.deallocate(value_id);
        }
        actions
    }

    /// Reload a previously spilled value into a freshly allocated register.
    fn reload_spilled_value(&mut self, value_id: ValueId) -> (BrilligVariable, Vec<Action>) {
        // Ensure capacity for the reload register (may trigger another spill).
        let mut actions = self.make_room(1);

        let spill_record =
            *self.spill_manager.as_ref().unwrap().get_spill(&value_id, &self.resident).unwrap();

        let new_reg = self.pool.borrow_mut().allocate_register();
        actions.push(Action::Reload {
            value: value_id,
            from: SpillSlot(spill_record.offset),
            into: new_reg,
        });

        // Create the updated variable with its new register and update the SSA mapping.
        let new_var = spill_record.variable.with_register(new_reg);
        self.ssa_value_allocations.insert(value_id, new_var);

        // Re-add to residency (removed during spill); this is what marks the value as no longer
        // spilled now that it holds a register again. The slot is kept (the emitted load may
        // re-execute in a loop iteration, so its data must stay valid).
        self.resident.add_available(value_id);
        self.spill_manager.as_mut().unwrap().touch(value_id, &self.resident);

        (new_var, actions)
    }

    /// Free a value's current register (which may differ from its definition register after a
    /// reload) and mark it unavailable in the residency model.
    fn deallocate(&mut self, value_id: &ValueId) {
        self.resident.mark_unavailable(value_id);
        let register = self
            .ssa_value_allocations
            .get(value_id)
            .expect("ICE: Variable allocation not found")
            .extract_register();
        self.pool.borrow_mut().deallocate_register(register);
    }
}
