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
//! addresses returned by `define_variable`/`use_variable`. The driver never tells the allocator
//! what is resident. Because the shadow is a pure function of the allocator's output, swapping the
//! greedy allocator for the linear-scan one requires no change to the driver — proving that is the
//! point of this phase.
//!
//! See `design/register_allocation.md` (Phase 0.5) for the full plan.

use acvm::FieldElement;
use acvm::acir::brillig::MemoryAddress;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::brillig_block_variables::{BlockVariables, allocate_value};
use super::coalescing::CoalescingMap;
use super::spill_manager::SpillManager;
use crate::brillig::brillig_ir::BrilligContext;
use crate::brillig::brillig_ir::brillig_variable::BrilligVariable;
use crate::brillig::brillig_ir::registers::RegisterAllocator;
use crate::ssa::ir::dfg::DataFlowGraph;
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

/// A single unit of memory traffic the allocator asks codegen to emit.
///
/// Every action carries the `value` it concerns so the driver updates its register shadow
/// mechanically as it applies the action: `Reload`/`Move` put the value at the destination
/// register; `Spill` removes it. Register-facing endpoints are [`MemoryAddress`]; the spill region
/// is addressed through [`SpillSlot`], which the allocator resolves from its own bookkeeping.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Action {
    /// Store a register-resident value into its spill slot (the value leaves the shadow).
    Spill { value: ValueId, from: MemoryAddress, to: SpillSlot },
    /// Load a spilled value from its slot into a register (the value enters the shadow at `into`).
    Reload { value: ValueId, from: SpillSlot, into: MemoryAddress },
    /// Register-to-register move (the value moves to `to` in the shadow). Produced by edge
    /// resolution, which the greedy allocator does not yet drive; see `resolve_edge` in the design
    /// doc (Phase 1).
    #[allow(dead_code)]
    Move { value: ValueId, from: MemoryAddress, to: MemoryAddress },
}

/// The register-allocation seam: codegen asks the allocator where each value lives and what memory
/// traffic to emit, and the allocator makes those decisions and maintains the register/spill state.
///
/// The driver ([`BrilligBlock`](super::brillig_block::BrilligBlock)) owns opcode emission and the
/// one-way register shadow (see the module docs); the allocator owns the register pool, the spill
/// slots, and the SSA-value → register cache. Methods that move data return the [`Action`]s the
/// driver must emit; methods that only reserve or free registers return nothing to emit.
///
/// The `brillig_context`/`dfg` parameters remain a stepping stone toward the design's ID-only
/// signatures: the register pool still lives in `BrilligContext`, and `define_variable` still reads
/// the value's type from the DFG. Both are removed once the pool moves into the allocator and value
/// shapes are precomputed (mirroring how the linear-scan allocator precomputes its plan).
pub(crate) trait Allocator {
    /// Re-seed for a new block. `live_in` is the block's non-global live-in set; the allocator
    /// strips permanently-spilled values from it (they are reloaded on demand, not pre-allocated),
    /// resets its per-block eviction state, and returns the register-resident live-ins so the
    /// driver can seed its shadow.
    fn begin_block(&mut self, live_in: &mut HashSet<ValueId>) -> Vec<(ValueId, MemoryAddress)>;

    /// Bring a value into existence at its definition point (a constant, an instruction result, or
    /// a parameter). Reserves its register — spilling LRU victims first if the frame is full — and
    /// returns the allocation plus the spills that freed the room. The driver then writes the value
    /// into the returned register.
    fn define_variable<R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<FieldElement, R>,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> (BrilligVariable, Vec<Action>);

    /// Make an already-defined value register-resident and return its allocation, plus the actions
    /// to get it there: a [`Action::Reload`] if it was spilled (and any [`Action::Spill`]s to free
    /// a slot for it), or no actions if it is already resident.
    fn use_variable<R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<FieldElement, R>,
        value_id: ValueId,
    ) -> (BrilligVariable, Vec<Action>);

    /// Ensure `n` registers are free for codegen to allocate scratch temporaries via its RAII path,
    /// returning the spills that freed the room. The contract is "N slots are free", not "here are
    /// the registers" — codegen owns the temporaries.
    fn before_instruction<R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<FieldElement, R>,
        scratch: usize,
    ) -> Vec<Action>;

    /// Spill a value to a slot, freeing its register. `permanent` slots survive the whole function
    /// (used for cross-block values); `emit_store = false` skips the store for block parameters
    /// whose register does not yet hold the final value. Returns the store to emit, if any.
    ///
    /// This is the terminator/edge spilling primitive the current greedy driver still calls
    /// directly; the design folds it into `before_terminator`/`resolve_edge` (Phase 1), at which
    /// point it stops being part of the trait surface.
    fn spill_value<R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<FieldElement, R>,
        value_id: ValueId,
        permanent: bool,
        emit_store: bool,
    ) -> Vec<Action>;

    /// The permanent spill slot of a value, if it has one. Used by the terminator to write a jmp
    /// argument straight into its destination parameter's slot; subsumed by `resolve_edge`.
    fn permanent_spill_slot(&self, value_id: &ValueId) -> Option<SpillSlot>;

    /// Retire a value at its last use: free its register (returning it to the pool for the next
    /// allocation) and drop any bookkeeping. Emits no opcode, so there is nothing for the driver to
    /// apply. The caller is responsible for skipping globals, which live for the whole program.
    fn retire<R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<FieldElement, R>,
        value_id: &ValueId,
    );

    /// Whether spilling is enabled for this function (there is a spill manager).
    fn spill_enabled(&self) -> bool;

    /// The high-water mark of spill slots used, for sizing the spill prologue (0 if none).
    fn max_spill_offset(&self) -> usize;
}

/// The concrete "greedy + LRU spilling" allocator.
///
/// It owns the authoritative allocation state: the SSA-value → register cache, its own residency
/// model (`resident`), the spill manager, and the coalescing map (a construction-time input, not a
/// rival pass). The pluggable seam that lets a different strategy (e.g. linear scan) take its place
/// is the [`Allocator`] trait above.
#[derive(Default)]
pub(crate) struct GreedyAllocator {
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
}

impl GreedyAllocator {
    /// Build the greedy allocator with the spill manager and coalescing map decided by
    /// [`FunctionContext::new`](super::brillig_fn::FunctionContext::new).
    pub(crate) fn new(spill_manager: Option<SpillManager>, coalescing: CoalescingMap) -> Self {
        Self {
            resident: BlockVariables::default(),
            ssa_value_allocations: HashMap::default(),
            spill_manager,
            coalescing,
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
}

impl Allocator for GreedyAllocator {
    fn begin_block(&mut self, live_in: &mut HashSet<ValueId>) -> Vec<(ValueId, MemoryAddress)> {
        // Strip permanently-spilled live-ins and reset the eviction state for the new block.
        if let Some(sm) = self.spill_manager.as_mut() {
            sm.begin_block(live_in);
        }
        // Seed the allocator's residency with the (non-spilled) live-ins and report their registers
        // so the driver can seed its shadow and pre-allocate the pool.
        self.resident = BlockVariables::new(live_in.clone());
        live_in
            .iter()
            .map(|value_id| {
                let register = self
                    .ssa_value_allocations
                    .get(value_id)
                    .unwrap_or_else(|| panic!("ICE: live-in {value_id} not allocated"))
                    .extract_register();
                (*value_id, register)
            })
            .collect()
    }

    fn define_variable<R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<FieldElement, R>,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> (BrilligVariable, Vec<Action>) {
        // Making room here mirrors the historical `ensure_register_capacity(1)`. When coalescing is
        // active spilling is disabled (they are mutually exclusive), so this is a no-op on the
        // coalesced path below.
        let actions = self.make_room(brillig_context, 1);

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

        // Allocators are rebuilt per block, with visible variables becoming pre-allocated in the
        // next block; what is allocated in one block might be deallocated in another. Because of
        // this we detach from the allocator and manage deallocation manually.
        let variable = allocate_value(value_id, brillig_context, dfg).detach();

        if self.ssa_value_allocations.insert(value_id, variable).is_some() {
            unreachable!("ICE: ValueId {value_id:?} was already in cache");
        }
        self.resident.add_available(value_id);

        if let Some(sm) = self.spill_manager.as_mut() {
            sm.touch(value_id, &self.resident);
        }

        (variable, actions)
    }

    fn use_variable<R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<FieldElement, R>,
        value_id: ValueId,
    ) -> (BrilligVariable, Vec<Action>) {
        if self.is_spilled(&value_id) {
            return self.reload_spilled_value(brillig_context, value_id);
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

    fn before_instruction<R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<FieldElement, R>,
        scratch: usize,
    ) -> Vec<Action> {
        self.make_room(brillig_context, scratch)
    }

    fn spill_value<R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<FieldElement, R>,
        value_id: ValueId,
        permanent: bool,
        emit_store: bool,
    ) -> Vec<Action> {
        // For a permanent spill, try to promote an existing record first.
        if permanent {
            // A reloaded transient value holds a register that must be freed when promoted to a
            // permanent spill. A value not currently in a register must not have its register freed
            // here — it may still be live (e.g. the condition register of a jmpif when
            // `spill_non_param_live_ins` fires multiple times). Capture this before promoting.
            let was_transient_reloaded = self.is_transient_reloaded(&value_id);
            let promoted = self
                .spill_manager
                .as_mut()
                .expect("ICE: spill_value called without spill manager")
                .ensure_permanent_spill(&value_id);
            if promoted {
                if was_transient_reloaded {
                    self.deallocate(brillig_context, &value_id);
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

        self.deallocate(brillig_context, &value_id);
        actions
    }

    fn permanent_spill_slot(&self, value_id: &ValueId) -> Option<SpillSlot> {
        self.spill_manager
            .as_ref()
            .and_then(|sm| sm.get_permanent_spill_offset(value_id))
            .map(SpillSlot)
    }

    fn retire<R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<FieldElement, R>,
        value_id: &ValueId,
    ) {
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
            self.deallocate(brillig_context, value_id);
            if let Some(sm) = self.spill_manager.as_mut() {
                // A value can reach this branch while still owning a spill slot: a transiently
                // spilled value that was reloaded into a register is no longer spilled, yet its
                // transient slot stays reserved. Release it so the offset returns to the free list.
                // For a value with no spill record, or a permanently spilled one, this is a no-op.
                sm.remove_spill(value_id);
            }
        }
    }

    fn spill_enabled(&self) -> bool {
        self.spill_manager.is_some()
    }

    fn max_spill_offset(&self) -> usize {
        self.spill_manager.as_ref().map_or(0, |sm| sm.max_spill_offset())
    }
}

impl GreedyAllocator {
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
    fn needs_spill_for<R: RegisterAllocator>(
        &self,
        brillig_context: &BrilligContext<FieldElement, R>,
        n: usize,
    ) -> bool {
        if self.spill_manager.is_none() {
            return false;
        }
        brillig_context.registers().available_registers() < n
    }

    /// Ensure there is capacity for `n` more register allocations by spilling if necessary, and
    /// return the stores to emit. It frees registers in the pool and records the spills, but leaves
    /// emission to the driver.
    fn make_room<R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<FieldElement, R>,
        n: usize,
    ) -> Vec<Action> {
        if !self.needs_spill_for(brillig_context, n) {
            return Vec::new();
        }
        // Spill `n - available` in a batch first (more efficient, and lets consecutive slots share
        // address computation), then fall back to single spills until capacity is met.
        let available = brillig_context.registers().available_registers();
        let mut actions = self.spill_lru_values(brillig_context, n.saturating_sub(available));
        while self.needs_spill_for(brillig_context, n) {
            actions.extend(self.spill_lru_value(brillig_context));
        }
        actions
    }

    /// Spill the least-recently-used value.
    fn spill_lru_value<R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<FieldElement, R>,
    ) -> Vec<Action> {
        let victim_id = self
            .spill_manager
            .as_ref()
            .unwrap()
            .lru_victim(&self.resident)
            .expect("No values available to spill");
        self.spill_value(brillig_context, victim_id, false, true)
    }

    /// Spill the `k` least-recently-used values, recording all spills before freeing their
    /// registers so the stores can be emitted as one batch.
    fn spill_lru_values<R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<FieldElement, R>,
        k: usize,
    ) -> Vec<Action> {
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
            self.deallocate(brillig_context, value_id);
        }
        actions
    }

    /// Reload a previously spilled value into a freshly allocated register.
    fn reload_spilled_value<R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<FieldElement, R>,
        value_id: ValueId,
    ) -> (BrilligVariable, Vec<Action>) {
        // Ensure capacity for the reload register (may trigger another spill).
        let mut actions = self.make_room(brillig_context, 1);

        let spill_record =
            *self.spill_manager.as_ref().unwrap().get_spill(&value_id, &self.resident).unwrap();

        let new_reg = brillig_context.allocate_register().detach();
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
    fn deallocate<R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<FieldElement, R>,
        value_id: &ValueId,
    ) {
        self.resident.mark_unavailable(value_id);
        let register = self
            .ssa_value_allocations
            .get(value_id)
            .expect("ICE: Variable allocation not found")
            .extract_register();
        brillig_context.deallocate_register(register);
    }
}
