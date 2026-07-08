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
