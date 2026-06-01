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

use std::collections::hash_map::Entry;

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

/// Tracks register values that have been spilled to the spill region in heap memory.
///
/// See the [module docs][self] for an overview of when and how the spill manager is used.
pub(crate) struct SpillManager {
    /// Map of all spill records
    records: HashMap<ValueId, SpillRecord>,
    /// LRU order: front = least recently used, back = most recently used.
    /// Only tracks local SSA values (not globals, not hoisted constants).
    lru_order: Vec<ValueId>,
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

impl SpillManager {
    pub(crate) fn new() -> Self {
        Self {
            records: HashMap::default(),
            lru_order: Vec::new(),
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
    /// 4. Updates the LRU: retains existing entries still live-in and not spilled
    ///    (preserving eviction hints from the previous block), then appends any
    ///    new live-in values sorted by [ValueId] for determinism.
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
        matches!(
            self.records.get(value_id),
            Some(r) if matches!(r.status, SpillStatus::Transient | SpillStatus::Permanent)
        )
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
        self.remove_from_lru(&value_id);
        self.lru_order.push(value_id);
    }

    /// Remove a value from LRU tracking entirely.
    pub(crate) fn remove_from_lru(&mut self, value_id: &ValueId) {
        if let Some(pos) = self.lru_order.iter().position(|v| v == value_id) {
            self.lru_order.remove(pos);
        }
    }

    /// Remove a value from the spill tracking.
    ///
    /// Permanent spill slots are never freed — they must remain valid across all blocks.
    /// For `Permanent` records, this transitions to `PermanentReloaded` (value is live in
    /// a register again). For transient records, the record is removed and the slot is freed.
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

    /// Return the least recently used value that is not currently spilled.
    /// Returns None if there are no eligible values in the LRU.
    ///
    /// Note: permanently-spilled values may appear in the LRU if they were
    /// reloaded (which calls `touch()`). This is correct — a reloaded value
    /// has a register and is a valid eviction candidate.
    pub(crate) fn lru_victim(&self) -> Option<ValueId> {
        for value_id in &self.lru_order {
            if !self.is_spilled(value_id) {
                return Some(*value_id);
            }
        }
        None
    }

    /// Record that a value has been spilled.
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
    }

    /// Get the spill record for a value if it is currently spilled.
    pub(crate) fn get_spill(&self, value_id: &ValueId) -> Option<&SpillRecord> {
        self.records
            .get(value_id)
            .filter(|r| matches!(r.status, SpillStatus::Transient | SpillStatus::Permanent))
    }

    /// Reset the LRU for a new block, retaining ordering from the previous block.
    ///
    /// Entries already in `lru_order` that are still live-in and not spilled are kept
    /// in their existing order (preserving eviction hints from the previous block).
    /// New live-in values not yet in the LRU are appended, sorted by [ValueId] for
    /// determinism.
    fn reset_lru_for_block(&mut self, live_in: &HashSet<ValueId>) {
        let records = &self.records;
        let is_spilled = |v: &ValueId| {
            matches!(
                records.get(v),
                Some(r) if matches!(r.status, SpillStatus::Transient | SpillStatus::Permanent)
            )
        };

        // Retain existing entries that are still live-in and not spilled.
        self.lru_order.retain(|v| live_in.contains(v) && !is_spilled(v));

        // Collect live-in values not already present in LRU, sorted for determinism.
        let existing: HashSet<ValueId> = self.lru_order.iter().copied().collect();
        let mut new_entries: Vec<ValueId> =
            live_in.iter().copied().filter(|v| !existing.contains(v) && !is_spilled(v)).collect();
        new_entries.sort();
        self.lru_order.extend(new_entries);
    }

    /// Record a value as permanently spilled.
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

    /// Ensure a value has a permanent spill slot.
    ///
    /// Any existing record — regardless of its current state — is promoted to
    /// `Permanent` (the slot is the source of truth and the value is not in a register).
    ///
    /// # Returns
    /// * `true` if a record already existed (caller should skip further processing),
    /// * `false` if no record exists (first encounter — caller must allocate a slot).
    pub(crate) fn ensure_permanent_spill(&mut self, value_id: &ValueId) -> bool {
        let Some(record) = self.records.get_mut(value_id) else {
            return false;
        };
        record.status = SpillStatus::Permanent;
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

    use super::{SpillManager, SpillStatus};

    fn test_var(n: u32) -> BrilligVariable {
        BrilligVariable::SingleAddr(SingleAddrVariable::new(MemoryAddress::relative(n), 32))
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
    fn spill_record_and_victim_skips_spilled() {
        let mut sm = SpillManager::new();
        let v0 = Id::test_new(0);
        let v1 = Id::test_new(1);
        let v2 = Id::test_new(2);

        sm.touch(v0);
        sm.touch(v1);
        sm.touch(v2);

        // Spill v0 (the LRU victim)
        let offset = sm.allocate_spill_offset();
        sm.record_spill(v0, offset, test_var(0));

        assert!(sm.is_spilled(&v0));
        assert!(!sm.is_spilled(&v1));

        let record = sm.get_spill(&v0).unwrap();
        assert_eq!(record.offset, 0);
        assert_eq!(record.status, SpillStatus::Transient);

        // Victim should skip v0 (spilled) and return v1
        assert_eq!(sm.lru_victim(), Some(v1));
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
}
