//! Register spill management for the Brillig code generator.
//!
//! # When spilling kicks in
//!
//! Most functions fit comfortably inside the per-frame register budget, in which case the
//! code generator just uses coalescing to reuse registers and never constructs a
//! [`SpillManager`]. Spilling activates only when [`VariableLiveness::max_live_count`]
//! exceeds the available registers for the current [`LayoutConfig`] — i.e. there is some
//! program point where too many values are simultaneously live. When the spill manager is
//! active coalescing is disabled: the two are mutually exclusive allocation strategies.
//!
//! # Scope of decisions
//!
//! The spill manager operates one basic block at a time. Inside a block, spilled values can
//! be reloaded into any free register and their heap slot re-used by a later spill, so the
//! manager effectively acts as a pool allocator for _transient_ spills. Across blocks this
//! is unsafe — the next block can't predict which register a predecessor left a value in —
//! so live-across-block values (block parameters, non-param live-ins) are promoted to
//! _permanent_ spills: their slot is reserved for the remainder of the function and every
//! use reloads from it.
//!
//! Transient slots are freed as soon as the value dies; permanent slots are never freed
//! (see <https://github.com/noir-lang/noir/issues/11695>). Each block begins with
//! [`SpillManager::begin_block`], which (a) asserts no transient spills leaked from the
//! previous block, (b) re-marks permanent slots as "currently spilled" so their owners
//! must reload before use, and (c) rebuilds the LRU with this block's live-in set.
//!
//! # Per-use flow
//!
//! The code generator calls into the spill manager at a small number of well-defined points:
//!
//! - Before making room for a fresh value, [`BrilligBlock::ensure_register_capacity`] asks
//!   [`SpillManager::lru_victim`] for the least-recently-used non-spilled value and spills
//!   it via [`BrilligBlock::spill_value`]. That emits the store and frees the register.
//! - When a spilled value is needed, [`BrilligBlock::reload_spilled_value`] allocates a
//!   fresh register, emits the load, and calls [`SpillManager::unmark_spilled`] to record
//!   that the value is once again in a register (the slot still holds a valid copy).
//! - When a value is used, [`SpillManager::touch`] bumps it to the most-recently-used end
//!   of the LRU, so that freshly-loaded and just-produced values aren't the first victims
//!   of the next eviction.
//! - When a value dies, [`SpillManager::remove_spill`] frees its transient slot (permanent
//!   slots are kept for the rest of the function).
//!
//! [`VariableLiveness::max_live_count`]: super::variable_liveness::VariableLiveness::max_live_count
//! [`LayoutConfig`]: crate::brillig::brillig_ir::LayoutConfig
//! [`BrilligBlock::ensure_register_capacity`]: super::brillig_block::BrilligBlock::ensure_register_capacity
//! [`BrilligBlock::spill_value`]: super::brillig_block::BrilligBlock::spill_value
//! [`BrilligBlock::reload_spilled_value`]: super::brillig_block::BrilligBlock::reload_spilled_value

use std::collections::BTreeSet;
use std::collections::hash_map::Entry;

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::brillig::brillig_ir::brillig_variable::BrilligVariable;
use crate::ssa::ir::value::ValueId;

/// The register-allocation state the spill manager consults but does not own.
///
/// The manager tracks which values have a heap slot and whether that slot is transient or
/// permanent; whether a value is *currently* in a register is owned elsewhere — by
/// [`BlockVariables`] in real codegen, or by a mock in tests — and injected through this trait.
/// Together they answer "is this value currently spilled": it has a slot and is not in a
/// register (see [`SpillManager::is_spilled`]).
///
/// [`BlockVariables`]: super::brillig_block_variables::BlockVariables
pub(crate) trait RegisterState {
    /// Whether `value_id` currently occupies a register.
    fn is_in_register(&self, value_id: &ValueId) -> bool;
}

/// Tracks register values that have been spilled to the spill region in heap memory.
///
/// The manager owns only *slot* bookkeeping (which values have a heap slot, and whether that
/// slot is transient or permanent) plus the LRU. It deliberately does not track whether a value
/// is currently in a register — that is owned by the register allocator and reached through
/// [`RegisterState`]. "Currently spilled" is derived: `has_slot(v) && !is_in_register(v)`.
///
/// See the [module docs][self] for an overview of when and how the spill manager is used.
pub(crate) struct SpillManager {
    /// Map of all spill records
    records: HashMap<ValueId, SpillRecord>,
    /// Least-recently-used tracker over local SSA values (not globals, not hoisted constants).
    lru: Lru,
    /// Next offset within the spill region (relative to spill base register).
    next_spill_offset: usize,
    /// Free list of spill slots that have been reclaimed.
    free_spill_slots: Vec<usize>,
    /// The maximum number of spill slots needed across all blocks in this function.
    max_spill_offset: usize,
}

/// A heap slot reserved for a spilled value.
///
/// # Value lifecycle
///
/// A value's state is the combination of its *slot* (tracked here: none, transient, or
/// permanent) and whether it is *currently in a register* (tracked by [`RegisterState`], not
/// stored here). "Spilled" means it has a slot but is not in a register. The combinations —
/// and the names the former `SpillStatus` enum gave them — are:
///
/// | slot      | in register | former name         | meaning                                       |
/// |-----------|-------------|---------------------|-----------------------------------------------|
/// | none      | yes         | (no record)         | normal value, lives only in a register        |
/// | transient | no          | `Transient`         | within-block spill, awaiting reload           |
/// | transient | yes         | `TransientReloaded` | reloaded; transient slot still reserved       |
/// | permanent | no          | `Permanent`         | cross-block spill, heap slot is authoritative |
/// | permanent | yes         | `PermanentReloaded` | reloaded; permanent slot still authoritative  |
///
/// Transitions:
/// - First eviction → gains a transient slot (or, across a block boundary, a permanent one);
///   the register is freed.
/// - Reloaded into a register, or evicted again by the LRU → purely a [`RegisterState`] change;
///   the slot itself is unchanged.
/// - Block boundary → the register state resets (nothing is in a register); permanent slots
///   persist while transient slots must not survive.
#[derive(Clone, Copy)]
pub(crate) struct SpillRecord {
    /// Offset relative to the per-frame heap-allocated spill region base.
    pub(crate) offset: usize,
    /// Original variable (for type/register info on reload).
    pub(crate) variable: BrilligVariable,
    /// Whether the slot is permanent (reserved for the rest of the function, the source of
    /// truth across block boundaries) rather than transient (freed when the value dies).
    permanent: bool,
}

/// Least-recently-used tracker over local SSA values, keyed by a monotonic touch clock.
///
/// Backs the eviction heuristic with logarithmic `touch`/`remove` instead of the linear
/// scans and shifts a `Vec` would require (see <https://github.com/noir-lang/noir/issues/11694>).
/// Values are ordered by the time they were last touched: the least-recently-used value is
/// the first element of `order`, the most-recently-used the last.
///
/// Invariant: when `SpillManager::lru_victim` consults the LRU, no tracked value is spilled.
/// The methods that record a spill (`record_spill`, `record_permanent_spill`,
/// `ensure_permanent_spill`) drop the value from the LRU, so the order only ever yields values
/// that are in a register. `lru_victim` asserts it.
#[derive(Default)]
struct Lru {
    /// Logical clock, incremented once per `touch`. Strictly increasing, so no two entries
    /// ever share a key and the ordering is total and deterministic.
    clock: u64,
    /// Last touch time of each tracked value.
    last_used: HashMap<ValueId, u64>,
    /// Tracked values ordered by touch time (ascending): first = least recently used.
    order: BTreeSet<(u64, ValueId)>,
}

impl Lru {
    fn new() -> Self {
        Self::default()
    }

    /// Mark `value_id` as the most recently used value, inserting it if not already tracked.
    fn touch(&mut self, value_id: ValueId) {
        self.clock += 1;
        if let Some(previous) = self.last_used.insert(value_id, self.clock) {
            self.order.remove(&(previous, value_id));
        }
        self.order.insert((self.clock, value_id));
    }

    /// Stop tracking `value_id` entirely.
    fn remove(&mut self, value_id: &ValueId) {
        if let Some(previous) = self.last_used.remove(value_id) {
            self.order.remove(&(previous, *value_id));
        }
    }

    /// Iterate tracked values from least- to most-recently used.
    fn iter(&self) -> impl Iterator<Item = ValueId> + '_ {
        self.order.iter().map(|&(_, value_id)| value_id)
    }

    /// Forget all tracked values. The clock is left untouched so it stays monotonic.
    fn clear(&mut self) {
        self.last_used.clear();
        self.order.clear();
    }

    /// Insert `values` as equally — and least — recently used (one shared clock tick).
    ///
    /// Because `order` is keyed by `(clock, value_id)`, entries sharing a clock are ordered by
    /// [`ValueId`], so the caller need not pre-sort to get a deterministic ordering.
    fn seed(&mut self, values: impl IntoIterator<Item = ValueId>) {
        self.clock += 1;
        let clock = self.clock;
        for value_id in values {
            if let Some(previous) = self.last_used.insert(value_id, clock) {
                self.order.remove(&(previous, value_id));
            }
            self.order.insert((clock, value_id));
        }
    }
}

impl SpillManager {
    pub(crate) fn new() -> Self {
        Self {
            records: HashMap::default(),
            lru: Lru::new(),
            next_spill_offset: 0,
            free_spill_slots: Vec::new(),
            max_spill_offset: 0,
        }
    }

    /// Prepare spill state for a new block.
    ///
    /// 1. Asserts that no *live-in* value carries a transient slot. A value that lives across a
    ///    block boundary must be permanently spilled (transient slots are within-block), so a
    ///    transient slot for a live-in value signals a leak. A leftover transient slot for a
    ///    value that is *not* live-in (e.g. one that died at the terminator after being
    ///    reloaded for it) is a benign within-block artifact and is tolerated.
    /// 2. Removes permanently-spilled values from the live-in set: the register state is reset
    ///    at block entry, so these must be reloaded before use.
    /// 3. Rebuilds the LRU from the surviving live-in set.
    ///
    /// All three steps run *before* the block's register state exists — `BrilligBlock::compile_block`
    /// builds the new `BlockVariables` and resets the registers only after `begin_block` returns
    /// — so they decide purely from slot state and the live-in set, never from [`RegisterState`]
    /// (which has no valid value to consult here).
    pub(crate) fn begin_block(&mut self, live_in: &mut HashSet<ValueId>) {
        // A live-in value must be permanently spilled; only a transient slot for one is a leak.
        assert!(
            !self.records.iter().any(|(v, r)| !r.permanent && live_in.contains(v)),
            "Transient spill leaked across block boundary: {:?}",
            self.records
                .iter()
                .filter(|(v, r)| !r.permanent && live_in.contains(v))
                .map(|(v, _)| *v)
                .collect::<Vec<_>>()
        );
        live_in.retain(|v| !self.has_permanent_slot(v));
        self.reset_lru_for_block(live_in);
    }

    /// Whether a value is currently spilled: it has a heap slot but is not in a register.
    pub(crate) fn is_spilled(&self, value_id: &ValueId, registers: &impl RegisterState) -> bool {
        self.has_slot(value_id) && !registers.is_in_register(value_id)
    }

    /// Whether a value was transiently spilled and has since been reloaded into a register.
    pub(crate) fn is_transient_reloaded(
        &self,
        value_id: &ValueId,
        registers: &impl RegisterState,
    ) -> bool {
        self.has_transient_slot(value_id) && registers.is_in_register(value_id)
    }

    /// Whether a value has any heap slot (transient or permanent).
    fn has_slot(&self, value_id: &ValueId) -> bool {
        self.records.contains_key(value_id)
    }

    /// Whether a value has a transient heap slot.
    fn has_transient_slot(&self, value_id: &ValueId) -> bool {
        self.records.get(value_id).is_some_and(|r| !r.permanent)
    }

    /// Whether a value has a permanent heap slot.
    pub(crate) fn has_permanent_slot(&self, value_id: &ValueId) -> bool {
        self.records.get(value_id).is_some_and(|r| r.permanent)
    }

    /// Move a value to the back of the LRU (most recently used), inserting it if absent.
    ///
    /// The value must currently be in a register: the LRU only ever holds register-resident
    /// values (a definition or a reload is the only way one enters). Asserting that here keeps a
    /// spilled value from ever entering the LRU, which is what lets [`Self::lru_victim`] hand back
    /// its oldest entry without re-checking — a spilled value can only get in through an erroneous
    /// touch, and this catches it at the source.
    pub(crate) fn touch(&mut self, value_id: ValueId, registers: &impl RegisterState) {
        assert!(
            registers.is_in_register(&value_id),
            "touched {value_id} which is not in a register"
        );
        self.lru.touch(value_id);
    }

    /// Remove a value from LRU tracking entirely.
    fn remove_from_lru(&mut self, value_id: &ValueId) {
        self.lru.remove(value_id);
    }

    /// Remove a (dead) value from spill tracking and from the LRU.
    ///
    /// A transient slot is reclaimed to the free list. A permanent slot is never freed — it must
    /// remain valid across all blocks — so the record is kept. Either way the value is dropped
    /// from the LRU, since a value that is gone must never be an eviction candidate.
    ///
    /// TODO(<https://github.com/noir-lang/noir/issues/11695>) - Free globally dead permanent spill slots
    pub(crate) fn remove_spill(&mut self, value_id: &ValueId) {
        // A transient slot is reclaimed; a permanent slot is kept (never freed).
        if let Entry::Occupied(entry) = self.records.entry(*value_id)
            && !entry.get().permanent
        {
            let record = entry.remove();
            self.free_spill_slots.push(record.offset);
        }
        self.remove_from_lru(value_id);
    }

    /// Allocate a spill slot offset, reusing a freed slot if available.
    pub(crate) fn allocate_spill_offset(&mut self) -> usize {
        let offset = if let Some(offset) = self.free_spill_slots.pop() {
            offset
        } else {
            let offset = self.next_spill_offset;
            self.next_spill_offset += 1;
            offset
        };
        self.max_spill_offset = self.max_spill_offset.max(offset + 1);
        offset
    }

    /// The maximum number of spill slots used across all blocks.
    pub(crate) fn max_spill_offset(&self) -> usize {
        self.max_spill_offset
    }

    /// Return the least recently used value tracked in the LRU, or `None` if it is empty.
    ///
    /// Every tracked value is in a register — [`Self::touch`] only admits register-resident
    /// values and spills/deaths drop them — so the oldest entry is always a valid eviction
    /// candidate, with no spill check needed here.
    pub(crate) fn lru_victim(&self) -> Option<ValueId> {
        self.lru.iter().next()
    }

    /// Batched version of [`Self::lru_victim`]: the `k` least-recently-used values, oldest first.
    pub(crate) fn lru_victims(&self, k: usize) -> Vec<ValueId> {
        self.lru.iter().take(k).collect()
    }

    /// Record that a value has been spilled to a transient slot, dropping it from the LRU since
    /// a value living in its heap slot must never be an eviction candidate.
    ///
    /// If the value already has a slot (e.g. it was reloaded and is now being re-evicted), the
    /// existing slot is kept — including its kind, so a permanent slot stays permanent — and only
    /// the variable is refreshed.
    ///
    /// # Precondition
    /// `value_id` must still be in a register when this is called: the caller frees its register
    /// only *after* recording the spill. That ordering is what makes the double-spill assert
    /// meaningful — an already-spilled value (has a slot and is not in a register) trips it, while
    /// a legitimate re-eviction of a reloaded value passes because it is still in a register here.
    pub(crate) fn record_spill(
        &mut self,
        value_id: ValueId,
        offset: usize,
        variable: BrilligVariable,
        registers: &impl RegisterState,
    ) {
        assert!(!self.is_spilled(&value_id, registers), "Double-spill of {value_id}");
        match self.records.entry(value_id) {
            Entry::Occupied(mut entry) => {
                // Always preserve the offset, so we don't leak free slots.
                assert_eq!(
                    offset,
                    entry.get().offset,
                    "Spill of {value_id} orphaned existing slot"
                );
                entry.get_mut().variable = variable;
            }
            Entry::Vacant(entry) => {
                entry.insert(SpillRecord { offset, variable, permanent: false });
            }
        }
        self.remove_from_lru(&value_id);
    }

    /// Get the spill record for a value if it is currently spilled (has a slot and is not in a
    /// register).
    pub(crate) fn get_spill(
        &self,
        value_id: &ValueId,
        registers: &impl RegisterState,
    ) -> Option<&SpillRecord> {
        self.records.get(value_id).filter(|_| !registers.is_in_register(value_id))
    }

    /// Rebuild the LRU for a new block from the surviving live-in set.
    ///
    /// Called only from [`Self::begin_block`], after permanently-spilled values have been removed
    /// from `live_in`. Like `begin_block`, it runs before the block's register state exists, so it
    /// is register-free.
    ///
    /// The `debug_assert!` is a trip-wire: today every value live across a block boundary is
    /// permanently spilled, so nothing live-in survives in the carried-over LRU. If a future
    /// register allocator (<https://github.com/noir-lang/noir/issues/11638>) keeps live values in
    /// registers across blocks it fires — a signal that re-seeding by `ValueId` discards a real
    /// previous-block recency ordering we should be preserving instead.
    fn reset_lru_for_block(&mut self, live_in: &HashSet<ValueId>) {
        debug_assert!(
            !self.lru.iter().any(|v| live_in.contains(&v)),
            "cross-block LRU carryover is non-empty; preserve the previous block's order instead of re-seeding: {:?}",
            self.lru.iter().collect::<Vec<_>>()
        );
        self.lru.clear();
        // Seed every surviving live-in as equally least-recently-used; the LRU orders them by
        // `ValueId`, so no explicit sort is needed.
        self.lru.seed(live_in.iter().copied());
    }

    /// Record a value as permanently spilled, dropping it from the LRU (like [`Self::record_spill`]).
    ///
    /// If the value already has a transient spill slot, it is promoted to permanent.
    /// The permanent slot ensures consistency regardless of what happens during block processing.
    pub(crate) fn record_permanent_spill(
        &mut self,
        value_id: ValueId,
        offset: usize,
        variable: BrilligVariable,
    ) {
        self.records
            .entry(value_id)
            .and_modify(|record| {
                assert_eq!(record.offset, offset);
                record.permanent = true;
                record.variable = variable;
            })
            .or_insert(SpillRecord { offset, variable, permanent: true });
        self.remove_from_lru(&value_id);
    }

    /// Get the permanent spill slot offset for a value, if any.
    pub(crate) fn get_permanent_spill_offset(&self, value_id: &ValueId) -> Option<usize> {
        self.records.get(value_id).filter(|r| r.permanent).map(|r| r.offset)
    }

    /// Get the spill slot offset for a value, if any.
    ///
    /// Unlike `get_permanent_spill_offset` this could be a permanent or a transient spill;
    /// there should be only one at any point.
    pub(crate) fn get_spill_offset(&self, value_id: &ValueId) -> Option<usize> {
        self.records.get(value_id).map(|r| r.offset)
    }

    /// Promote an existing spill record to a permanent slot, dropping it from the LRU.
    ///
    /// The permanent slot becomes the value's authoritative copy across block boundaries. This
    /// updates slot bookkeeping only; whether the value still occupies a register is the caller's
    /// concern — see `BrilligBlock::spill_value`.
    ///
    /// # Returns
    /// * `true` — a record already existed, so the value already owns a permanent slot; the
    ///   caller can return early without allocating a slot or emitting a store.
    /// * `false` — no record existed: this is the value's first spill, so the caller must
    ///   allocate a slot and record the spill itself.
    pub(crate) fn ensure_permanent_spill(&mut self, value_id: &ValueId) -> bool {
        let Some(record) = self.records.get_mut(value_id) else {
            return false;
        };
        record.permanent = true;
        self.remove_from_lru(value_id);
        true
    }
}

#[cfg(test)]
mod tests {
    use acvm::acir::brillig::MemoryAddress;

    use crate::{
        brillig::brillig_ir::brillig_variable::{BrilligVariable, SingleAddrVariable},
        ssa::ir::map::Id,
    };

    use rustc_hash::FxHashSet;

    use super::{Lru, RegisterState, SpillManager};
    use crate::ssa::ir::value::ValueId;

    fn test_var(n: u32) -> BrilligVariable {
        BrilligVariable::SingleAddr(SingleAddrVariable::new(MemoryAddress::relative(n), 32))
    }

    /// Stand-in for the register allocator the spill manager consults but does not own.
    ///
    /// Tests drive this explicitly to mirror what `BrilligBlock` does to the real registers:
    /// `allocate` is "a register now holds the value" (e.g. a reload, formerly `unmark_spilled`),
    /// and `free` is "the register was released" (e.g. after a spill, or the block-entry reset
    /// that formerly went through `restore_permanent_spills`).
    #[derive(Default)]
    struct MockRegisters {
        in_register: FxHashSet<ValueId>,
    }

    impl RegisterState for MockRegisters {
        fn is_in_register(&self, value_id: &ValueId) -> bool {
            self.in_register.contains(value_id)
        }
    }

    impl MockRegisters {
        fn allocate(&mut self, value_id: ValueId) {
            self.in_register.insert(value_id);
        }

        fn free(&mut self, value_id: ValueId) {
            self.in_register.remove(&value_id);
        }
    }

    #[test]
    fn lru_orders_values_by_touch_recency() {
        let mut lru = Lru::new();
        let v0 = Id::test_new(0);
        let v1 = Id::test_new(1);
        let v2 = Id::test_new(2);

        lru.touch(v0);
        lru.touch(v1);
        lru.touch(v2);
        assert_eq!(lru.iter().collect::<Vec<_>>(), vec![v0, v1, v2]);

        // Re-touching a tracked value moves it to the most-recently-used end.
        lru.touch(v0);
        assert_eq!(lru.iter().collect::<Vec<_>>(), vec![v1, v2, v0]);
    }

    #[test]
    fn lru_remove_drops_value_from_both_collections() {
        let mut lru = Lru::new();
        let v0 = Id::test_new(0);
        let v1 = Id::test_new(1);

        lru.touch(v0);
        lru.touch(v1);

        lru.remove(&v0);
        assert_eq!(lru.iter().collect::<Vec<_>>(), vec![v1]);
        assert!(!lru.last_used.contains_key(&v0), "stamp must be dropped too");

        // Removing an untracked value is a no-op.
        lru.remove(&v0);
        assert_eq!(lru.iter().collect::<Vec<_>>(), vec![v1]);
    }

    #[test]
    fn lru_clear_empties_order_but_keeps_clock_monotonic() {
        let mut lru = Lru::new();
        let v0 = Id::test_new(0);
        let v1 = Id::test_new(1);

        lru.touch(v0);
        let clock_before_clear = lru.clock;

        lru.clear();
        assert!(lru.iter().next().is_none());

        // A later touch must outrank anything from before the clear.
        lru.touch(v1);
        assert!(lru.clock > clock_before_clear, "clock must keep increasing across clear");
        assert_eq!(lru.iter().collect::<Vec<_>>(), vec![v1]);
    }

    #[test]
    fn lru_ordering_and_victim_selection() {
        let mut sm = SpillManager::new();
        let mut regs = MockRegisters::default();
        let v0 = Id::test_new(0);
        let v1 = Id::test_new(1);
        let v2 = Id::test_new(2);

        // Touch v0, v1, v2 in order (all in registers). LRU order: [v0, v1, v2]
        for v in [v0, v1, v2] {
            regs.allocate(v);
            sm.touch(v, &regs);
        }

        // Victim should be v0 (least recently used)
        assert_eq!(sm.lru_victim(), Some(v0));

        // Touch v0 again, making it most recent. LRU order: [v1, v2, v0]
        sm.touch(v0, &regs);

        // Victim should now be v1
        assert_eq!(sm.lru_victim(), Some(v1));
    }

    #[test]
    fn lru_victims_returns_the_k_least_recently_used() {
        let mut sm = SpillManager::new();
        let mut regs = MockRegisters::default();
        let v0 = Id::test_new(0);
        let v1 = Id::test_new(1);
        let v2 = Id::test_new(2);
        let v3 = Id::test_new(3);

        for v in [v0, v1, v2, v3] {
            regs.allocate(v);
            sm.touch(v, &regs);
        }

        // The `k` least-recently-used, oldest first.
        assert_eq!(sm.lru_victims(2), vec![v0, v1]);
        // `k` larger than the tracked set returns everything.
        assert_eq!(sm.lru_victims(10), vec![v0, v1, v2, v3]);
    }

    #[test]
    fn spilling_removes_value_from_lru_so_it_is_not_a_victim() {
        let mut sm = SpillManager::new();
        let mut regs = MockRegisters::default();
        let v0 = Id::test_new(0);
        let v1 = Id::test_new(1);
        let v2 = Id::test_new(2);

        // All three are in registers and tracked in the LRU.
        for v in [v0, v1, v2] {
            regs.allocate(v);
            sm.touch(v, &regs);
        }

        // Spill v0: record_spill drops it from the LRU, then its register is freed.
        let offset = sm.allocate_spill_offset();
        sm.record_spill(v0, offset, test_var(0), &regs);
        regs.free(v0);

        assert!(sm.is_spilled(&v0, &regs));
        assert!(!sm.is_spilled(&v1, &regs));

        let record = sm.get_spill(&v0, &regs).unwrap();
        assert_eq!(record.offset, 0);
        assert!(!sm.has_permanent_slot(&v0)); // it is a transient slot

        // v0 is no longer tracked, so the next-oldest live value is the victim.
        assert_eq!(sm.lru_victim(), Some(v1));
    }

    #[test]
    #[should_panic(expected = "is not in a register")]
    fn touch_rejects_a_value_not_in_a_register() {
        // The LRU must only ever hold register-resident values. Touching a spilled value is the
        // only way a spilled value could enter it, and `touch` catches that at the source.
        let mut sm = SpillManager::new();
        let mut regs = MockRegisters::default();
        let v0 = Id::test_new(0);

        regs.allocate(v0);
        let offset = sm.allocate_spill_offset();
        sm.record_spill(v0, offset, test_var(0), &regs);
        regs.free(v0); // v0 is now spilled (out of a register)

        sm.touch(v0, &regs); // touching a value that is not in a register panics
    }

    #[test]
    fn spill_slot_allocation_and_reuse() {
        let mut sm = SpillManager::new();
        let mut regs = MockRegisters::default();
        let v0 = Id::test_new(0);
        let v1 = Id::test_new(1);

        // Allocate two slots: offsets 0 and 1
        regs.allocate(v0);
        regs.allocate(v1);
        let off0 = sm.allocate_spill_offset();
        sm.record_spill(v0, off0, test_var(0), &regs);
        let off1 = sm.allocate_spill_offset();
        sm.record_spill(v1, off1, test_var(1), &regs);
        assert_eq!(off0, 0);
        assert_eq!(off1, 1);

        // Free slot 0 by removing the spill (the value died).
        sm.remove_spill(&v0);
        assert!(!sm.is_spilled(&v0, &regs));

        // Next allocation should reuse freed slot 0
        let off_reused = sm.allocate_spill_offset();
        assert_eq!(off_reused, 0);

        // Then a fresh slot at 2
        let off_fresh = sm.allocate_spill_offset();
        assert_eq!(off_fresh, 2);
    }

    #[test]
    fn remove_from_lru() {
        let mut sm = SpillManager::new();
        let mut regs = MockRegisters::default();
        let v0 = Id::test_new(0);
        let v1 = Id::test_new(1);
        let v2 = Id::test_new(2);

        for v in [v0, v1, v2] {
            regs.allocate(v);
            sm.touch(v, &regs);
        }

        // Remove v1 from LRU entirely. LRU order: [v0, v2]
        sm.remove_from_lru(&v1);

        // Victim should be v0 (v1 is absent)
        assert_eq!(sm.lru_victim(), Some(v0));

        // Touch v0, making it most recent. LRU order: [v2, v0]
        sm.touch(v0, &regs);

        // Victim should be v2 (v1 is absent, v0 was touched)
        assert_eq!(sm.lru_victim(), Some(v2));
    }

    #[test]
    fn permanent_spill_lifecycle() {
        let mut sm = SpillManager::new();
        let mut regs = MockRegisters::default();
        let v0 = Id::test_new(0);

        // The value starts in a register, then is permanently spilled (register then freed).
        regs.allocate(v0);
        let off = sm.allocate_spill_offset();
        sm.record_permanent_spill(v0, off, test_var(0));
        regs.free(v0);
        assert_eq!(off, 0);
        assert!(sm.is_spilled(&v0, &regs));
        assert!(sm.has_permanent_slot(&v0));
        assert_eq!(sm.get_permanent_spill_offset(&v0), Some(0));

        // Reload: a register is allocated again (was `unmark_spilled`); the slot stays.
        regs.allocate(v0);
        assert!(!sm.is_spilled(&v0, &regs));
        assert!(sm.has_permanent_slot(&v0));
        assert_eq!(sm.get_permanent_spill_offset(&v0), Some(0));

        // While reloaded (in a register) the value is a normal eviction candidate.
        sm.touch(v0, &regs);
        assert_eq!(sm.lru_victim(), Some(v0));

        // Block entry resets the register state (was `restore_permanent_spills`): no longer resident.
        regs.free(v0);
        assert!(sm.is_spilled(&v0, &regs));

        // The value dies: the permanent slot is kept, but it is dropped from the LRU.
        sm.remove_spill(&v0);
        assert!(sm.is_spilled(&v0, &regs));
        // Permanent record still exists
        assert!(sm.has_permanent_slot(&v0));
        // Slot is NOT freed (no slot in free list)
        assert!(sm.free_spill_slots.is_empty());
        // The dead value was dropped from the LRU, so it is no longer an eviction candidate.
        assert_eq!(sm.lru_victim(), None);
    }

    #[test]
    fn promote_transient_to_permanent() {
        let mut sm = SpillManager::new();
        let mut regs = MockRegisters::default();
        let v0 = Id::test_new(0);

        // Transient spill: in a register, then spilled (register freed).
        regs.allocate(v0);
        let off = sm.allocate_spill_offset();
        sm.record_spill(v0, off, test_var(0), &regs);
        regs.free(v0);
        assert!(sm.is_spilled(&v0, &regs));
        assert!(!sm.has_permanent_slot(&v0));

        // Promote to permanent via ensure_permanent_spill
        assert!(sm.ensure_permanent_spill(&v0));
        assert!(sm.is_spilled(&v0, &regs));
        assert!(sm.has_permanent_slot(&v0));
        assert_eq!(sm.get_permanent_spill_offset(&v0), Some(0));

        // Removing a promoted permanent spill keeps the slot and does NOT free it.
        sm.remove_spill(&v0);
        assert!(sm.is_spilled(&v0, &regs));
        assert!(sm.free_spill_slots.is_empty());
        // Although still `is_spilled`, the dead value must not be returned by the LRU.
        assert_eq!(sm.lru_victim(), None);
    }

    #[test]
    fn ensure_permanent_spill_returns_true_when_a_record_exists() {
        let mut sm = SpillManager::new();
        let regs = MockRegisters::default();
        let v0 = Id::test_new(0);

        // A transient spill record exists; ensure_permanent_spill promotes it and returns true.
        let off = sm.allocate_spill_offset();
        sm.record_spill(v0, off, test_var(0), &regs);
        assert!(sm.ensure_permanent_spill(&v0), "expect true because the record exists");
        assert!(sm.has_permanent_slot(&v0));
    }

    #[test]
    #[should_panic(expected = "Transient spill leaked across block boundary")]
    fn begin_block_panics_on_transient_spill_leak() {
        let mut sm = SpillManager::new();
        let regs = MockRegisters::default();
        let v0 = Id::test_new(0);

        // A live-in value carrying only a transient slot is a leak (it should be permanent).
        let off = sm.allocate_spill_offset();
        sm.record_spill(v0, off, test_var(0), &regs);

        let mut live_in = FxHashSet::default();
        live_in.insert(v0);
        sm.begin_block(&mut live_in);
    }

    #[test]
    #[should_panic(expected = "Double-spill")]
    fn record_spill_panics_on_double_spill() {
        let mut sm = SpillManager::new();
        let mut regs = MockRegisters::default();
        let v0 = Id::test_new(0);

        regs.allocate(v0);
        let off = sm.allocate_spill_offset();
        sm.record_spill(v0, off, test_var(0), &regs);
        regs.free(v0); // spilling frees the register; v0 is now spilled

        // Spilling v0 again while it is already spilled (out of a register) is a double-spill.
        let off2 = sm.allocate_spill_offset();
        sm.record_spill(v0, off2, test_var(0), &regs);
    }

    #[test]
    #[should_panic(expected = "orphaned existing slot")]
    fn record_spill_panics_on_orphaned_slot() {
        let mut sm = SpillManager::new();
        let mut regs = MockRegisters::default();
        let v0 = Id::test_new(0);

        regs.allocate(v0);
        let off = sm.allocate_spill_offset();
        sm.record_spill(v0, off, test_var(0), &regs);
        assert_eq!(sm.get_spill_offset(&v0), Some(off));

        // v0 stays in a register (re-eviction), so this passes the double-spill check, but a
        // different offset orphans the existing slot.
        let off2 = sm.allocate_spill_offset();
        sm.record_spill(v0, off2, test_var(0), &regs);
    }

    #[test]
    fn record_spill_of_reloaded_value_with_same_offset() {
        let mut sm = SpillManager::new();
        let mut regs = MockRegisters::default();
        let v0 = Id::test_new(0);

        regs.allocate(v0);
        let off = sm.allocate_spill_offset();
        sm.record_spill(v0, off, test_var(0), &regs);
        // v0 stays in a register (reloaded); re-recording the same offset is fine.
        sm.record_spill(v0, off, test_var(1), &regs); // Different BrilligVariable to show it's not checked.
    }

    #[test]
    fn ensure_permanent_spill_all_branches() {
        let mut sm = SpillManager::new();
        let regs = MockRegisters::default();
        let v0 = Id::test_new(0);
        let v1 = Id::test_new(1);
        let v3 = Id::test_new(3);

        // Case 1: No record -> returns false
        assert!(!sm.ensure_permanent_spill(&v3));

        // Case 2: Already permanent -> returns true, stays permanent
        let off0 = sm.allocate_spill_offset();
        sm.record_permanent_spill(v0, off0, test_var(0));
        assert!(sm.has_permanent_slot(&v0));
        assert!(sm.ensure_permanent_spill(&v0));
        assert!(sm.has_permanent_slot(&v0));

        // Case 3: Transient -> returns true, promoted to permanent
        let off1 = sm.allocate_spill_offset();
        sm.record_spill(v1, off1, test_var(1), &regs);
        assert!(!sm.has_permanent_slot(&v1));
        assert!(sm.ensure_permanent_spill(&v1));
        assert!(sm.has_permanent_slot(&v1));
    }

    /// Promoting a reloaded permanent value back to spilled must drop it from the LRU, otherwise
    /// a spilled value lingers as an eviction candidate. In real codegen this happens for a
    /// JmpIf condition re-spilled by `spill_non_param_live_ins` then reloaded, with
    /// `ensure_register_capacity` querying `lru_victim` in between.
    #[test]
    fn ensure_permanent_spill_drops_reloaded_value_from_lru() {
        let mut sm = SpillManager::new();
        let mut regs = MockRegisters::default();
        let v0 = Id::test_new(0);

        // Permanent slot, currently reloaded (in a register) and tracked in the LRU.
        regs.allocate(v0);
        let off = sm.allocate_spill_offset();
        sm.record_permanent_spill(v0, off, test_var(0));
        sm.touch(v0, &regs);
        assert!(!sm.is_transient_reloaded(&v0, &regs)); // it is permanent, not transient
        assert_eq!(sm.lru_victim(), Some(v0));

        // ensure_permanent_spill drops it from the LRU, so it is no longer a victim.
        assert!(sm.ensure_permanent_spill(&v0));
        assert_eq!(sm.lru_victim(), None);
    }
}
