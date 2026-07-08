//! The pluggable register-allocation seam for Brillig codegen.
//!
//! Brillig has no hardware register file; "registers" are the low slots of a function's stack
//! frame. Allocation decides which SSA value lives in which slot and what to do when a frame runs
//! out of slots (spill to the heap-backed spill region). Historically those decisions were made
//! inline by [`BrilligBlock`](super::brillig_block::BrilligBlock), interleaved with opcode
//! emission. This module introduces the currency the allocator and codegen communicate in — an
//! [`Action`] describing one unit of memory traffic — so the allocation *decisions* can be lifted
//! behind an interface while codegen becomes a pure consumer that emits whatever the allocator
//! dictates.
//!
//! See `design/register_allocation.md` (Phase 0.5) for the full plan.

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

/// The register-allocation state and decisions for one function's Brillig codegen.
///
/// This gathers the state that was previously spread across [`FunctionContext`] — the persistent
/// SSA-value → register cache, the spill manager, and the coalescing map — under one owner. It is
/// the concrete "greedy + LRU spilling" allocator; the pluggable seam that lets a different
/// strategy (e.g. linear scan) take its place is introduced on top of it.
///
/// [`FunctionContext`]: super::brillig_fn::FunctionContext
#[derive(Default)]
pub(crate) struct GreedyAllocator {
    /// Map from SSA values to their allocation. Since values are defined only once in SSA form,
    /// we insert them here when we allocate them at their definition.
    ///
    /// Multiple variables could be assigned the same slot, because this structure accumulates
    /// historical allocations, not just the currently active ones. This is needed so that
    /// when we start processing a block, we can always look up the allocation of the variables
    /// which are live at the beginning of it, even if they were deemed dead by another block
    /// we already visited.
    ///
    /// Note that we don't use `Allocated<BrilligVariable>` here, because we create a fresh
    /// allocator for each block we process, and something that is allocated in e.g. block 1
    /// might be deallocated in block 2, so it has to be done manually.
    pub(crate) ssa_value_allocations: HashMap<ValueId, BrilligVariable>,
    /// Manages spilling of register values to the heap spill region when register pressure
    /// exceeds the stack frame limit. Persists across blocks so spill state is not lost.
    /// Present only when the function may need spilling (based on liveness analysis).
    pub(crate) spill_manager: Option<SpillManager>,
    /// Coalescing map for jmp argument → block parameter register sharing.
    pub(crate) coalescing: CoalescingMap,
}

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
/// Register-facing endpoints are [`MemoryAddress`]; the spill region is addressed through
/// [`SpillSlot`], which the allocator resolves from its own bookkeeping. Codegen applies an action
/// by emitting its opcodes only — the register shadow and spill records are the allocator's to
/// maintain.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Action {
    /// Store a register-resident value into its spill slot.
    Spill { from: MemoryAddress, to: SpillSlot },
    /// Load a spilled value from its slot into a register.
    Reload { from: SpillSlot, into: MemoryAddress },
}

/// The register-allocation seam: codegen asks the allocator where each value lives and what memory
/// traffic to emit, and the allocator makes those decisions and maintains the register/spill state.
///
/// The driver ([`BrilligBlock`](super::brillig_block::BrilligBlock)) owns opcode emission and the
/// per-block register shadow ([`BlockVariables`], passed in); the allocator owns the physical
/// register pool (allocating from and freeing to the [`BrilligContext`] passed in), the spill slots,
/// and the SSA-value → register cache. Methods that need to move data return the [`Action`]s the
/// driver must emit; methods that only reserve or free registers return nothing to emit.
pub(crate) trait Allocator {
    /// Re-seed for a new block. Strips permanently-spilled values from `live_in` (they are reloaded
    /// on demand, not pre-allocated) and resets the per-block eviction state.
    fn begin_block(&mut self, live_in: &mut HashSet<ValueId>);

    /// Bring a value into existence at its definition point (a constant, an instruction result, or
    /// a parameter). Reserves its register — spilling LRU victims first if the frame is full — and
    /// returns the allocation plus the spills that freed the room. The driver then writes the value
    /// into the returned register.
    fn define_variable<F, R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<F, R>,
        variables: &mut BlockVariables,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> (BrilligVariable, Vec<Action>);

    /// Make an already-defined value register-resident and return its allocation, plus the actions
    /// to get it there: a [`Action::Reload`] if it was spilled (and any [`Action::Spill`]s to free
    /// a slot for it), or no actions if it is already resident.
    fn use_variable<F, R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<F, R>,
        variables: &mut BlockVariables,
        value_id: ValueId,
    ) -> (BrilligVariable, Vec<Action>);

    /// Spill a value to a slot, freeing its register. `permanent` slots survive the whole function
    /// (used for cross-block values); `emit_store = false` skips the store for block parameters
    /// whose register does not yet hold the final value. Returns the store to emit, if any.
    fn spill_value<F, R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<F, R>,
        variables: &mut BlockVariables,
        value_id: ValueId,
        permanent: bool,
        emit_store: bool,
    ) -> Vec<Action>;

    /// Whether the value has a spill slot and is not currently in a register.
    fn is_spilled(&self, value_id: &ValueId, variables: &BlockVariables) -> bool;

    /// Whether the value was transiently spilled and is currently reloaded into a register.
    fn is_transient_reloaded(&self, value_id: &ValueId, variables: &BlockVariables) -> bool;
}

impl Allocator for GreedyAllocator {
    fn begin_block(&mut self, live_in: &mut HashSet<ValueId>) {
        if let Some(sm) = self.spill_manager.as_mut() {
            sm.begin_block(live_in);
        }
    }

    fn define_variable<F, R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<F, R>,
        variables: &mut BlockVariables,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> (BrilligVariable, Vec<Action>) {
        // Making room here mirrors the historical `ensure_register_capacity(1)`. When coalescing is
        // active spilling is disabled (they are mutually exclusive), so this is a no-op on the
        // coalesced path below.
        let actions = self.make_room(brillig_context, variables, 1);

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
                variables.is_allocated(&param),
                "ICE: Coalesced parameter not currently available"
            );
            self.ssa_value_allocations.insert(value_id, variable);
            variables.add_available(value_id);
            return (variable, actions);
        }

        // Allocators are rebuilt per block, with visible variables becoming pre-allocated in the
        // next block; what is allocated in one block might be deallocated in another. Because of
        // this we detach from the allocator and manage deallocation manually.
        let variable = allocate_value(value_id, brillig_context, dfg).detach();

        if self.ssa_value_allocations.insert(value_id, variable).is_some() {
            unreachable!("ICE: ValueId {value_id:?} was already in cache");
        }
        variables.add_available(value_id);

        if let Some(sm) = self.spill_manager.as_mut() {
            sm.touch(value_id, variables);
        }

        (variable, actions)
    }

    fn use_variable<F, R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<F, R>,
        variables: &mut BlockVariables,
        value_id: ValueId,
    ) -> (BrilligVariable, Vec<Action>) {
        if self.is_spilled(&value_id, variables) {
            return self.reload_spilled_value(brillig_context, variables, value_id);
        }

        // Already resident: fetch the cached allocation and mark it most-recently-used.
        assert!(variables.is_allocated(&value_id), "ICE: ValueId {value_id:?} is not available");
        let variable = *self
            .ssa_value_allocations
            .get(&value_id)
            .unwrap_or_else(|| panic!("ICE: Value not found in cache {value_id}"));
        if let Some(sm) = self.spill_manager.as_mut() {
            sm.touch(value_id, variables);
        }
        (variable, Vec::new())
    }

    fn spill_value<F, R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<F, R>,
        variables: &mut BlockVariables,
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
            let was_transient_reloaded = self.is_transient_reloaded(&value_id, variables);
            let promoted = self
                .spill_manager
                .as_mut()
                .expect("ICE: spill_value called without spill manager")
                .ensure_permanent_spill(&value_id);
            if promoted {
                if was_transient_reloaded {
                    self.deallocate(brillig_context, variables, &value_id);
                }
                return Vec::new();
            }
        }

        if self.is_spilled(&value_id, variables) {
            return Vec::new();
        }

        let var = *self.ssa_value_allocations.get(&value_id).unwrap();
        let sm = self.spill_manager.as_mut().unwrap();
        let prior_offset = sm.get_spill_offset(&value_id);
        let offset = prior_offset.unwrap_or_else(|| sm.allocate_spill_offset());
        if permanent {
            sm.record_permanent_spill(value_id, offset, var);
        } else {
            // `value_id` is still in a register here (we free it below), so record_spill's
            // double-spill assert sees a legitimate spill.
            sm.record_spill(value_id, offset, var, variables);
        }

        // Only store when we've just allocated the slot. If the value already had a slot, the slot
        // still holds the correct value (SSA values are immutable) so the store would be redundant.
        let actions = if emit_store && prior_offset.is_none() {
            vec![Action::Spill { from: var.extract_register(), to: SpillSlot(offset) }]
        } else {
            Vec::new()
        };

        self.deallocate(brillig_context, variables, &value_id);
        actions
    }

    fn is_spilled(&self, value_id: &ValueId, variables: &BlockVariables) -> bool {
        self.spill_manager.as_ref().is_some_and(|sm| sm.is_spilled(value_id, variables))
    }

    fn is_transient_reloaded(&self, value_id: &ValueId, variables: &BlockVariables) -> bool {
        self.spill_manager.as_ref().is_some_and(|sm| sm.is_transient_reloaded(value_id, variables))
    }
}

impl GreedyAllocator {
    /// Check whether allocating `n` more registers would exceed the stack frame limit. Always
    /// `false` when spilling is disabled (including the globals context, whose spill manager is
    /// absent), matching the historical `building_globals || !spill_enabled` guard.
    fn needs_spill_for<F, R: RegisterAllocator>(
        &self,
        brillig_context: &BrilligContext<F, R>,
        n: usize,
    ) -> bool {
        if self.spill_manager.is_none() {
            return false;
        }
        brillig_context.registers().available_registers() < n
    }

    /// Ensure there is capacity for `n` more register allocations by spilling if necessary, and
    /// return the stores to emit. This is the decision side of the old `ensure_register_capacity`:
    /// it frees registers in the pool and records the spills, but leaves emission to the driver.
    pub(crate) fn make_room<F, R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<F, R>,
        variables: &mut BlockVariables,
        n: usize,
    ) -> Vec<Action> {
        if !self.needs_spill_for(brillig_context, n) {
            return Vec::new();
        }
        // Spill `n - available` in a batch first (more efficient, and lets consecutive slots share
        // address computation), then fall back to single spills until capacity is met.
        let available = brillig_context.registers().available_registers();
        let mut actions =
            self.spill_lru_values(brillig_context, variables, n.saturating_sub(available));
        while self.needs_spill_for(brillig_context, n) {
            actions.extend(self.spill_lru_value(brillig_context, variables));
        }
        actions
    }

    /// Spill the least-recently-used value.
    fn spill_lru_value<F, R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<F, R>,
        variables: &mut BlockVariables,
    ) -> Vec<Action> {
        let victim_id = self
            .spill_manager
            .as_ref()
            .unwrap()
            .lru_victim(variables)
            .expect("No values available to spill");
        self.spill_value(brillig_context, variables, victim_id, false, true)
    }

    /// Spill the `k` least-recently-used values, recording all spills before freeing their
    /// registers so the stores can be emitted as one batch.
    fn spill_lru_values<F, R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<F, R>,
        variables: &mut BlockVariables,
        k: usize,
    ) -> Vec<Action> {
        let victims = self.spill_manager.as_ref().unwrap().lru_victims(k, variables);
        if victims.is_empty() {
            return Vec::new();
        }

        // Record each spill and collect the stores. A value that already has a slot keeps it and
        // needs no store. The victims are still in registers here (freed in the loop below), so
        // record_spill's double-spill assert sees legitimate spills.
        let mut actions = Vec::with_capacity(victims.len());
        for value_id in &victims {
            let var = *self.ssa_value_allocations.get(value_id).unwrap();
            let sm = self.spill_manager.as_mut().unwrap();
            let prior_offset = sm.get_spill_offset(value_id);
            let offset = prior_offset.unwrap_or_else(|| sm.allocate_spill_offset());
            sm.record_spill(*value_id, offset, var, variables);
            if prior_offset.is_none() {
                actions.push(Action::Spill { from: var.extract_register(), to: SpillSlot(offset) });
            }
        }

        for value_id in &victims {
            self.deallocate(brillig_context, variables, value_id);
        }
        actions
    }

    /// Reload a previously spilled value into a freshly allocated register.
    fn reload_spilled_value<F, R: RegisterAllocator>(
        &mut self,
        brillig_context: &BrilligContext<F, R>,
        variables: &mut BlockVariables,
        value_id: ValueId,
    ) -> (BrilligVariable, Vec<Action>) {
        // Ensure capacity for the reload register (may trigger another spill).
        let mut actions = self.make_room(brillig_context, variables, 1);

        let spill_record =
            *self.spill_manager.as_ref().unwrap().get_spill(&value_id, variables).unwrap();

        let new_reg = brillig_context.allocate_register().detach();
        actions.push(Action::Reload { from: SpillSlot(spill_record.offset), into: new_reg });

        // Create the updated variable with its new register and update the SSA mapping.
        let new_var = spill_record.variable.with_register(new_reg);
        self.ssa_value_allocations.insert(value_id, new_var);

        // Re-add to available variables (removed during spill); this is what marks the value as no
        // longer spilled now that it holds a register again. The slot is kept (the emitted load may
        // re-execute in a loop iteration, so its data must stay valid).
        variables.add_available(value_id);
        self.spill_manager.as_mut().unwrap().touch(value_id, variables);

        (new_var, actions)
    }

    /// Free a value's current register (which may differ from its definition register after a
    /// reload) and mark it unavailable in the register shadow.
    fn deallocate<F, R: RegisterAllocator>(
        &self,
        brillig_context: &BrilligContext<F, R>,
        variables: &mut BlockVariables,
        value_id: &ValueId,
    ) {
        variables.mark_unavailable(value_id);
        let register = self
            .ssa_value_allocations
            .get(value_id)
            .expect("ICE: Variable allocation not found")
            .extract_register();
        brillig_context.deallocate_register(register);
    }
}
