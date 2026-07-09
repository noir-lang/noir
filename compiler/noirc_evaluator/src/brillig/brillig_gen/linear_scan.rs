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
use super::live_intervals::{LiveInterval, LiveIntervals, ProgramPoint};
use super::spill_manager::SpillManager;
use super::variable_liveness::VariableLiveness;
use crate::brillig::brillig_ir::brillig_variable::BrilligVariable;
use crate::brillig::brillig_ir::registers::RegisterAllocator;
use crate::ssa::ir::basic_block::BasicBlockId;
use crate::ssa::ir::dfg::DataFlowGraph;
use crate::ssa::ir::instruction::InstructionId;
use crate::ssa::ir::value::ValueId;

/// The register or spill-slot home assigned to an SSA value for its entire lifetime.
///
/// The fixed-home model gives each value exactly one location for its whole live range: a register
/// (an index in `[0, value_capacity)` that codegen maps to a concrete `MemoryAddress`) or a spill
/// slot. A value never changes home, so one that lives across a block boundary and got a register
/// simply stays in it — no cross-block spill/reload. That is the reduction in spill traffic over the
/// greedy allocator, which permanently spills every cross-block value once spilling is enabled.
// The plan API is consumed by `LinearScanAllocator` in the next milestone; unused until then.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Home {
    /// Register index in `[0, value_capacity)`.
    Register(usize),
    /// Spill-slot offset.
    Spill(usize),
}

/// A global register-allocation plan: the home of every value, plus the spill-slot count.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub(crate) struct Plan {
    homes: HashMap<ValueId, Home>,
    num_spill_slots: usize,
}

#[allow(dead_code)]
impl Plan {
    /// The home assigned to `value`, or `None` if the value is not tracked (e.g. a global).
    pub(crate) fn home(&self, value: ValueId) -> Option<Home> {
        self.homes.get(&value).copied()
    }

    /// The number of distinct spill slots the plan uses (0 if nothing spilled).
    pub(crate) fn num_spill_slots(&self) -> usize {
        self.num_spill_slots
    }
}

/// Compute the fixed-home assignment by linear scan (Poletto & Sarkar, "Linear Scan Register
/// Allocation", 1999) over the value intervals, with a value-register capacity of `value_capacity`.
///
/// `value_capacity` is `usable_registers - min_live_count`. Reserving `min_live_count` registers
/// leaves every instruction room for its working set (operands, result, and scratch, which reuse
/// one another up to the per-instruction floor `min_live_count`), so bounding register-homed values
/// to `value_capacity` at every point guarantees codegen never overflows the frame. Values that do
/// not fit are spilled whole, evicting the furthest-`last_use` interval — the classic heuristic,
/// applied globally because a fixed home cannot be split.
#[allow(dead_code)]
pub(crate) fn assign(intervals: &LiveIntervals, value_capacity: usize) -> Plan {
    let sorted = intervals.intervals_by_def();

    // Free register indices; kept as a stack with the lowest index on top so `pop` is deterministic.
    let mut free: Vec<usize> = (0..value_capacity).rev().collect();
    // Register-homed values still live, with their interval and assigned index.
    let mut active: Vec<(ValueId, LiveInterval, usize)> = Vec::new();
    let mut homes: HashMap<ValueId, Home> = HashMap::default();
    let mut num_spill_slots = 0usize;

    for (value, interval) in sorted {
        // Free the registers of intervals that ended before this one begins.
        expire(&mut active, &mut free, interval.def);

        if let Some(index) = free.pop() {
            homes.insert(value, Home::Register(index));
            active.push((value, interval, index));
            continue;
        }

        // The frame's value budget is full. Spill either the active value that lives longest — and
        // give this value its register — or this value itself, whichever dies later.
        let furthest = active
            .iter()
            .enumerate()
            .max_by_key(|(_, (_, iv, _))| iv.last_use)
            .map(|(pos, (v, iv, idx))| (pos, *v, iv.last_use, *idx));

        match furthest {
            Some((pos, spilled_value, spilled_last_use, index))
                if spilled_last_use > interval.last_use =>
            {
                active.remove(pos);
                homes.insert(spilled_value, Home::Spill(num_spill_slots));
                num_spill_slots += 1;
                homes.insert(value, Home::Register(index));
                active.push((value, interval, index));
            }
            _ => {
                homes.insert(value, Home::Spill(num_spill_slots));
                num_spill_slots += 1;
            }
        }
    }

    Plan { homes, num_spill_slots }
}

/// Remove active intervals whose `last_use` is before `point`, returning their registers to `free`.
#[allow(dead_code)]
fn expire(
    active: &mut Vec<(ValueId, LiveInterval, usize)>,
    free: &mut Vec<usize>,
    point: ProgramPoint,
) {
    active.retain(|(_, interval, index)| {
        if interval.last_use < point {
            free.push(*index);
            false
        } else {
            true
        }
    });
}

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

#[cfg(test)]
mod assignment_tests {
    use super::{Home, Plan, assign};
    use crate::brillig::brillig_gen::constant_allocation::ConstantAllocation;
    use crate::brillig::brillig_gen::live_intervals::LiveIntervals;
    use crate::brillig::brillig_gen::variable_liveness::VariableLiveness;
    use crate::ssa::ir::post_order::PostOrder;
    use crate::ssa::ir::value::ValueId;
    use crate::ssa::ssa_gen::Ssa;

    fn intervals_for(src: &str) -> LiveIntervals {
        let ssa = Ssa::from_str(src).unwrap();
        let func = ssa.main();
        let constants = ConstantAllocation::from_function(func);
        let liveness = VariableLiveness::from_function(func, &constants);
        let post_order = PostOrder::with_function(func).into_vec();
        LiveIntervals::from_function(func, &liveness, &constants, &post_order)
    }

    fn register_homes(intervals: &LiveIntervals, plan: &Plan) -> Vec<(ValueId, usize)> {
        intervals
            .intervals_by_def()
            .into_iter()
            .filter_map(|(v, _)| match plan.home(v) {
                Some(Home::Register(index)) => Some((v, index)),
                _ => None,
            })
            .collect()
    }

    fn spilled(intervals: &LiveIntervals, plan: &Plan) -> Vec<ValueId> {
        intervals
            .intervals_by_def()
            .into_iter()
            .filter_map(|(v, _)| matches!(plan.home(v), Some(Home::Spill(_))).then_some(v))
            .collect()
    }

    /// The core soundness invariant: two values that interfere must never share a register index,
    /// and every register index is within the capacity.
    fn assert_sound(intervals: &LiveIntervals, plan: &Plan, capacity: usize) {
        let homes = register_homes(intervals, plan);
        for (i, &(a, ia)) in homes.iter().enumerate() {
            assert!(ia < capacity, "register index {ia} exceeds capacity {capacity}");
            for &(b, ib) in &homes[i + 1..] {
                if ia == ib {
                    assert!(
                        !intervals.interferes(a, b),
                        "interfering values {a} and {b} share register index {ia}"
                    );
                }
            }
        }
        // Every value has a home.
        for (v, _) in intervals.intervals_by_def() {
            assert!(plan.home(v).is_some(), "value {v} has no home");
        }
    }

    #[test]
    fn ample_capacity_spills_nothing() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = add v0, Field 1
            v2 = add v1, Field 2
            v3 = add v2, Field 3
            return v3
        }
        ";
        let intervals = intervals_for(src);
        let plan = assign(&intervals, 8);
        assert_eq!(plan.num_spill_slots(), 0, "nothing should spill with ample capacity");
        assert_sound(&intervals, &plan, 8);
    }

    #[test]
    fn tight_capacity_spills_the_overflow_soundly() {
        // Several values are simultaneously live; a capacity of 2 forces spilling.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            v3 = mul v0, v1
            v4 = add v2, v3
            v5 = mul v2, v3
            v6 = add v4, v5
            return v6
        }
        ";
        let intervals = intervals_for(src);
        let plan = assign(&intervals, 2);
        assert!(plan.num_spill_slots() > 0, "tight capacity should force spills");
        assert_sound(&intervals, &plan, 2);
    }

    #[test]
    fn zero_capacity_spills_everything() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = add v0, Field 1
            return v1
        }
        ";
        let intervals = intervals_for(src);
        let plan = assign(&intervals, 0);
        let total = intervals.intervals_by_def().len();
        assert_eq!(plan.num_spill_slots(), total, "with no registers every value spills");
        assert!(register_homes(&intervals, &plan).is_empty());
        assert_eq!(spilled(&intervals, &plan).len(), total);
    }

    #[test]
    fn cross_block_value_keeps_its_register() {
        // `v1` is live across the branch into both successors and the merge. With ample capacity the
        // fixed-home model must keep it in a register the whole time — the behaviour greedy lacks
        // (it would permanently spill it once spilling is on).
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1, v1: Field):
            jmpif v0 then: b1(), else: b2()
          b1():
            v2 = add v1, Field 1
            jmp b3(v2)
          b2():
            v3 = mul v1, Field 2
            jmp b3(v3)
          b3(v4: Field):
            return v4
        }
        ";
        let intervals = intervals_for(src);
        let plan = assign(&intervals, 8);
        let ssa = Ssa::from_str(src).unwrap();
        let v1 = ssa.main().dfg[ssa.main().entry_block()].parameters()[1];
        assert!(
            matches!(plan.home(v1), Some(Home::Register(_))),
            "cross-block value should keep a register with ample capacity"
        );
        assert_sound(&intervals, &plan, 8);
    }
}
