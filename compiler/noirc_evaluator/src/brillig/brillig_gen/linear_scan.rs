//! Linear-scan register allocation for Brillig (Phase 1 of the plan in
//! `design/register_allocation.md`).
//!
//! Where [`GreedyAllocator`] makes spill/placement decisions *online* (LRU eviction as it runs),
//! the linear-scan allocator computes a global assignment *up front* from the value [`LiveRanges`]
//! and then serves it as read-only queries. Both implement the same [`Allocator`] seam, so the
//! driver ([`BrilligBlock`]) is identical for either; which one runs is chosen per function by
//! [`FunctionAllocator`].
//!
//! **Pipeline.** [`assign`] runs linear scan over the hole-aware [`LiveRanges`] and produces a
//! [`Plan`] — each value's location timeline. [`LinearScanAllocator`] is then a pure read-only server
//! over that plan (no register pool, no online decisions): every trait method is a lookup returning
//! where a value lives and the [`Action`]s to realize it.
//!
//! **Scope.** Functions whose register pressure fits the value capacity are served — including those
//! that need pressure spilling (a value split register→slot→register, reloaded at its next use) and
//! interval-splitting across holes. [`LinearScanAllocator::try_build`] declines (falls back to
//! [`GreedyAllocator`]) only when [`assign`] cannot place the function at the capacity — e.g. a
//! high-arity `MakeArray` whose elements the single-point plan counts as simultaneously live (the
//! per-element sub-points that would stream them are a planned refinement).
//!
//! [`BrilligBlock`]: super::brillig_block::BrilligBlock
//! [`FunctionContext`]: super::brillig_fn::FunctionContext

use acvm::acir::brillig::MemoryAddress;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::allocator::{Action, Allocator, GreedyAllocator, ParamHome, SpillSlot};
use super::brillig_block_variables::compute_array_length;
use super::constant_allocation::ConstantAllocation;
use super::live_intervals::{LiveIntervals, LiveRanges, ProgramPoint};
use super::variable_liveness::VariableLiveness;
use crate::brillig::assert_u32;
use crate::brillig::brillig_ir::brillig_variable::{
    BrilligArray, BrilligVariable, BrilligVector, SingleAddrVariable, get_bit_size_from_ssa_type,
};
use crate::brillig::brillig_ir::registers::RegisterAllocator;
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
    /// Per block, the values register-resident at its entry point → their register index. Stored as
    /// `im` maps so the snapshots share structure across blocks; `begin_block` seeds the residency
    /// mirror from its block's snapshot in O(live-at-entry), rather than scanning every value's
    /// timeline (which is O(values) per block = quadratic over a whole function).
    entry_registers: HashMap<BasicBlockId, im::HashMap<ValueId, usize>>,
    num_spill_slots: usize,
}

#[allow(dead_code)]
impl Plan {
    /// The value's location timeline (segments in point order), or empty if the value is untracked.
    pub(crate) fn timeline(&self, value: ValueId) -> &[Segment] {
        self.timeline.get(&value).map_or(&[], Vec::as_slice)
    }

    /// The values register-resident at `block`'s entry, each mapped to its register index. Cheap to
    /// clone (structural sharing).
    pub(crate) fn registers_at_block_entry(
        &self,
        block: BasicBlockId,
    ) -> im::HashMap<ValueId, usize> {
        self.entry_registers.get(&block).cloned().unwrap_or_default()
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

/// The use points of every value in increasing order: the program points where the value is an
/// operand of an instruction or terminator. A spilled value must be reloaded to a register at each
/// of these, and the furthest next use drives victim selection.
pub(crate) fn compute_use_points(
    function: &Function,
    intervals: &LiveIntervals,
    post_order: &[BasicBlockId],
) -> HashMap<ValueId, Vec<ProgramPoint>> {
    let mut uses: HashMap<ValueId, Vec<ProgramPoint>> = HashMap::default();
    for &block in post_order {
        for &inst in function.dfg[block].instructions() {
            let point = intervals.instruction_point(inst).expect("instruction has a program point");
            function.dfg[inst].for_each_value(|value| {
                if super::variable_liveness::is_variable(value, &function.dfg) {
                    uses.entry(value).or_default().push(point);
                }
            });
        }
        let point = intervals.terminator_point(block).expect("block has a terminator point");
        function.dfg[block].unwrap_terminator().for_each_value(|value| {
            if super::variable_liveness::is_variable(value, &function.dfg) {
                uses.entry(value).or_default().push(point);
            }
        });
    }
    for points in uses.values_mut() {
        points.sort_unstable();
        points.dedup();
    }
    uses
}

/// The program points, increasing, at which the scan acts: where a live range begins or a value is
/// used. Range *ends* are handled lazily (a value's register frees at the first event past its
/// current range's end), so they need not be events themselves.
fn event_points(
    ranges: &LiveRanges,
    use_points: &HashMap<ValueId, Vec<ProgramPoint>>,
) -> Vec<ProgramPoint> {
    let mut points: Vec<ProgramPoint> = ranges
        .iter()
        .flat_map(|(_, list)| list.iter().map(|range| range.start))
        .chain(use_points.values().flatten().copied())
        .collect();
    points.sort_unstable();
    points.dedup();
    points
}

/// Mutable state of the linear scan, evolving as it walks program points. All decisions are made
/// here; the result is the read-only [`Plan`] (`timeline`).
struct Scan<'a> {
    param_registers: &'a HashMap<ValueId, usize>,
    use_points: &'a HashMap<ValueId, Vec<ProgramPoint>>,
    /// Register indices currently free for non-parameter values.
    free: std::collections::BTreeSet<usize>,
    /// Currently register-resident values → their register index.
    register_of: HashMap<ValueId, usize>,
    /// Each currently-live value → the end of the range it is in (for lazy expiry).
    range_end_of: HashMap<ValueId, ProgramPoint>,
    /// A value's most recent register, so a later range reclaims it when free (no spurious split).
    previous_register: HashMap<ValueId, usize>,
    /// Each live value's open segment: (start point, location).
    open: HashMap<ValueId, (ProgramPoint, Location)>,
    /// The accumulated per-value location timeline (the plan being built).
    timeline: HashMap<ValueId, Vec<Segment>>,
    /// The fixed spill slot of each value that is ever spilled (one per value; packing is a later
    /// refinement).
    slot_of: HashMap<ValueId, usize>,
    num_spill_slots: usize,
}

impl Scan<'_> {
    /// The first use of `value` strictly after `point`, if any.
    fn next_use(&self, value: ValueId, point: ProgramPoint) -> Option<ProgramPoint> {
        let points = self.use_points.get(&value)?;
        points.iter().copied().find(|&p| p > point)
    }

    /// The value's fixed spill slot, allocating one on first spill.
    fn slot(&mut self, value: ValueId) -> usize {
        if let Some(&slot) = self.slot_of.get(&value) {
            return slot;
        }
        let slot = self.num_spill_slots;
        self.num_spill_slots += 1;
        self.slot_of.insert(value, slot);
        slot
    }

    /// Close `value`'s open segment ending inclusively at `end`.
    fn close_segment(&mut self, value: ValueId, end: ProgramPoint) {
        if let Some((start, location)) = self.open.remove(&value) {
            self.timeline.entry(value).or_default().push(Segment { start, end, location });
        }
    }

    /// Spill a register-resident value to its slot at `point`: end its register segment just before
    /// `point`, free the register, and open a slot segment. (Never called on a parameter.)
    fn spill(&mut self, value: ValueId, point: ProgramPoint) -> usize {
        let (start, location) =
            self.open.remove(&value).expect("resident value has an open segment");
        let Location::Register(register) = location else {
            unreachable!("only register-resident values are spilled");
        };
        self.timeline.entry(value).or_default().push(Segment {
            start,
            end: point.pred(),
            location,
        });
        self.register_of.remove(&value);
        self.free.insert(register);
        let slot = self.slot(value);
        self.open.insert(value, (point, Location::Slot(slot)));
        register
    }

    /// Expire currently-live values whose range has ended, closing each one's open segment at its
    /// range end and returning its (non-parameter) register to the free set. With `strict`, only
    /// ranges ending *before* `point` expire — values still needed at `point` stay resident. Without
    /// it, ranges ending *at* `point` also expire; the scan runs this only *after* this point's
    /// result has claimed a register, so an operand consumed here keeps its register through the
    /// result's allocation and the result never aliases it.
    fn expire(&mut self, point: ProgramPoint, strict: bool) {
        let param_registers = self.param_registers;
        let expired: Vec<ValueId> = self
            .range_end_of
            .iter()
            .filter(|&(_, &end)| if strict { end < point } else { end <= point })
            .map(|(&value, _)| value)
            .collect();
        for value in expired {
            let end = self.range_end_of.remove(&value).expect("expiring value has a range end");
            let register = self.register_of.remove(&value);
            self.close_segment(value, end);
            if let Some(register) = register
                && !param_registers.contains_key(&value)
            {
                self.free.insert(register);
            }
        }
    }

    /// Allocate a register for `value` at `point`: reclaim its previous register if free, else the
    /// lowest free one, else evict the resident whose next use is furthest (spilling it). Values in
    /// `protected` (used at this very point, so needed now) and parameters are never evicted. Returns
    /// `None` only if no register can be obtained.
    fn allocate(
        &mut self,
        value: ValueId,
        point: ProgramPoint,
        protected: &HashSet<ValueId>,
    ) -> Option<usize> {
        if let Some(&previous) = self.previous_register.get(&value)
            && self.free.remove(&previous)
        {
            return Some(previous);
        }
        if let Some(&register) = self.free.iter().next() {
            self.free.remove(&register);
            return Some(register);
        }
        // Evict the evictable resident with the furthest next use (a value with no further use sorts
        // furthest, so it is evicted first).
        let victim = self
            .register_of
            .keys()
            .copied()
            .filter(|resident| {
                !self.param_registers.contains_key(resident) && !protected.contains(resident)
            })
            .max_by_key(|&resident| {
                let next = self.next_use(resident, point);
                (next.is_none(), next, resident)
            })?;
        let register = self.spill(victim, point);
        self.free.remove(&register);
        Some(register)
    }
}

/// Assign registers by linear scan over the hole-aware [`LiveRanges`], reloading spilled values at
/// their uses (Wimmer-style interval splitting), with a value-register capacity of `value_capacity`.
///
/// Walking program points in order: a value becomes resident when its live range begins (reclaiming
/// its previous register across a hole when free — no spurious split); when a value is used while
/// spilled it is reloaded into a register; and under pressure the resident whose next use is furthest
/// is evicted to its fixed spill slot. A value's register may therefore change over its life (a
/// register→slot→register split), which edge resolution reconciles at merges. Returns `None` only
/// when a value cannot be given any register (e.g. the entry parameters already fill the capacity).
#[allow(dead_code)]
pub(crate) fn assign(
    ranges: &LiveRanges,
    use_points: &HashMap<ValueId, Vec<ProgramPoint>>,
    value_capacity: usize,
    entry_params: &[ValueId],
) -> Option<Plan> {
    // Entry-block parameters are pre-colored fixed intervals: the calling convention places argument
    // `i` in register `i` (see `codegen_entry_point::allocate_function_arguments`), so the callee
    // must read parameter `i` from register `i`. Reserve `[0, n_params)` for them, never handing
    // those indices to other values.
    let param_registers: HashMap<ValueId, usize> =
        entry_params.iter().enumerate().map(|(index, &param)| (param, index)).collect();
    if param_registers.len() > value_capacity {
        return None;
    }

    // Per-point range starts (value → that range's end) and the values used at each point.
    let mut range_start_at: HashMap<ProgramPoint, Vec<(ValueId, ProgramPoint)>> =
        HashMap::default();
    for (value, list) in ranges.iter() {
        for range in list {
            range_start_at.entry(range.start).or_default().push((value, range.end));
        }
    }
    let mut used_at: HashMap<ProgramPoint, HashSet<ValueId>> = HashMap::default();
    for (&value, points) in use_points {
        for &point in points {
            used_at.entry(point).or_default().insert(value);
        }
    }

    let mut scan = Scan {
        param_registers: &param_registers,
        use_points,
        free: (param_registers.len()..value_capacity).collect(),
        register_of: HashMap::default(),
        range_end_of: HashMap::default(),
        previous_register: HashMap::default(),
        open: HashMap::default(),
        timeline: HashMap::default(),
        slot_of: HashMap::default(),
        num_spill_slots: 0,
    };
    let empty = HashSet::default();

    for point in event_points(ranges, use_points) {
        let protected = used_at.get(&point).unwrap_or(&empty);

        // 1. Expire values whose current range ended strictly before this point, freeing their
        //    registers. Values still used at this point stay resident.
        scan.expire(point, true);

        // A value whose live range *starts* at this point is defined here (an instruction result, or
        // a constant materialized at its use — its def is the materialization point). It is handled
        // by the range-starts step below, not reloaded as if it were a spilled operand; reloading it
        // here would allocate it twice and leak its first register out of the free set.
        let starts_here: HashSet<ValueId> = range_start_at
            .get(&point)
            .map(|starts| starts.iter().map(|(value, _)| *value).collect())
            .unwrap_or_default();

        // 2. Uses here: a value used at this instruction must be register-resident. A currently
        //    spilled operand is reloaded — its slot segment ends just before the use and it takes a
        //    register.
        if let Some(users) = used_at.get(&point) {
            let mut users: Vec<ValueId> = users.iter().copied().collect();
            users.sort_unstable();
            for value in users {
                if scan.register_of.contains_key(&value)
                    || param_registers.contains_key(&value)
                    || starts_here.contains(&value)
                {
                    continue;
                }
                scan.close_segment(value, point.pred());
                let register = scan.allocate(value, point, protected)?;
                scan.register_of.insert(value, register);
                scan.previous_register.insert(value, register);
                scan.open.insert(value, (point, Location::Register(register)));
            }
        }

        // 3. Ranges starting here: the value becomes live and is placed in a register. Parameters
        //    take their fixed register. A result is placed *before* its instruction's dying operands
        //    are freed (step 4), so it never reuses an operand's register — codegen claims the result
        //    register before reading the operands, so the two must not alias. Operands used at this
        //    point are `protected`, so the result never evicts one either.
        if let Some(starts) = range_start_at.get(&point) {
            let mut starts = starts.clone();
            starts.sort_unstable_by_key(|(value, _)| *value);
            for (value, range_end) in starts {
                scan.range_end_of.insert(value, range_end);
                if let Some(&index) = param_registers.get(&value) {
                    scan.register_of.insert(value, index);
                    scan.previous_register.insert(value, index);
                    scan.open.insert(value, (point, Location::Register(index)));
                    continue;
                }
                let register = scan.allocate(value, point, protected)?;
                scan.register_of.insert(value, register);
                scan.previous_register.insert(value, register);
                scan.open.insert(value, (point, Location::Register(register)));
            }
        }

        // 4. Expire operands consumed exactly at this point, freeing their registers — but only now
        //    that this point's result has already claimed a distinct register.
        scan.expire(point, false);
    }

    // Close every still-open segment at its range end.
    let open_values: Vec<ValueId> = scan.open.keys().copied().collect();
    for value in open_values {
        let end = scan.range_end_of.get(&value).copied().unwrap_or_else(|| scan.open[&value].0);
        scan.close_segment(value, end);
    }

    for segments in scan.timeline.values_mut() {
        segments.sort_by_key(|segment| segment.start);
    }

    Some(Plan {
        timeline: scan.timeline,
        entry_registers: HashMap::default(),
        num_spill_slots: scan.num_spill_slots,
    })
}

/// Compute, per block, the values register-resident at its entry point (value → register index), as
/// structurally-shared `im` maps. A single sweep over all register segments: a value enters the live
/// set at a segment's start and leaves at its end + 1; the snapshot at a block-entry point is the
/// live set there. This precomputes what `begin_block` would otherwise re-derive by scanning every
/// value's timeline on each block.
fn block_entry_registers(
    timeline: &HashMap<ValueId, Vec<Segment>>,
    intervals: &LiveIntervals,
    post_order: &[BasicBlockId],
) -> HashMap<BasicBlockId, im::HashMap<ValueId, usize>> {
    use std::collections::BTreeMap;
    // Register-segment enter/exit events, and the block-entry points to snapshot, keyed by point.
    let mut enters: BTreeMap<ProgramPoint, Vec<(ValueId, usize)>> = BTreeMap::new();
    let mut exits: BTreeMap<ProgramPoint, Vec<ValueId>> = BTreeMap::new();
    for (&value, segments) in timeline {
        for segment in segments {
            if let Location::Register(index) = segment.location {
                enters.entry(segment.start).or_default().push((value, index));
                exits.entry(segment.end).or_default().push(value);
            }
        }
    }
    let mut entries_at: BTreeMap<ProgramPoint, Vec<BasicBlockId>> = BTreeMap::new();
    for &block in post_order {
        if let Some(point) = intervals.block_entry_point(block) {
            entries_at.entry(point).or_default().push(block);
        }
    }

    let mut points: Vec<ProgramPoint> =
        enters.keys().chain(exits.keys()).chain(entries_at.keys()).copied().collect();
    points.sort_unstable();
    points.dedup();

    // A segment `[start, end]` covers a point `p` iff `start <= p <= end`. Processing per point in
    // the order enters → snapshot → exits keeps a value live for the snapshots at exactly its
    // `[start, end]`: it is inserted before the snapshot at `start` and removed only after the
    // snapshot at `end`.
    let mut live: im::HashMap<ValueId, usize> = im::HashMap::new();
    let mut result: HashMap<BasicBlockId, im::HashMap<ValueId, usize>> = HashMap::default();
    for point in points {
        if let Some(started) = enters.get(&point) {
            for (value, index) in started {
                live.insert(*value, *index);
            }
        }
        if let Some(blocks) = entries_at.get(&point) {
            for &block in blocks {
                result.insert(block, live.clone());
            }
        }
        if let Some(ended) = exits.get(&point) {
            for value in ended {
                live.remove(value);
            }
        }
    }
    result
}

/// The plan-based allocator: a server that *realizes* a precomputed [`Plan`] at codegen time.
///
/// It makes no online allocation decisions — [`assign`] already fixed where every value lives at
/// every program point. What this does is keep a small **residency mirror** (which value occupies
/// which register right now) and, at each definition and use, emit the [`Action`]s that move the
/// register file from its current state to what the plan requires at that point: an [`Action::Reload`]
/// to bring a spilled value back, an [`Action::Move`] when the plan splits a value into a different
/// register, and an [`Action::Spill`] to evict whatever currently holds a register the plan is about
/// to reuse. Residency is reset from the plan at each block entry, so a value's register is always a
/// lookup, never a decision.
///
/// **No move cycles.** A register a value needs may be held by another value; since [`assign`]
/// always evicts a live value to *its slot* (never to another register), the incumbent is simply
/// spilled first — there is never a register-to-register displacement chain to untangle. A result
/// never shares a register with its instruction's operands (codegen claims the result register
/// before reading the operands), so defining a result never displaces a value about to be read.
pub(crate) struct LinearScanAllocator {
    /// The precomputed assignment: each value's location timeline.
    plan: Plan,
    /// Program-point maps, so a block entry / instruction / terminator resolves to its point.
    intervals: LiveIntervals,
    /// Values dying after each instruction, so `after_instruction` prunes them.
    last_uses: HashMap<InstructionId, HashSet<ValueId>>,
    /// Each value's typed [`BrilligVariable`] at its *definition* register. The shape (bit size,
    /// array length) is fixed; only the register varies, so a value is served at its current register
    /// via [`BrilligVariable::with_register`]. Also the map handed to the artifact.
    templates: HashMap<ValueId, BrilligVariable>,
    /// Each value that is ever spilled → its fixed spill slot.
    value_slot: HashMap<ValueId, usize>,
    /// The first usable register offset; plan index `i` is the relative address `register_offset + i`.
    register_offset: usize,
    /// Low registers reserved for value homes (highest plan register index + 1); scratch sits above.
    /// Only the registers used (not the whole capacity) are reserved, so the frame's high-water mark
    /// stays minimal (important for recursion depth).
    reserved_registers: usize,
    /// Number of spill slots the plan uses — the spill prologue's high-water mark (0 if none).
    num_spill_slots: usize,
    /// Residency mirror, reset from the plan at each block entry and evolved as actions are emitted:
    /// which register index holds each live value, and the inverse.
    register_of_value: HashMap<ValueId, usize>,
    value_in_register: HashMap<usize, ValueId>,
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
        let use_points = compute_use_points(function, &intervals, post_order);
        let entry_params = function.dfg[function.entry_block()].parameters();
        let mut plan = assign(&ranges, &use_points, value_capacity, entry_params)?;
        plan.entry_registers = block_entry_registers(&plan.timeline, &intervals, post_order);

        // Scope: serve register-only plans (a value may change register across a hole; its data
        // survives on the taken path, and `make_resident` re-seeds it). Decline plans that use a
        // spill slot: pressure spilling needs the slot-canonical merge resolution (a value live on
        // both branches of a merge but spilled on one), which is not yet implemented. Until then,
        // fall back to greedy for those. Keeping register-resident values (including cross-block
        // ones) in a register already beats greedy, which permanently spills every cross-block value.
        if plan.num_spill_slots() > 0 {
            return None;
        }

        // Precompute, from the plan: each value's typed template at its definition register, each
        // spilled value's fixed slot, and the highest register index used (which fixes the reserved
        // value-home band). The template's register is the value's first register segment; codegen
        // serves the value at its current register by rewriting that address via `with_register`.
        let mut templates = HashMap::default();
        let mut value_slot = HashMap::default();
        let mut max_register_index: Option<usize> = None;
        for (value, _) in ranges.iter() {
            let segments = plan.timeline(value);
            let def_index = segments.iter().find_map(|seg| match seg.location {
                Location::Register(index) => Some(index),
                Location::Slot(_) => None,
            });
            // Every tracked value is defined into a register before any spill, so a timeline with no
            // register segment is malformed; decline rather than serve it.
            let def_index = def_index?;
            let register = MemoryAddress::relative(assert_u32(register_offset + def_index));
            templates.insert(value, typed_variable(function, value, register));
            for seg in segments {
                match seg.location {
                    Location::Register(index) => {
                        max_register_index =
                            Some(max_register_index.map_or(index, |current| current.max(index)));
                    }
                    Location::Slot(slot) => {
                        value_slot.insert(value, slot);
                    }
                }
            }
        }
        let reserved_registers = max_register_index.map_or(0, |index| index + 1);
        let num_spill_slots = plan.num_spill_slots();

        Some(Self {
            plan,
            intervals,
            last_uses: last_uses.clone(),
            templates,
            value_slot,
            register_offset,
            reserved_registers,
            num_spill_slots,
            register_of_value: HashMap::default(),
            value_in_register: HashMap::default(),
        })
    }

    /// The number of low registers the plan actually uses for value homes. Codegen reserves exactly
    /// these, keeping scratch above them, so scratch never collides with a value home and the frame
    /// stays no larger than the values require.
    pub(crate) fn reserved_registers(&self) -> usize {
        self.reserved_registers
    }

    /// The register allocations handed to the artifact: each value at its definition register. A
    /// split value has several registers over its life; its definition register is the representative
    /// the artifact metadata records.
    pub(crate) fn into_allocations(self) -> HashMap<ValueId, BrilligVariable> {
        self.templates
    }

    /// The relative memory address of plan register `index`.
    fn addr(&self, index: usize) -> MemoryAddress {
        MemoryAddress::relative(assert_u32(self.register_offset + index))
    }

    /// The typed variable for `value` served at register `index` (its fixed shape, this register).
    fn var_at(&self, value: ValueId, index: usize) -> BrilligVariable {
        self.templates[&value].with_register(self.addr(index))
    }

    /// The register index the plan places `value` in at `point`, panicking if the plan does not put
    /// it in a register there — definitions and uses are always register-homed points.
    fn register_index_at(&self, value: ValueId, point: ProgramPoint) -> usize {
        match self.plan.location_at(value, point) {
            Some(Location::Register(index)) => index,
            other => panic!(
                "ICE: linear-scan expected {value} in a register at {point:?}, got {other:?}"
            ),
        }
    }

    /// Record that `value` now occupies register `index`, updating both sides of the residency
    /// mirror and clearing any stale inverse entry for the register `value` used to hold.
    fn occupy(&mut self, value: ValueId, index: usize) {
        if let Some(previous) = self.register_of_value.insert(value, index) {
            self.value_in_register.remove(&previous);
        }
        self.value_in_register.insert(index, value);
    }

    /// Free register `index` for an incoming value at `point`, emitting the action to preserve its
    /// current occupant. The plan reuses a register only by evicting its occupant to that occupant's
    /// slot, so the occupant is spilled (a real store). The fallback [`Action::Prune`] covers only a
    /// value the plan no longer homes here at all — bookkeeping so the driver's shadow stays exact.
    /// Clears the occupant from the mirror either way.
    fn free_register(&mut self, index: usize, point: ProgramPoint, actions: &mut Vec<Action>) {
        let Some(occupant) = self.value_in_register.get(&index).copied() else {
            return;
        };
        match self.plan.location_at(occupant, point) {
            Some(Location::Slot(slot)) => {
                actions.push(Action::Spill {
                    value: occupant,
                    from: self.addr(index),
                    to: SpillSlot(slot),
                });
            }
            _ => {
                actions.push(Action::Prune { value: occupant, register: self.addr(index) });
            }
        }
        self.register_of_value.remove(&occupant);
        self.value_in_register.remove(&index);
    }

    /// Make `value` register-resident in register `index` for a use at `point`, returning the
    /// actions to get it there: nothing if it is already there, otherwise free the target and bring
    /// `value` in — a [`Action::Move`] from its current register, a [`Action::Reload`] from its slot
    /// if it is currently spilled, or (for a value whose register never changes) nothing but a
    /// re-seed of the residency mirror.
    fn make_resident(&mut self, value: ValueId, index: usize, point: ProgramPoint) -> Vec<Action> {
        if self.register_of_value.get(&value) == Some(&index) {
            return Vec::new();
        }
        let mut actions = Vec::new();
        self.free_register(index, point, &mut actions);
        match self.register_of_value.get(&value).copied() {
            Some(current) => {
                actions.push(Action::Move {
                    value,
                    from: self.addr(current),
                    to: self.addr(index),
                });
            }
            None => {
                // A currently-spilled value is reloaded from its slot. A value that is neither
                // mirrored nor spilled is one whose plan register never changes, revived after a
                // liveness hole: its data still sits in `index` on the path that reaches this use
                // (the hole is a divergent branch that did not execute, so nothing overwrote the
                // register there), so re-establishing the mirror entry below is enough — the pruning
                // that dropped it happened on a branch this path did not take. (A value whose
                // register *does* change across a hole cannot be served this way; `try_build`
                // declines such plans.)
                if let Some(slot) = self.value_slot.get(&value).copied() {
                    actions.push(Action::Reload {
                        value,
                        from: SpillSlot(slot),
                        into: self.addr(index),
                    });
                }
            }
        }
        self.occupy(value, index);
        actions
    }
}

impl Allocator for LinearScanAllocator {
    fn begin_block(
        &mut self,
        block: BasicBlockId,
        _dfg: &DataFlowGraph,
    ) -> (Vec<(ValueId, MemoryAddress)>, Vec<Action>) {
        // Reset the residency mirror to the plan's picture at this block's entry: the values the plan
        // homes in a register there, read directly from the precomputed snapshot. The physical
        // registers hold exactly these on entry — the plan is globally consistent and each incoming
        // edge leaves cross-edge values where the successor expects them — so this is a lookup, not a
        // decision.
        let snapshot = self.plan.registers_at_block_entry(block);
        self.register_of_value.clear();
        self.value_in_register.clear();
        let mut resident = Vec::with_capacity(snapshot.len());
        for (&value, &index) in &snapshot {
            self.register_of_value.insert(value, index);
            self.value_in_register.insert(index, value);
            resident.push((value, self.addr(index)));
        }
        (resident, Vec::new())
    }

    fn define_variable(
        &mut self,
        value_id: ValueId,
        _dfg: &DataFlowGraph,
    ) -> (BrilligVariable, Vec<Action>) {
        // The result is defined into its planned register at its definition point. Free that
        // register first (spilling a live incumbent the plan evicts here, or pruning a dying operand
        // whose register is handed off); the driver then writes the value into it. There is no
        // reload/move for the value itself — it is freshly computed.
        let def_point =
            self.plan.timeline(value_id).first().expect("defined value has a segment").start;
        let index = self.register_index_at(value_id, def_point);
        let mut actions = Vec::new();
        self.free_register(index, def_point, &mut actions);
        self.occupy(value_id, index);
        (self.var_at(value_id, index), actions)
    }

    fn use_variable(
        &mut self,
        value_id: ValueId,
        at: Option<InstructionId>,
    ) -> (BrilligVariable, Vec<Action>) {
        match at {
            Some(inst) => {
                let point =
                    self.intervals.instruction_point(inst).expect("instruction has a point");
                let index = self.register_index_at(value_id, point);
                let actions = self.make_resident(value_id, index, point);
                (self.var_at(value_id, index), actions)
            }
            None => {
                // Terminator operand: `before_terminator` made every terminator operand resident up
                // front, so this is a plain lookup of the register it currently occupies.
                let index = *self.register_of_value.get(&value_id).unwrap_or_else(|| {
                    panic!("ICE: terminator operand {value_id} is not resident")
                });
                (self.var_at(value_id, index), Vec::new())
            }
        }
    }

    fn reserve_scratch(&mut self, _scratch: usize, _at: Option<InstructionId>) -> Vec<Action> {
        // Nothing to do: `value_capacity` was sized as `usable - (min_live_count + SPILL_MARGIN)`,
        // and `min_live_count` already folds in each instruction's `instruction_scratch_demand`
        // (see `VariableLiveness`). So the registers above the reserved value band are provably free
        // for codegen's scratch at every point — the reservation is made once at capacity time,
        // rather than per instruction here.
        Vec::new()
    }

    fn after_instruction(&mut self, inst: InstructionId) -> Vec<Action> {
        let Some(dead) = self.last_uses.get(&inst) else {
            return Vec::new();
        };
        let dead: Vec<ValueId> =
            dead.iter().copied().filter(|value| self.templates.contains_key(value)).collect();
        let mut actions = Vec::new();
        for value in dead {
            // A register-resident dead value is dropped from the mirror, freeing its register (a
            // dying value already handed off at a define was pruned there, so it is no longer in the
            // mirror). A dead value that is currently spilled holds no register — nothing to prune.
            if let Some(index) = self.register_of_value.remove(&value) {
                self.value_in_register.remove(&index);
                actions.push(Action::Prune { value, register: self.addr(index) });
            }
        }
        actions
    }

    fn before_terminator(&mut self, block: BasicBlockId, dfg: &DataFlowGraph) -> Vec<Action> {
        // Make every terminator operand resident in its planned register at the terminator point,
        // before the operands are read. Terminator operands are bounded (condition, return values,
        // jmp arguments), so — unlike a streaming `MakeArray` — they can all be made resident here.
        let point = self.intervals.terminator_point(block).expect("block has a terminator point");
        let mut operands: Vec<ValueId> = Vec::new();
        dfg[block].unwrap_terminator().for_each_value(|value| {
            if super::variable_liveness::is_variable(value, dfg)
                && self.templates.contains_key(&value)
            {
                operands.push(value);
            }
        });
        operands.sort_unstable();
        operands.dedup();
        let mut actions = Vec::new();
        for value in operands {
            let index = self.register_index_at(value, point);
            actions.extend(self.make_resident(value, index, point));
        }
        actions
    }

    fn resolve_edge(
        &self,
        _pred: BasicBlockId,
        succ: BasicBlockId,
        dfg: &DataFlowGraph,
    ) -> Vec<ParamHome> {
        // Each successor parameter lives where the plan places it at the successor's entry point.
        let entry = self.intervals.block_entry_point(succ).expect("block has an entry point");
        dfg[succ]
            .parameters()
            .iter()
            .map(|&param| match self.plan.location_at(param, entry) {
                Some(Location::Register(index)) => ParamHome::Register(self.addr(index)),
                Some(Location::Slot(slot)) => ParamHome::Slot(SpillSlot(slot)),
                // The parameter is dead at its own block's entry: it is passed by a predecessor but
                // never read in the block (e.g. only forwarded as a jmp argument earlier). The jmp
                // still has to write it somewhere valid, so route it to the parameter's home (its
                // definition register). The write is dead but harmless.
                None => ParamHome::Register(self.templates[&param].extract_register()),
            })
            .collect()
    }

    fn spill_enabled(&self) -> bool {
        self.num_spill_slots > 0
    }

    fn max_spill_offset(&self) -> usize {
        self.num_spill_slots
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

    fn use_variable(
        &mut self,
        value_id: ValueId,
        at: Option<InstructionId>,
    ) -> (BrilligVariable, Vec<Action>) {
        match self {
            Self::Greedy(a) => a.use_variable(value_id, at),
            Self::LinearScan(a) => a.use_variable(value_id, at),
        }
    }

    fn reserve_scratch(&mut self, scratch: usize, at: Option<InstructionId>) -> Vec<Action> {
        match self {
            Self::Greedy(a) => a.reserve_scratch(scratch, at),
            Self::LinearScan(a) => a.reserve_scratch(scratch, at),
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
    use super::{Location, Plan, Segment, assign, compute_use_points};
    use crate::brillig::brillig_gen::constant_allocation::ConstantAllocation;
    use crate::brillig::brillig_gen::live_intervals::{LiveIntervals, LiveRanges, ProgramPoint};
    use crate::brillig::brillig_gen::variable_liveness::VariableLiveness;
    use crate::ssa::ir::post_order::PostOrder;
    use crate::ssa::ir::value::ValueId;
    use crate::ssa::ssa_gen::Ssa;
    use rustc_hash::FxHashMap as HashMap;

    fn ranges_for(src: &str) -> (LiveRanges, HashMap<ValueId, Vec<ProgramPoint>>, Ssa) {
        let ssa = Ssa::from_str(src).unwrap();
        let func = ssa.main();
        let constants = ConstantAllocation::from_function(func);
        let liveness = VariableLiveness::from_function(func, &constants);
        let post_order = PostOrder::with_function(func).into_vec();
        let intervals = LiveIntervals::from_function(func, &liveness, &constants, &post_order);
        let ranges = LiveRanges::from_intervals(&intervals, &liveness, &post_order);
        let use_points = compute_use_points(func, &intervals, &post_order);
        (ranges, use_points, ssa)
    }

    /// Inclusive overlap: two segments conflict if they share any point. A result defined at `p` and
    /// an operand last-used at `p` both need a register at `p` — codegen claims the result register
    /// before reading the operands, so they must not alias — hence sharing the boundary point `p` is
    /// a genuine conflict, and `assign` gives them distinct registers.
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
        let (ranges, use_points, ssa) = ranges_for(src);
        let plan =
            assign(&ranges, &use_points, 8, &entry_params(&ssa)).expect("fits in 8 registers");
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
    fn pressure_spills_soundly() {
        // `v2` and `v3` are defined up front and used by both `v4` and `v5`, so at the `mul v2, v3`
        // point four non-parameter values are live at once: the operands `v2`, `v3`, the earlier
        // result `v4` (used later by `v6`), and the result `v5` — and the result never reuses an
        // operand's register. Two parameters take registers 0 and 1; capacity 5 leaves three
        // non-parameter registers, one short, so the scan must spill one and reload it at its later
        // use. No constants appear, so nothing piles up at the block entry. The plan must stay sound
        // (no two register-resident values share a register over overlapping points).
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = add v0, v1
            v3 = mul v0, v1
            v4 = add v2, v3
            v5 = mul v2, v3
            v6 = add v4, v5
            return v6
        }
        ";
        let (ranges, use_points, ssa) = ranges_for(src);
        let plan =
            assign(&ranges, &use_points, 5, &entry_params(&ssa)).expect("placeable at capacity 5");
        assert!(plan.num_spill_slots() > 0, "capacity 5 should force at least one spill");
        assert_sound(&ranges, &plan, 5);

        // Every value has a home at every point of its liveness (register or slot).
        for (value, value_ranges) in ranges.iter() {
            for range in value_ranges {
                assert!(
                    plan.location_at(value, range.start).is_some(),
                    "value {value} has no location at the start of a live range"
                );
            }
        }
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
        let (ranges, use_points, ssa) = ranges_for(src);
        let plan =
            assign(&ranges, &use_points, 8, &entry_params(&ssa)).expect("fits in 8 registers");
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
