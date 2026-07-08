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
use rustc_hash::FxHashMap as HashMap;

use super::coalescing::CoalescingMap;
use super::spill_manager::SpillManager;
use crate::brillig::brillig_ir::brillig_variable::BrilligVariable;
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
