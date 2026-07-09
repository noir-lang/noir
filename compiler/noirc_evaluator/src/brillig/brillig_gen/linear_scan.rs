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

use acvm::acir::brillig::MemoryAddress;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::allocator::{Action, Allocator, GreedyAllocator, ParamHome};
use super::brillig_block_variables::compute_array_length;
use super::constant_allocation::ConstantAllocation;
use super::live_intervals::{LiveIntervals, LiveRanges, ProgramPoint};
use super::variable_liveness::VariableLiveness;
use crate::brillig::brillig_ir::brillig_variable::{
    BrilligArray, BrilligVariable, BrilligVector, SingleAddrVariable, get_bit_size_from_ssa_type,
};
use crate::brillig::brillig_ir::registers::RegisterAllocator;
use crate::brillig::{assert_u32, assert_usize};
use crate::ssa::ir::basic_block::BasicBlockId;
use crate::ssa::ir::dfg::DataFlowGraph;
use crate::ssa::ir::function::Function;
use crate::ssa::ir::instruction::InstructionId;
use crate::ssa::ir::types::Type;
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
pub(crate) fn assign(
    ranges: &LiveRanges,
    value_capacity: usize,
    entry_params: &[ValueId],
) -> Option<Plan> {
    // Entry-block parameters are pre-colored fixed intervals: the calling convention places argument
    // `i` in register `i` (see `codegen_entry_point::allocate_function_arguments`), so the callee
    // must read parameter `i` from register `i`. Reserve `[0, n_params)` for them, matching order,
    // and never hand those indices to other values — over-reserving vs sharing a param's register
    // after it dies, but correct and simple (register sharing for dead params is a later refinement).
    let param_registers: HashMap<ValueId, usize> =
        entry_params.iter().enumerate().map(|(index, &param)| (param, index)).collect();
    if param_registers.len() > value_capacity {
        return None;
    }

    // Flatten to one interval per (value, range), scanned in start order.
    let mut intervals: Vec<RangeOf> = ranges
        .iter()
        .flat_map(|(value, list)| {
            list.iter().map(move |range| RangeOf { start: range.start, end: range.end, value })
        })
        .collect();
    intervals.sort_by_key(|r| (r.start, r.end, r.value));

    // Free register indices available to non-parameter values (parameter indices are reserved).
    let mut free: std::collections::BTreeSet<usize> =
        (param_registers.len()..value_capacity).collect();
    // Currently register-resident non-parameter ranges: (range end, register index).
    let mut active: Vec<(ProgramPoint, usize)> = Vec::new();
    // A value's most recent register, so a later range can reclaim it when free.
    let mut previous_register: HashMap<ValueId, usize> = HashMap::default();
    let mut timeline: HashMap<ValueId, Vec<Segment>> = HashMap::default();

    for RangeOf { start, end, value } in intervals {
        // Expire non-parameter ranges that ended strictly before this one begins, freeing their
        // registers. Ranges touching at a point (`prev.end == start`) stay active — they interfere.
        active.retain(|&(active_end, register)| {
            let expired = active_end < start;
            if expired {
                free.insert(register);
            }
            !expired
        });

        let register = if let Some(&index) = param_registers.get(&value) {
            // Fixed-interval parameter: always its reserved index, never drawn from `free`.
            index
        } else {
            // Prefer the value's previous register (reclaim across a hole), else the lowest free one.
            let register = match previous_register.get(&value) {
                Some(&prev) if free.remove(&prev) => prev,
                _ => *free.iter().next()?,
            };
            free.remove(&register);
            active.push((end, register));
            register
        };

        previous_register.insert(value, register);
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

/// The plan-based allocator: a **read-only** server over a precomputed [`Plan`].
///
/// It holds no register pool and takes no online decisions — every method is a lookup into the plan
/// (and the program-point maps in [`LiveIntervals`]) that returns where a value lives and the
/// [`Action`]s to realize it. This is the endgame the design doc describes: "the allocator's methods
/// become read-only queries over a precomputed plan".
///
/// **No-pressure scope.** This handles functions whose register pressure fits the value capacity
/// (`usable - min_live_count`); [`Self::try_build`] returns `None` otherwise so the caller falls
/// back to greedy. With no pressure every value keeps one register for its whole life, so there are
/// no spills or reloads — `begin_block` seeds the register-resident values, `after_instruction`
/// prunes the dead, and `resolve_edge` reports each parameter's register. Pressure spilling (splits,
/// reloads, resolution) is layered on next.
pub(crate) struct LinearScanAllocator {
    /// The precomputed assignment (value location timelines).
    plan: Plan,
    /// Program-point maps, so `begin_block` can find a block's entry point.
    intervals: LiveIntervals,
    /// Values dying after each instruction, so `after_instruction` prunes them.
    last_uses: HashMap<InstructionId, HashSet<ValueId>>,
    /// Each value's typed `BrilligVariable`, its register fixed by the plan. This is the final
    /// allocation map handed to the artifact via `into_allocations`.
    allocations: HashMap<ValueId, BrilligVariable>,
    /// The number of low registers the plan actually uses for value homes — the reserved band
    /// codegen keeps scratch above. Only the registers used (not the whole capacity) are reserved,
    /// so the frame's high-water mark stays minimal (important for recursion depth).
    reserved_registers: usize,
}

impl LinearScanAllocator {
    /// Build the plan and the read-only allocator, or `None` if the function does not fit
    /// `value_capacity` without pressure spilling (the caller then falls back to greedy).
    ///
    /// `value_capacity` is the number of low registers usable for value homes; the caller reserves
    /// the remaining upper registers for scratch (see [`FunctionContext`] — it sets the capacity to
    /// `usable - (min_live_count + SPILL_MARGIN)`, leaving comfortable room for instruction scratch,
    /// parallel-move temporaries, and on-demand constants that `min_live_count` under-counts).
    /// `register_offset` is the first usable register offset (`Stack::START_OFFSET`), used to map a
    /// plan register index to a frame-relative [`MemoryAddress`].
    ///
    /// [`FunctionContext`]: super::brillig_fn::FunctionContext
    pub(crate) fn try_build(
        function: &Function,
        liveness: &VariableLiveness,
        constants: &ConstantAllocation,
        post_order: &[BasicBlockId],
        last_uses: &HashMap<InstructionId, HashSet<ValueId>>,
        value_capacity: usize,
        register_offset: usize,
    ) -> Option<Self> {
        let intervals = LiveIntervals::from_function(function, liveness, constants, post_order);
        let ranges = LiveRanges::from_intervals(&intervals, liveness, post_order);
        let entry_params = function.dfg[function.entry_block()].parameters();
        let plan = assign(&ranges, value_capacity, entry_params)?;

        // Materialize each value's typed variable at its single planned register.
        //
        // The no-pressure allocator emits no spills, reloads, or moves, so it can only serve a plan
        // where every value lives in exactly one register for its whole life. Register scarcity can
        // still force a *split* even without slot spilling: when a value's register is reused during
        // a divergent-path hole by another value whose range overlaps the first value's revival,
        // reclaim fails and the value lands in a second register. Realizing a split needs a move on
        // the revival edge, which only the pressure-aware allocator (with resolution) emits — so if
        // any value's timeline is not a single register, decline and fall back to greedy.
        let mut allocations = HashMap::default();
        for (value, _) in ranges.iter() {
            let segments = plan.timeline(value);
            let Some(Location::Register(index)) = segments.first().map(|seg| seg.location) else {
                return None;
            };
            if segments.iter().any(|seg| seg.location != Location::Register(index)) {
                return None;
            }
            let register = MemoryAddress::relative(assert_u32(register_offset + index));
            allocations.insert(value, typed_variable(function, value, register));
        }

        // Reserve only the registers actually used, not the whole capacity: the highest value index
        // plus one. Scratch sits above this band. Keeping the reservation tight keeps the frame's
        // high-water mark minimal, which matters for recursion (each call frame is that size).
        let reserved_registers = allocations
            .values()
            .map(|variable| assert_usize(variable.extract_register().unwrap_relative()))
            .max()
            .map_or(0, |max_offset| max_offset - register_offset + 1);

        Some(Self {
            plan,
            intervals,
            last_uses: last_uses.clone(),
            allocations,
            reserved_registers,
        })
    }

    /// The number of low registers the plan actually uses for value homes. Codegen reserves exactly
    /// these, keeping scratch above them, so scratch never collides with a value home and the frame
    /// stays no larger than the values require.
    pub(crate) fn reserved_registers(&self) -> usize {
        self.reserved_registers
    }

    /// The final register allocations, consumed when handing the global allocations to the artifact.
    pub(crate) fn into_allocations(self) -> HashMap<ValueId, BrilligVariable> {
        self.allocations
    }

    /// The register a tracked value lives in (its home), panicking if it is untracked — every value
    /// the driver defines or uses is in the plan.
    fn register_of(&self, value: ValueId) -> MemoryAddress {
        self.allocations
            .get(&value)
            .unwrap_or_else(|| panic!("ICE: linear-scan has no allocation for {value}"))
            .extract_register()
    }
}

impl Allocator for LinearScanAllocator {
    fn begin_block(
        &mut self,
        block: BasicBlockId,
        _dfg: &DataFlowGraph,
    ) -> (Vec<(ValueId, MemoryAddress)>, Vec<Action>) {
        let entry = self.intervals.block_entry_point(block).expect("block has an entry point");
        // Seed the shadow with every value register-resident at this block's entry.
        let resident = self
            .allocations
            .iter()
            .filter(|&(&value, _)| {
                matches!(self.plan.location_at(value, entry), Some(Location::Register(_)))
            })
            .map(|(&value, variable)| (value, variable.extract_register()))
            .collect();
        (resident, Vec::new())
    }

    fn define_variable(
        &mut self,
        value_id: ValueId,
        _dfg: &DataFlowGraph,
    ) -> (BrilligVariable, Vec<Action>) {
        (self.allocations[&value_id], Vec::new())
    }

    fn use_variable(&mut self, value_id: ValueId) -> (BrilligVariable, Vec<Action>) {
        (self.allocations[&value_id], Vec::new())
    }

    fn reserve_scratch(&mut self, _scratch: usize) -> Vec<Action> {
        // The plan reserved `min_live_count` upper registers for scratch, so there is always room;
        // codegen allocates the temporaries from that band. Nothing to spill.
        Vec::new()
    }

    fn after_instruction(&mut self, inst: InstructionId) -> Vec<Action> {
        let Some(dead) = self.last_uses.get(&inst) else {
            return Vec::new();
        };
        dead.iter()
            .filter(|value| self.allocations.contains_key(value))
            .map(|&value| Action::Prune { value, register: self.register_of(value) })
            .collect()
    }

    fn before_terminator(&mut self, _block: BasicBlockId, _dfg: &DataFlowGraph) -> Vec<Action> {
        // No pressure means no cross-edge value is spilled, so nothing to store before the branch.
        Vec::new()
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
            .map(|param| ParamHome::Register(self.register_of(*param)))
            .collect()
    }

    fn spill_enabled(&self) -> bool {
        false
    }

    fn max_spill_offset(&self) -> usize {
        0
    }
}

/// Build a value's typed [`BrilligVariable`] at `register`, mirroring the greedy allocator's typed
/// allocation but with the register fixed by the plan rather than pulled from a pool.
fn typed_variable(function: &Function, value: ValueId, register: MemoryAddress) -> BrilligVariable {
    let typ = function.dfg.type_of_value(value);
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

/// The allocator a [`FunctionContext`](super::brillig_fn::FunctionContext) runs, chosen per function.
///
/// Both variants implement [`Allocator`], so the driver dispatches through this enum without caring
/// which strategy is active. Keeping the choice in an enum (rather than `Box<dyn Allocator>`) lets
/// the non-trait lifecycle methods (`into_allocations`, the test-only inspectors) stay off the
/// [`Allocator`] trait, which is deliberately free of allocator-specific surface.
// Exactly one instance exists per function, so the variant size gap does not matter; boxing would
// add indirection on the hot allocator methods for no benefit.
#[allow(clippy::large_enum_variant)]
pub(crate) enum FunctionAllocator<R: RegisterAllocator> {
    Greedy(GreedyAllocator<R>),
    LinearScan(LinearScanAllocator),
}

impl<R: RegisterAllocator> FunctionAllocator<R> {
    /// The final register allocations, consumed when handing the global allocations to the artifact.
    pub(crate) fn into_allocations(self) -> HashMap<ValueId, BrilligVariable> {
        match self {
            Self::Greedy(a) => a.into_allocations(),
            Self::LinearScan(a) => a.into_allocations(),
        }
    }

    /// Greedy-only test inspectors. The tests that use them build greedy contexts (the default), so
    /// the linear-scan arms are unreachable.
    #[cfg(test)]
    pub(crate) fn get_coalesced(&self, value_id: &ValueId) -> Option<ValueId> {
        match self {
            Self::Greedy(a) => a.get_coalesced(value_id),
            Self::LinearScan(_) => panic!("get_coalesced is greedy-only"),
        }
    }

    #[cfg(test)]
    pub(crate) fn liveness(&self) -> &VariableLiveness {
        match self {
            Self::Greedy(a) => a.liveness(),
            Self::LinearScan(_) => panic!("liveness is greedy-only"),
        }
    }

    #[cfg(test)]
    pub(crate) fn retire(&mut self, value_id: &ValueId) {
        match self {
            Self::Greedy(a) => a.retire(value_id),
            Self::LinearScan(_) => panic!("retire is greedy-only"),
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

    fn entry_params(ssa: &Ssa) -> Vec<ValueId> {
        let func = ssa.main();
        func.dfg[func.entry_block()].parameters().to_vec()
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
        let (ranges, ssa) = ranges_for(src);
        let plan = assign(&ranges, 8, &entry_params(&ssa)).expect("fits in 8 registers");
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
        let (ranges, ssa) = ranges_for(src);
        assert!(
            assign(&ranges, 2, &entry_params(&ssa)).is_none(),
            "capacity 2 should be unplaceable without spilling"
        );
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
        let plan = assign(&ranges, 8, &entry_params(&ssa)).expect("fits in 8 registers");
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
