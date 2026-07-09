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
use super::live_intervals::{LiveRanges, ProgramPoint};
use super::spill_manager::SpillManager;
use super::variable_liveness::VariableLiveness;
use crate::brillig::brillig_ir::brillig_variable::BrilligVariable;
use crate::brillig::brillig_ir::registers::RegisterAllocator;
use crate::ssa::ir::basic_block::BasicBlockId;
use crate::ssa::ir::dfg::DataFlowGraph;
use crate::ssa::ir::instruction::InstructionId;
use crate::ssa::ir::value::ValueId;

/// Where a value physically lives over one stretch of its lifetime.
///
/// A value's register can change over time (interval splitting under pressure), but its spill slot
/// is fixed — spills are unbounded, so a value keeps one slot for the whole function and
/// non-interfering values pack into shared slots. So only [`Location::Register`] varies point to
/// point; a spilled value is always in its one slot.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Location {
    /// A register, identified by index in `[0, value_capacity)` (codegen maps it to a concrete
    /// `MemoryAddress`).
    Register(usize),
    /// The value's fixed spill slot (offset in the spill region).
    Slot(usize),
}

/// One contiguous stretch `[start, end]` over which a value sits in a single [`Location`].
///
/// A value's timeline is a list of these. Consecutive segments with different locations are the
/// split points where codegen emits a spill/reload/move; a gap between segments is a liveness hole
/// (the value is dead there and its register is free for another value).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Segment {
    pub(crate) start: ProgramPoint,
    pub(crate) end: ProgramPoint,
    pub(crate) location: Location,
}

/// The global register-allocation plan: each value's location timeline, plus the spill-slot count.
///
/// Read-only once built. The allocator answers "where is `v` at point `P`" by finding the segment
/// containing `P`, and derives the spill/reload/move actions at `P` from the segment boundaries.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub(crate) struct Plan {
    timeline: HashMap<ValueId, Vec<Segment>>,
    num_spill_slots: usize,
}

#[allow(dead_code)]
impl Plan {
    /// The value's location timeline (segments in point order), or empty if the value is untracked.
    pub(crate) fn timeline(&self, value: ValueId) -> &[Segment] {
        self.timeline.get(&value).map_or(&[], Vec::as_slice)
    }

    /// Where `value` lives at `point`, or `None` if it is dead there (in a hole) or untracked.
    pub(crate) fn location_at(&self, value: ValueId, point: ProgramPoint) -> Option<Location> {
        self.timeline(value)
            .iter()
            .find(|seg| seg.start <= point && point <= seg.end)
            .map(|seg| seg.location)
    }

    /// The number of distinct spill slots the plan uses (0 if nothing spilled).
    pub(crate) fn num_spill_slots(&self) -> usize {
        self.num_spill_slots
    }
}

/// A `[start, end]` live range tagged with the value it belongs to — the unit the scan consumes.
struct RangeOf {
    start: ProgramPoint,
    end: ProgramPoint,
    value: ValueId,
}

/// Assign registers by linear scan over the hole-aware [`LiveRanges`], with a value-register
/// capacity of `value_capacity`.
///
/// Each of a value's live ranges is scanned as a separate interval, but a value **reclaims its
/// previous register** when a later range begins if that register is still free — so a value with a
/// hole on its own runtime path keeps one register (no spurious split), while a value dead across a
/// *divergent-path* hole releases its register for another value to share during the hole.
///
/// This function handles only the case that fits in `value_capacity` (no pressure spilling yet):
/// it returns `None` if a program point needs more registers than are available, which the caller
/// treats as "linear scan cannot place this function" and falls back. Pressure spilling to the
/// fixed per-value slot is layered on next.
#[allow(dead_code)]
pub(crate) fn assign(ranges: &LiveRanges, value_capacity: usize) -> Option<Plan> {
    // Flatten to one interval per (value, range), scanned in start order.
    let mut intervals: Vec<RangeOf> = ranges
        .iter()
        .flat_map(|(value, list)| {
            list.iter().map(move |range| RangeOf { start: range.start, end: range.end, value })
        })
        .collect();
    intervals.sort_by_key(|r| (r.start, r.end, r.value));

    let mut free: std::collections::BTreeSet<usize> = (0..value_capacity).collect();
    // Currently register-resident ranges: (range end, value, register index).
    let mut active: Vec<(ProgramPoint, ValueId, usize)> = Vec::new();
    // A value's most recent register, so a later range can reclaim it when free.
    let mut previous_register: HashMap<ValueId, usize> = HashMap::default();
    let mut timeline: HashMap<ValueId, Vec<Segment>> = HashMap::default();

    for RangeOf { start, end, value } in intervals {
        // Expire ranges that ended strictly before this one begins, freeing their registers back to
        // the pool. Ranges touching at a point (`prev.end == start`) stay active — they interfere.
        active.retain(|&(active_end, _, register)| {
            let expired = active_end < start;
            if expired {
                free.insert(register);
            }
            !expired
        });

        // Prefer the value's previous register (reclaim across a hole), else the lowest free one.
        let register = match previous_register.get(&value) {
            Some(&prev) if free.remove(&prev) => prev,
            _ => *free.iter().next()?,
        };
        free.remove(&register);

        previous_register.insert(value, register);
        active.push((end, value, register));
        timeline.entry(value).or_default().push(Segment {
            start,
            end,
            location: Location::Register(register),
        });
    }

    // Keep each value's segments in point order.
    for segments in timeline.values_mut() {
        segments.sort_by_key(|seg| seg.start);
    }

    Some(Plan { timeline, num_spill_slots: 0 })
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
    use super::{Location, Plan, Segment, assign};
    use crate::brillig::brillig_gen::constant_allocation::ConstantAllocation;
    use crate::brillig::brillig_gen::live_intervals::{LiveIntervals, LiveRanges};
    use crate::brillig::brillig_gen::variable_liveness::VariableLiveness;
    use crate::ssa::ir::post_order::PostOrder;
    use crate::ssa::ir::value::ValueId;
    use crate::ssa::ssa_gen::Ssa;

    fn ranges_for(src: &str) -> (LiveRanges, Ssa) {
        let ssa = Ssa::from_str(src).unwrap();
        let func = ssa.main();
        let constants = ConstantAllocation::from_function(func);
        let liveness = VariableLiveness::from_function(func, &constants);
        let post_order = PostOrder::with_function(func).into_vec();
        let intervals = LiveIntervals::from_function(func, &liveness, &constants, &post_order);
        (LiveRanges::from_intervals(&intervals, &liveness, &post_order), ssa)
    }

    fn segments_overlap(a: &Segment, b: &Segment) -> bool {
        a.start <= b.end && b.start <= a.end
    }

    /// The core soundness invariant: no two register segments of *different* values that share a
    /// register index overlap in program points (interfering values never co-occupy a register),
    /// and every register index is within capacity. Also every value has at least one segment.
    fn assert_sound(ranges: &LiveRanges, plan: &Plan, capacity: usize) {
        let mut register_segments: Vec<(ValueId, Segment)> = Vec::new();
        for (value, _) in ranges.iter() {
            assert!(!plan.timeline(value).is_empty(), "value {value} has no timeline");
            for seg in plan.timeline(value) {
                if let Location::Register(index) = seg.location {
                    assert!(index < capacity, "register index {index} exceeds capacity {capacity}");
                    register_segments.push((value, *seg));
                }
            }
        }
        for (i, (va, sa)) in register_segments.iter().enumerate() {
            for (vb, sb) in &register_segments[i + 1..] {
                if va != vb && sa.location == sb.location && segments_overlap(sa, sb) {
                    panic!("values {va} and {vb} share {:?} over overlapping ranges", sa.location);
                }
            }
        }
    }

    #[test]
    fn no_pressure_assigns_registers_soundly() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = add v0, Field 1
            v2 = add v1, Field 2
            v3 = add v2, Field 3
            return v3
        }
        ";
        let (ranges, _ssa) = ranges_for(src);
        let plan = assign(&ranges, 8).expect("fits in 8 registers");
        assert_eq!(plan.num_spill_slots(), 0);
        assert_sound(&ranges, &plan, 8);
        // Nothing is spilled: every segment is a register.
        for (value, _) in ranges.iter() {
            for seg in plan.timeline(value) {
                assert!(matches!(seg.location, Location::Register(_)));
            }
        }
    }

    #[test]
    fn pressure_beyond_capacity_is_not_yet_placeable() {
        // Five values are live at once; capacity 2 cannot hold them and pressure spilling is not
        // implemented yet, so the scan reports the function as unplaceable.
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
        let (ranges, _ssa) = ranges_for(src);
        assert!(assign(&ranges, 2).is_none(), "capacity 2 should be unplaceable without spilling");
    }

    #[test]
    fn cross_block_value_gets_one_register() {
        // `v1` is live across the branch into both successors and the merge. Linear scan keeps it in
        // a single register the whole time — no cross-block spill/reload, the behaviour greedy lacks
        // (it permanently spills every cross-block value once spilling is on).
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
        let (ranges, ssa) = ranges_for(src);
        let plan = assign(&ranges, 8).expect("fits in 8 registers");
        let func = ssa.main();
        let v1 = func.dfg[func.entry_block()].parameters()[1];

        let segments = plan.timeline(v1);
        assert!(!segments.is_empty(), "v1 should have a timeline");
        let first = match segments[0].location {
            Location::Register(index) => index,
            Location::Slot(_) => panic!("v1 should be register-homed"),
        };
        for seg in segments {
            assert_eq!(
                seg.location,
                Location::Register(first),
                "v1 should keep one register across all its ranges"
            );
        }
        assert_sound(&ranges, &plan, 8);
    }
}
