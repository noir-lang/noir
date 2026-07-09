//! Linear-scan register allocation for Brillig (Phase 1 of the plan in
//! `design/register_allocation.md`).
//!
//! Where [`GreedyAllocator`] makes spill/placement decisions *online* (LRU eviction as it runs),
//! the linear-scan allocator computes a global assignment *up front* from the value
//! [`LiveIntervals`](super::live_intervals::LiveIntervals) and then serves it as read-only queries.
//! Both implement the same [`Allocator`] seam, so the driver ([`BrilligBlock`]) is identical for
//! either; which one runs is chosen per function by [`FunctionAllocator`].
//!
//! **Scaffold status.** [`LinearScanAllocator`] currently delegates to [`GreedyAllocator`]. This
//! lets the selection seam — the flag, the polymorphic [`FunctionContext`] field, construction, the
//! globals `into_allocations` path — be exercised end-to-end and proven behavior-preserving before
//! the plan-based internals land. Selecting linear scan today therefore reproduces greedy output
//! exactly; the assignment pass replaces these delegations.
//!
//! [`BrilligBlock`]: super::brillig_block::BrilligBlock
//! [`FunctionContext`]: super::brillig_fn::FunctionContext

use std::cell::RefCell;
use std::rc::Rc;

use acvm::acir::brillig::MemoryAddress;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::allocator::{Action, Allocator, GreedyAllocator, ParamHome};
use super::coalescing::CoalescingMap;
use super::spill_manager::SpillManager;
use super::variable_liveness::VariableLiveness;
use crate::brillig::brillig_ir::brillig_variable::BrilligVariable;
use crate::brillig::brillig_ir::registers::RegisterAllocator;
use crate::ssa::ir::basic_block::BasicBlockId;
use crate::ssa::ir::dfg::DataFlowGraph;
use crate::ssa::ir::instruction::InstructionId;
use crate::ssa::ir::value::ValueId;

/// The plan-based allocator. See the module docs; presently a delegating scaffold over
/// [`GreedyAllocator`] pending the assignment pass.
pub(crate) struct LinearScanAllocator<R: RegisterAllocator> {
    inner: GreedyAllocator<R>,
}

impl<R: RegisterAllocator> LinearScanAllocator<R> {
    /// Build the linear-scan allocator. Takes the same inputs as the greedy allocator so the two
    /// are interchangeable at the construction site; the plan is derived from `liveness`.
    pub(crate) fn new(
        pool: Rc<RefCell<R>>,
        spill_manager: Option<SpillManager>,
        coalescing: CoalescingMap,
        liveness: VariableLiveness,
        last_uses: HashMap<InstructionId, HashSet<ValueId>>,
    ) -> Self {
        Self { inner: GreedyAllocator::new(pool, spill_manager, coalescing, liveness, last_uses) }
    }

    /// The final register allocations, consumed when handing the global allocations to the artifact.
    pub(crate) fn into_allocations(self) -> HashMap<ValueId, BrilligVariable> {
        self.inner.into_allocations()
    }

    #[cfg(test)]
    pub(crate) fn get_coalesced(&self, value_id: &ValueId) -> Option<ValueId> {
        self.inner.get_coalesced(value_id)
    }

    #[cfg(test)]
    pub(crate) fn liveness(&self) -> &VariableLiveness {
        self.inner.liveness()
    }

    #[cfg(test)]
    pub(crate) fn retire(&mut self, value_id: &ValueId) {
        self.inner.retire(value_id);
    }
}

impl<R: RegisterAllocator> Allocator for LinearScanAllocator<R> {
    fn begin_block(
        &mut self,
        block: BasicBlockId,
        dfg: &DataFlowGraph,
    ) -> (Vec<(ValueId, MemoryAddress)>, Vec<Action>) {
        self.inner.begin_block(block, dfg)
    }

    fn define_variable(
        &mut self,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> (BrilligVariable, Vec<Action>) {
        self.inner.define_variable(value_id, dfg)
    }

    fn use_variable(&mut self, value_id: ValueId) -> (BrilligVariable, Vec<Action>) {
        self.inner.use_variable(value_id)
    }

    fn reserve_scratch(&mut self, scratch: usize) -> Vec<Action> {
        self.inner.reserve_scratch(scratch)
    }

    fn after_instruction(&mut self, inst: InstructionId) -> Vec<Action> {
        self.inner.after_instruction(inst)
    }

    fn before_terminator(&mut self, block: BasicBlockId, dfg: &DataFlowGraph) -> Vec<Action> {
        self.inner.before_terminator(block, dfg)
    }

    fn resolve_edge(
        &self,
        pred: BasicBlockId,
        succ: BasicBlockId,
        dfg: &DataFlowGraph,
    ) -> Vec<ParamHome> {
        self.inner.resolve_edge(pred, succ, dfg)
    }

    fn spill_enabled(&self) -> bool {
        self.inner.spill_enabled()
    }

    fn max_spill_offset(&self) -> usize {
        self.inner.max_spill_offset()
    }
}

/// The allocator a [`FunctionContext`](super::brillig_fn::FunctionContext) runs, chosen per function.
///
/// Both variants implement [`Allocator`], so the driver dispatches through this enum without caring
/// which strategy is active. Keeping the choice in an enum (rather than `Box<dyn Allocator>`) lets
/// the non-trait lifecycle methods (`into_allocations`, the test-only inspectors) stay off the
/// [`Allocator`] trait, which is deliberately free of allocator-specific surface.
pub(crate) enum FunctionAllocator<R: RegisterAllocator> {
    Greedy(GreedyAllocator<R>),
    LinearScan(LinearScanAllocator<R>),
}

impl<R: RegisterAllocator> FunctionAllocator<R> {
    /// The final register allocations, consumed when handing the global allocations to the artifact.
    pub(crate) fn into_allocations(self) -> HashMap<ValueId, BrilligVariable> {
        match self {
            Self::Greedy(a) => a.into_allocations(),
            Self::LinearScan(a) => a.into_allocations(),
        }
    }

    #[cfg(test)]
    pub(crate) fn get_coalesced(&self, value_id: &ValueId) -> Option<ValueId> {
        match self {
            Self::Greedy(a) => a.get_coalesced(value_id),
            Self::LinearScan(a) => a.get_coalesced(value_id),
        }
    }

    #[cfg(test)]
    pub(crate) fn liveness(&self) -> &VariableLiveness {
        match self {
            Self::Greedy(a) => a.liveness(),
            Self::LinearScan(a) => a.liveness(),
        }
    }

    #[cfg(test)]
    pub(crate) fn retire(&mut self, value_id: &ValueId) {
        match self {
            Self::Greedy(a) => a.retire(value_id),
            Self::LinearScan(a) => a.retire(value_id),
        }
    }
}

impl<R: RegisterAllocator> Allocator for FunctionAllocator<R> {
    fn begin_block(
        &mut self,
        block: BasicBlockId,
        dfg: &DataFlowGraph,
    ) -> (Vec<(ValueId, MemoryAddress)>, Vec<Action>) {
        match self {
            Self::Greedy(a) => a.begin_block(block, dfg),
            Self::LinearScan(a) => a.begin_block(block, dfg),
        }
    }

    fn define_variable(
        &mut self,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> (BrilligVariable, Vec<Action>) {
        match self {
            Self::Greedy(a) => a.define_variable(value_id, dfg),
            Self::LinearScan(a) => a.define_variable(value_id, dfg),
        }
    }

    fn use_variable(&mut self, value_id: ValueId) -> (BrilligVariable, Vec<Action>) {
        match self {
            Self::Greedy(a) => a.use_variable(value_id),
            Self::LinearScan(a) => a.use_variable(value_id),
        }
    }

    fn reserve_scratch(&mut self, scratch: usize) -> Vec<Action> {
        match self {
            Self::Greedy(a) => a.reserve_scratch(scratch),
            Self::LinearScan(a) => a.reserve_scratch(scratch),
        }
    }

    fn after_instruction(&mut self, inst: InstructionId) -> Vec<Action> {
        match self {
            Self::Greedy(a) => a.after_instruction(inst),
            Self::LinearScan(a) => a.after_instruction(inst),
        }
    }

    fn before_terminator(&mut self, block: BasicBlockId, dfg: &DataFlowGraph) -> Vec<Action> {
        match self {
            Self::Greedy(a) => a.before_terminator(block, dfg),
            Self::LinearScan(a) => a.before_terminator(block, dfg),
        }
    }

    fn resolve_edge(
        &self,
        pred: BasicBlockId,
        succ: BasicBlockId,
        dfg: &DataFlowGraph,
    ) -> Vec<ParamHome> {
        match self {
            Self::Greedy(a) => a.resolve_edge(pred, succ, dfg),
            Self::LinearScan(a) => a.resolve_edge(pred, succ, dfg),
        }
    }

    fn spill_enabled(&self) -> bool {
        match self {
            Self::Greedy(a) => a.spill_enabled(),
            Self::LinearScan(a) => a.spill_enabled(),
        }
    }

    fn max_spill_offset(&self) -> usize {
        match self {
            Self::Greedy(a) => a.max_spill_offset(),
            Self::LinearScan(a) => a.max_spill_offset(),
        }
    }
}
