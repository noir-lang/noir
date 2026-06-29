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

use itertools::Itertools;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::brillig::brillig_ir::brillig_variable::BrilligVariable;
use crate::ssa::ir::value::ValueId;

/// Lifecycle state of a spill record.
///
/// Transitions:
/// - First eviction → `Transient` or `Permanent`
/// - Reloaded into a register → `TransientReloaded` or `PermanentReloaded`
/// - Evicted again by LRU → back to `Transient` or `Permanent`
/// - Block boundary → `PermanentReloaded` becomes `Permanent`; `Transient` must not survive
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SpillStatus {
    /// Within-block spill: value is in the spill slot, not in a register.
    Transient,
    /// Was transiently spilled then reloaded into a register.
    /// The spill slot is still allocated and the register holds the live value.
    TransientReloaded,
    /// Permanent spill: the slot is the source of truth across block boundaries.
    /// Value is not currently in a register.
    Permanent,
    /// Permanently spilled, but currently also loaded into a register.
    /// The slot remains authoritative; the register is a transient copy.
    PermanentReloaded,
}

impl SpillStatus {
    /// Whether the value is currently in memory rather than a register.
    fn is_spilled(self) -> bool {
        matches!(self, SpillStatus::Transient | SpillStatus::Permanent)
    }
}

/// Tracks register values that have been spilled to the spill region in heap memory.
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

#[derive(Clone, Copy)]
pub(crate) struct SpillRecord {
    /// Offset relative to the per-frame heap-allocated spill region base.
    pub(crate) offset: usize,
    /// Original variable (for type/register info on reload).
    pub(crate) variable: BrilligVariable,
    /// Current lifecycle state of this spill record.
    pub(crate) status: SpillStatus,
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
    /// 1. Asserts that no transient spills leaked from the previous block
    ///    (only `Permanent` and `PermanentReloaded` records may survive).
    /// 2. Restores permanently-spilled values by marking them as currently spilled.
    /// 3. Removes spilled values from the live-in set (they have no register).
    /// 4. Updates the LRU: rebuilds the LRU from the live-in set in [`ValueId`] order.
    pub(crate) fn begin_block(&mut self, live_in: &mut HashSet<ValueId>) {
        // No transient spills should survive across block boundaries.
        assert!(
            self.records.values().all(|r| r.status != SpillStatus::Transient),
            "Transient spill leaked across block boundary"
        );
        self.restore_permanent_spills();
        live_in.retain(|v| !self.is_spilled(v));
        self.reset_lru_for_block(live_in);
    }

    /// Check if a value is currently spilled (in memory, not in a register).
    pub(crate) fn is_spilled(&self, value_id: &ValueId) -> bool {
        self.get_spill(value_id).is_some()
    }

    /// Check if a value was transiently spilled and has since been reloaded into a register.
    pub(crate) fn is_transient_reloaded(&self, value_id: &ValueId) -> bool {
        matches!(self.records.get(value_id), Some(r) if r.status == SpillStatus::TransientReloaded)
    }

    /// Check if a value has a permanent spill slot.
    #[cfg(test)]
    pub(crate) fn has_permanent_slot(&self, value_id: &ValueId) -> bool {
        matches!(
            self.records.get(value_id),
            Some(r) if matches!(r.status, SpillStatus::Permanent | SpillStatus::PermanentReloaded)
        )
    }

    /// Move a value to the back of the LRU (most recently used).
    /// If the value isn't tracked yet, add it.
    pub(crate) fn touch(&mut self, value_id: ValueId) {
        self.lru.touch(value_id);
    }

    /// Remove a value from LRU tracking entirely.
    fn remove_from_lru(&mut self, value_id: &ValueId) {
        self.lru.remove(value_id);
    }

    /// Remove a (dead) value from spill tracking and from the LRU.
    ///
    /// Transient slots are freed for reuse; permanent slots are never freed — they must remain
    /// valid across all blocks (for a `Permanent` record this only flips the status to
    /// `PermanentReloaded`). Either way the value is dropped from the LRU, since a value that is
    /// gone must never be an eviction candidate.
    ///
    /// TODO(<https://github.com/noir-lang/noir/issues/11695>) - Free globally dead permanent spill slots
    pub(crate) fn remove_spill(&mut self, value_id: &ValueId) {
        if let Entry::Occupied(mut entry) = self.records.entry(*value_id) {
            match entry.get().status {
                SpillStatus::Permanent => {
                    entry.get_mut().status = SpillStatus::PermanentReloaded;
                }
                SpillStatus::PermanentReloaded => {}
                SpillStatus::Transient | SpillStatus::TransientReloaded => {
                    let record = entry.remove();
                    self.free_spill_slots.push(record.offset);
                }
            }
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
    /// Recording a spill drops the value from the LRU (see `record_spill`), so by the time we
    /// pick a victim every tracked value is in a register. The `assert!` guards that invariant:
    /// handing back a spilled value would be a bug, since re-spilling it frees no register.
    pub(crate) fn lru_victim(&self) -> Option<ValueId> {
        let victim = self.lru.iter().next();
        assert!(
            victim.is_none_or(|v| !self.is_spilled(&v)),
            "lru_victim returned a spilled value: {victim:?}"
        );
        victim
    }

    /// Batched version of [`Self::lru_victim`]
    ///
    /// Return up to `k` least-recently-used values, asserting that they are not currently spilled,
    /// ordered least-recently-used first.
    pub(crate) fn lru_victims(&self, k: usize) -> Vec<ValueId> {
        let victims = self.lru.iter().take(k).collect();
        for victim in &victims {
            assert!(!self.is_spilled(victim), "lru_victim returned a spilled value: {victim:?}");
        }
        victims
    }

    /// Record that a value has been spilled, dropping it from the LRU since a value living in
    /// its heap slot must never be an eviction candidate.
    ///
    /// If the value already has a record (e.g., it was reloaded and then
    /// re-evicted by LRU), this updates the existing record:
    /// - `PermanentReloaded`: transitions back to `Permanent` (slot data is still valid).
    /// - `TransientReloaded`: transitions back to `Transient`, updating the variable.
    pub(crate) fn record_spill(
        &mut self,
        value_id: ValueId,
        offset: usize,
        variable: BrilligVariable,
    ) {
        assert!(!self.is_spilled(&value_id), "Double-spill of {value_id}");
        let record = self.records.entry(value_id).or_insert(SpillRecord {
            offset,
            variable,
            status: SpillStatus::Transient,
        });
        // Always preserve the offset, so we don't leak free slots.
        assert_eq!(offset, record.offset, "Spill of {value_id} orphaned existing slot");
        match record.status {
            SpillStatus::PermanentReloaded => {
                // Re-evicting a permanently-spilled value: keep permanent, slot data is valid.
                record.status = SpillStatus::Permanent;
            }
            SpillStatus::TransientReloaded | SpillStatus::Transient => {
                record.status = SpillStatus::Transient;
                record.variable = variable;
            }
            SpillStatus::Permanent => {
                unreachable!(
                    "record_spill called on a Permanent record — is_spilled should have caught this as a double-spill"
                );
            }
        }
        self.remove_from_lru(&value_id);
    }

    /// Get the spill record for a value if it is currently spilled and is not in a register.
    pub(crate) fn get_spill(&self, value_id: &ValueId) -> Option<&SpillRecord> {
        self.records.get(value_id).filter(|r| r.status.is_spilled())
    }

    /// Rebuild `lru_order` for a new block from scratch, retaining only live-in
    /// values that are not currently spilled, in deterministic ValueId order.
    ///
    /// This discards any ordering carried over from the previous block, which is sound while
    /// spilling is enabled: every value live across a block boundary is permanently spilled
    /// before its block is entered (block parameters via `convert_block_params`, other
    /// live-ins via `spill_non_param_live_ins`), so no non-spilled value is ever carried over
    /// and there is no recency order to preserve. The `debug_assert!` enforces that premise.
    ///
    /// If a future register allocator (<https://github.com/noir-lang/noir/issues/11638>) lets
    /// values stay in registers across blocks, the assert will fire — at which point the
    /// surviving entries' previous-block order is a real eviction hint and should be preserved
    /// here rather than re-sorted by `ValueId`.
    fn reset_lru_for_block(&mut self, live_in: &HashSet<ValueId>) {
        debug_assert!(
            !self.lru.iter().any(|v| live_in.contains(&v) && !self.is_spilled(&v)),
            "cross-block LRU carryover is non-empty; preserve the previous block's order instead of re-sorting: {:?}",
            self.lru.iter().collect::<Vec<_>>()
        );
        let seed: Vec<ValueId> =
            live_in.iter().copied().filter(|v| !self.is_spilled(v)).sorted().collect();
        self.lru.clear();
        for value_id in seed {
            self.lru.touch(value_id);
        }
    }

    /// Record a value as permanently spilled, dropping it from the LRU (like [`Self::record_spill`]).
    ///
    /// If the value already has a transient spill record, it is promoted to `Permanent`.
    /// The permanent record ensures consistency regardless of what happens during
    /// block processing.
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
                record.status = SpillStatus::Permanent;
                record.variable = variable;
            })
            .or_insert(SpillRecord { offset, variable, status: SpillStatus::Permanent });
        self.remove_from_lru(&value_id);
    }

    /// Get the permanent spill slot offset for a value, if any.
    pub(crate) fn get_permanent_spill_offset(&self, value_id: &ValueId) -> Option<usize> {
        self.records
            .get(value_id)
            .filter(|r| matches!(r.status, SpillStatus::Permanent | SpillStatus::PermanentReloaded))
            .map(|r| r.offset)
    }

    /// Get the spill slot offset for a value, if any.
    ///
    /// Unlike `get_permanent_spill_offset` this could be a permanent or a transient spill;
    /// there should be only one at any point.
    pub(crate) fn get_spill_offset(&self, value_id: &ValueId) -> Option<usize> {
        self.records.get(value_id).map(|r| r.offset)
    }

    /// Re-mark permanently-spilled values as currently spilled at block entry.
    ///
    /// A reload in a previous block sets the status to `PermanentReloaded`, but the
    /// value must be reloaded again in every block that uses it (since the
    /// permanent spill slot is always the source of truth).
    pub(crate) fn restore_permanent_spills(&mut self) {
        for record in self.records.values_mut() {
            if record.status == SpillStatus::PermanentReloaded {
                record.status = SpillStatus::Permanent;
            }
        }
    }

    /// Mark a spilled value as reloaded into a register, without freeing the spill slot.
    ///
    /// - `Transient` → `TransientReloaded`
    /// - `Permanent` → `PermanentReloaded`
    ///
    /// The slot's data must remain valid because the reload code may execute
    /// again in a loop iteration. Only `remove_spill` (used when the value is
    /// truly dead) should free the slot.
    pub(crate) fn unmark_spilled(&mut self, value_id: &ValueId) {
        let record = self.records.get_mut(value_id).expect("No record for value");
        record.status = match record.status {
            SpillStatus::Transient => SpillStatus::TransientReloaded,
            SpillStatus::Permanent => SpillStatus::PermanentReloaded,
            _ => panic!("Expected record not to be spilled, but was {:?}", record.status),
        };
    }

    /// Promote an existing spill record to `Permanent`: the heap slot becomes the value's
    /// authoritative copy, so it is also dropped from the LRU (an in-memory value must never
    /// be an eviction victim).
    ///
    /// This updates spill bookkeeping only. A `*Reloaded` value still physically occupies a
    /// register at this point; whether that register is freed or kept alive is the caller's
    /// decision — see `BrilligBlock::spill_value`.
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
        record.status = SpillStatus::Permanent;
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

    use super::{Lru, SpillManager, SpillStatus};

    fn test_var(n: u32) -> BrilligVariable {
        BrilligVariable::SingleAddr(SingleAddrVariable::new(MemoryAddress::relative(n), 32))
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
        let v0 = Id::test_new(0);
        let v1 = Id::test_new(1);
        let v2 = Id::test_new(2);

        // Touch v0, v1, v2 in order. LRU order: [v0, v1, v2]
        sm.touch(v0);
        sm.touch(v1);
        sm.touch(v2);

        // Victim should be v0 (least recently used)
        assert_eq!(sm.lru_victim(), Some(v0));

        // Touch v0 again, making it most recent. LRU order: [v1, v2, v0]
        sm.touch(v0);

        // Victim should now be v1
        assert_eq!(sm.lru_victim(), Some(v1));
    }

    #[test]
    fn lru_victims_returns_the_k_least_recently_used() {
        let mut sm = SpillManager::new();
        let v0 = Id::test_new(0);
        let v1 = Id::test_new(1);
        let v2 = Id::test_new(2);
        let v3 = Id::test_new(3);

        sm.touch(v0);
        sm.touch(v1);
        sm.touch(v2);
        sm.touch(v3);

        // The `k` least-recently-used, oldest first.
        assert_eq!(sm.lru_victims(2), vec![v0, v1]);
        // `k` larger than the tracked set returns everything.
        assert_eq!(sm.lru_victims(10), vec![v0, v1, v2, v3]);
    }

    #[test]
    fn spilling_removes_value_from_lru_so_it_is_not_a_victim() {
        let mut sm = SpillManager::new();
        let v0 = Id::test_new(0);
        let v1 = Id::test_new(1);
        let v2 = Id::test_new(2);

        sm.touch(v0);
        sm.touch(v1);
        sm.touch(v2);

        // `record_spill` itself drops the value from the LRU.
        let offset = sm.allocate_spill_offset();
        sm.record_spill(v0, offset, test_var(0));

        assert!(sm.is_spilled(&v0));
        assert!(!sm.is_spilled(&v1));

        let record = sm.get_spill(&v0).unwrap();
        assert_eq!(record.offset, 0);
        assert_eq!(record.status, SpillStatus::Transient);

        // v0 is no longer tracked, so the next-oldest live value is the victim.
        assert_eq!(sm.lru_victim(), Some(v1));
    }

    #[test]
    #[should_panic(expected = "returned a spilled value")]
    fn lru_victim_rejects_a_spilled_oldest_value() {
        // Recording a spill drops the value from the LRU, so a spilled value can only re-enter
        // it through an erroneous `touch`. `lru_victim` must catch that rather than hand back a
        // value whose re-spill would free no register.
        let mut sm = SpillManager::new();
        let v0 = Id::test_new(0);

        let offset = sm.allocate_spill_offset();
        sm.record_spill(v0, offset, test_var(0));
        sm.touch(v0);

        let _ = sm.lru_victim();
    }

    #[test]
    fn spill_slot_allocation_and_reuse() {
        let mut sm = SpillManager::new();
        let v0 = Id::test_new(0);
        let v1 = Id::test_new(1);

        // Allocate two slots: offsets 0 and 1
        let off0 = sm.allocate_spill_offset();
        sm.record_spill(v0, off0, test_var(0));
        let off1 = sm.allocate_spill_offset();
        sm.record_spill(v1, off1, test_var(1));
        assert_eq!(off0, 0);
        assert_eq!(off1, 1);

        // Free slot 0 by removing the spill
        sm.remove_spill(&v0);
        assert!(!sm.is_spilled(&v0));

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
        let v0 = Id::test_new(0);
        let v1 = Id::test_new(1);
        let v2 = Id::test_new(2);

        sm.touch(v0);
        sm.touch(v1);
        sm.touch(v2);

        // Remove v1 from LRU entirely. LRU order: [v0, v2]
        sm.remove_from_lru(&v1);

        // Victim should be v0 (v1 is absent)
        assert_eq!(sm.lru_victim(), Some(v0));

        // Touch v0, making it most recent. LRU order: [v2, v0]
        sm.touch(v0);

        // Victim should be v2 (v1 is absent, v0 was touched)
        assert_eq!(sm.lru_victim(), Some(v2));
    }

    #[test]
    fn permanent_spill_lifecycle() {
        let mut sm = SpillManager::new();
        let v0 = Id::test_new(0);

        // Allocate and record a permanent spill
        let off = sm.allocate_spill_offset();
        sm.record_permanent_spill(v0, off, test_var(0));
        assert_eq!(off, 0);
        assert!(sm.is_spilled(&v0));
        assert!(sm.has_permanent_slot(&v0));
        assert_eq!(sm.get_permanent_spill_offset(&v0), Some(0));

        // Unmark spilled (reload) — record stays, slot stays
        sm.unmark_spilled(&v0);
        assert!(!sm.is_spilled(&v0));
        assert!(sm.has_permanent_slot(&v0));
        assert_eq!(sm.get_permanent_spill_offset(&v0), Some(0));

        // Restore permanent spills (block entry) — re-marks as spilled
        sm.restore_permanent_spills();
        assert!(sm.is_spilled(&v0));

        // Remove spill on a permanent record — keeps record, clears spilled
        sm.remove_spill(&v0);
        assert!(!sm.is_spilled(&v0));
        // Permanent record still exists
        assert!(sm.has_permanent_slot(&v0));
        // Slot is NOT freed (no slot in free list)
        assert!(sm.free_spill_slots.is_empty());
    }

    #[test]
    fn promote_transient_to_permanent() {
        let mut sm = SpillManager::new();
        let v0 = Id::test_new(0);

        // Record a transient spill
        let off = sm.allocate_spill_offset();
        sm.record_spill(v0, off, test_var(0));
        assert!(sm.is_spilled(&v0));
        assert!(!sm.has_permanent_slot(&v0));

        // Promote to permanent via ensure_permanent_spill
        assert!(sm.ensure_permanent_spill(&v0));
        assert!(sm.is_spilled(&v0));
        assert!(sm.has_permanent_slot(&v0));
        assert_eq!(sm.get_permanent_spill_offset(&v0), Some(0));

        // Removing a promoted permanent spill should NOT free the slot
        sm.remove_spill(&v0);
        assert!(!sm.is_spilled(&v0));
        assert!(sm.free_spill_slots.is_empty());
    }

    #[test]
    fn ensure_permanent_spill_for_not_spilled() {
        let mut sm = SpillManager::new();
        let v0 = Id::test_new(0);

        // Record a transient spill
        let off = sm.allocate_spill_offset();
        sm.record_spill(v0, off, test_var(0));
        sm.unmark_spilled(&v0);

        assert!(sm.ensure_permanent_spill(&v0), "expect true because the record exists");
    }

    #[test]
    #[should_panic(expected = "Transient spill leaked across block boundary")]
    fn begin_block_panics_on_transient_spill_leak() {
        let mut sm = SpillManager::new();
        let v0 = Id::test_new(0);

        // Record a transient spill (not permanent), leave it currently spilled
        let off = sm.allocate_spill_offset();
        sm.record_spill(v0, off, test_var(0));

        // begin_block should panic because a transient spill is still active
        let mut live_in = rustc_hash::FxHashSet::default();
        sm.begin_block(&mut live_in);
    }

    #[test]
    #[should_panic(expected = "Double-spill")]
    fn record_spill_panics_on_double_spill() {
        let mut sm = SpillManager::new();
        let v0 = Id::test_new(0);

        let off = sm.allocate_spill_offset();
        sm.record_spill(v0, off, test_var(0));

        // Attempting to spill v0 again without unmark/remove should panic
        let off2 = sm.allocate_spill_offset();
        sm.record_spill(v0, off2, test_var(0));
    }

    #[test]
    #[should_panic(expected = "orphaned existing slot")]
    fn record_spill_panics_on_orphaned_slot() {
        let mut sm = SpillManager::new();
        let v0 = Id::test_new(0);

        let off = sm.allocate_spill_offset();
        sm.record_spill(v0, off, test_var(0));

        sm.unmark_spilled(&v0);

        assert_eq!(sm.get_spill_offset(&v0), Some(off));

        // Attempting to spill v0 again with a different offset should panic.
        let off2 = sm.allocate_spill_offset();
        sm.record_spill(v0, off2, test_var(0));
    }

    #[test]
    fn record_spill_of_unmarked_with_same_offset() {
        let mut sm = SpillManager::new();
        let v0 = Id::test_new(0);

        let off = sm.allocate_spill_offset();
        sm.record_spill(v0, off, test_var(0));
        sm.unmark_spilled(&v0);
        sm.record_spill(v0, off, test_var(1)); // Different BrilligVariable to show it's not checked.
    }

    #[test]
    fn ensure_permanent_spill_all_branches() {
        let mut sm = SpillManager::new();
        let v0 = Id::test_new(0);
        let v1 = Id::test_new(1);
        let v2 = Id::test_new(2);
        let v3 = Id::test_new(3);

        // Case 1: No record -> returns false
        assert!(!sm.ensure_permanent_spill(&v3));

        // Case 2: Permanent -> returns true, stays Permanent
        let off0 = sm.allocate_spill_offset();
        sm.record_permanent_spill(v0, off0, test_var(0));
        assert_eq!(sm.records[&v0].status, SpillStatus::Permanent);
        assert!(sm.ensure_permanent_spill(&v0));
        assert_eq!(sm.records[&v0].status, SpillStatus::Permanent);

        // Case 3: Transient -> returns true, promoted to Permanent
        let off1 = sm.allocate_spill_offset();
        sm.record_spill(v1, off1, test_var(1));
        assert_eq!(sm.records[&v1].status, SpillStatus::Transient);
        assert!(sm.ensure_permanent_spill(&v1));
        assert_eq!(sm.records[&v1].status, SpillStatus::Permanent);

        // Case 4: PermanentReloaded -> returns true, re-marked as Permanent
        let off2 = sm.allocate_spill_offset();
        sm.record_permanent_spill(v2, off2, test_var(2));
        sm.unmark_spilled(&v2);
        assert_eq!(sm.records[&v2].status, SpillStatus::PermanentReloaded);
        assert!(sm.ensure_permanent_spill(&v2));
        assert_eq!(sm.records[&v2].status, SpillStatus::Permanent);
    }

    /// Promoting a `PermanentReloaded` value back to `Permanent` must drop it from the LRU,
    /// otherwise a spilled value lingers as an eviction candidate. In real codegen this
    /// happens for a JmpIf condition that is re-spilled by `spill_non_param_live_ins` and
    /// then reloaded, with `ensure_register_capacity` querying `lru_victim` in between.
    #[test]
    fn ensure_permanent_spill_drops_reloaded_value_from_lru() {
        let mut sm = SpillManager::new();
        let v0 = Id::test_new(0);

        let off = sm.allocate_spill_offset();
        sm.record_permanent_spill(v0, off, test_var(0));
        sm.unmark_spilled(&v0); // PermanentReloaded: currently in a register
        sm.touch(v0); // tracked in the LRU

        assert!(!sm.is_transient_reloaded(&v0));
        assert!(sm.ensure_permanent_spill(&v0));
        assert!(sm.is_spilled(&v0));

        // The value is spilled again, so it must no longer be an eviction candidate.
        assert_eq!(sm.lru_victim(), None);
    }
}
