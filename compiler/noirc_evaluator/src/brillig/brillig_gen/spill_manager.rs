use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::brillig::brillig_ir::brillig_variable::BrilligVariable;
use crate::ssa::ir::value::ValueId;

/// Tracks register values that have been spilled to the spill region in heap memory.
///
/// When register pressure exceeds the stack frame limit, the SpillManager coordinates
/// evicting the least-recently-used values from registers to the spill region,
/// and reloading them when needed.
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
    /// Whether this spill slot is permanent (must survive across blocks).
    /// Otherwise, the spill slot is transient and can be re-used.
    pub(crate) is_permanent: bool,
    /// Whether the value is currently spilled (has no valid register).
    pub(crate) is_currently_spilled: bool,
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
    ///    (all currently-spilled entries must also be permanent).
    /// 2. Restores permanently-spilled values by marking them as currently spilled.
    /// 3. Removes spilled values from the live-in set (they have no register).
    /// 4. Updates the LRU: retains existing entries still live-in and not spilled
    ///    (preserving eviction hints from the previous block), then appends any
    ///    new live-in values sorted by [ValueId] for determinism.
    pub(crate) fn begin_block(&mut self, live_in: &mut HashSet<ValueId>) {
        // No transient spills should survive across block boundaries.
        assert!(
            self.records.values().all(|r| !r.is_currently_spilled || r.is_permanent),
            "Transient spill leaked across block boundary"
        );
        self.restore_permanent_spills();
        live_in.retain(|v| !self.is_spilled(v));
        self.reset_lru_for_block(live_in);
    }

    /// Check if a value is currently spilled.
    pub(crate) fn is_spilled(&self, value_id: &ValueId) -> bool {
        self.records.get(value_id).is_some_and(|r| r.is_currently_spilled)
    }

    /// Check if a value has a permanent spill slot.
    #[cfg(test)]
    pub(crate) fn has_permanent_slot(&self, value_id: &ValueId) -> bool {
        self.records.get(value_id).is_some_and(|r| r.is_permanent)
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
    /// For permanent records, this just clears `is_currently_spilled`.
    /// For transient records, the record is removed entirely and the slot is freed.
    ///
    /// TODO(<https://github.com/noir-lang/noir/issues/11695>) - Free globally dead permanent spill slots
    pub(crate) fn remove_spill(&mut self, value_id: &ValueId) {
        if let std::collections::hash_map::Entry::Occupied(mut entry) =
            self.records.entry(*value_id)
        {
            if entry.get().is_permanent {
                entry.get_mut().is_currently_spilled = false;
            } else {
                let record = entry.remove();
                self.free_spill_slots.push(record.offset);
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
    /// - Permanent records: just re-marks as currently spilled, preserving the
    ///   permanent offset. The slot already has valid data when operating over SSA where variables are immutable.
    /// - Transient records: updates the offset and variable to the new values.
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
            is_permanent: false,
            is_currently_spilled: true,
        });
        // Always preserve the offset, so we don't leak free slots.
        assert_eq!(offset, record.offset, "Spill of {value_id} orphaned existing slot");
        record.is_currently_spilled = true;
        if !record.is_permanent {
            // Transient record from a previous spill/reload cycle — update it.
            record.variable = variable;
        }
    }

    /// Get the spill record for a value if it is currently spilled.
    pub(crate) fn get_spill(&self, value_id: &ValueId) -> Option<&SpillRecord> {
        self.records.get(value_id).filter(|r| r.is_currently_spilled)
    }

    /// Reset the LRU for a new block, retaining ordering from the previous block.
    ///
    /// Entries already in `lru_order` that are still live-in and not spilled are kept
    /// in their existing order (preserving eviction hints from the previous block).
    /// New live-in values not yet in the LRU are appended, sorted by [ValueId] for
    /// determinism.
    fn reset_lru_for_block(&mut self, live_in: &HashSet<ValueId>) {
        let records = &self.records;
        let is_spilled = |v: &ValueId| records.get(v).is_some_and(|r| r.is_currently_spilled);

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
    /// If the value already has a transient spill record, it is promoted to permanent.
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
                record.is_permanent = true;
                record.is_currently_spilled = true;
                record.variable = variable;
            })
            .or_insert(SpillRecord {
                offset,
                variable,
                is_permanent: true,
                is_currently_spilled: true,
            });
    }

    /// Get the permanent spill slot offset for a value, if any.
    pub(crate) fn get_permanent_spill_offset(&self, value_id: &ValueId) -> Option<usize> {
        self.records.get(value_id).filter(|r| r.is_permanent).map(|r| r.offset)
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
    /// A reload in a previous block clears `is_currently_spilled`, but the
    /// value must be reloaded again in every block that uses it (since the
    /// permanent spill slot is always the source of truth).
    pub(crate) fn restore_permanent_spills(&mut self) {
        for record in self.records.values_mut() {
            if record.is_permanent {
                record.is_currently_spilled = true;
            }
        }
    }

    /// Clear the `is_currently_spilled` flag WITHOUT freeing the spill slot.
    ///
    /// This is used when reloading a spilled value. The slot's data must remain valid
    /// because the reload code may execute again in a loop iteration. Only `remove_spill`
    /// (used when the value is truly dead) should free the slot.
    pub(crate) fn unmark_spilled(&mut self, value_id: &ValueId) {
        if let Some(record) = self.records.get_mut(value_id) {
            record.is_currently_spilled = false;
        }
    }

    /// Ensure a value has a permanent spill slot.
    ///
    /// Handles all three cases where a record already exists with a single lookup:
    /// - Permanent + currently spilled -> no-op (slot has correct data)
    /// - Currently spilled but not permanent -> promote to permanent
    /// - Permanent but not currently spilled (reloaded) -> re-mark as spilled
    ///
    /// # Returns
    /// `true` if a record existed (caller should skip further processing),
    /// `false` if no record exists (first encounter — caller must allocate a slot).
    pub(crate) fn ensure_permanent_spill(&mut self, value_id: &ValueId) -> bool {
        if let Some(record) = self.records.get_mut(value_id) {
            if record.is_permanent && record.is_currently_spilled {
                // Already permanent and spilled — nothing to do.
            } else if record.is_currently_spilled {
                // Transient spill — promote to permanent.
                record.is_permanent = true;
            } else if record.is_permanent {
                // Permanent but reloaded — re-mark as spilled.
                record.is_currently_spilled = true;
            }
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use acvm::acir::brillig::MemoryAddress;

    use crate::{
        brillig::brillig_ir::brillig_variable::{BrilligVariable, SingleAddrVariable},
        ssa::ir::map::Id,
    };

    use super::SpillManager;

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
        assert!(!record.is_permanent);
        assert!(record.is_currently_spilled);

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

        // Case 2: Permanent + currently spilled -> returns true, no state change
        let off0 = sm.allocate_spill_offset();
        sm.record_permanent_spill(v0, off0, test_var(0));
        assert!(sm.is_spilled(&v0));
        assert!(sm.has_permanent_slot(&v0));
        assert!(sm.ensure_permanent_spill(&v0));
        // State unchanged
        assert!(sm.is_spilled(&v0));
        assert!(sm.has_permanent_slot(&v0));

        // Case 3: Transient spill (not permanent) -> returns true, promoted to permanent
        let off1 = sm.allocate_spill_offset();
        sm.record_spill(v1, off1, test_var(1));
        assert!(sm.is_spilled(&v1));
        assert!(!sm.has_permanent_slot(&v1));
        assert!(sm.ensure_permanent_spill(&v1));
        assert!(sm.is_spilled(&v1));
        assert!(sm.has_permanent_slot(&v1));

        // Case 4: Permanent but reloaded (not currently spilled) -> returns true, re-marked as spilled
        let off2 = sm.allocate_spill_offset();
        sm.record_permanent_spill(v2, off2, test_var(2));
        sm.unmark_spilled(&v2);
        assert!(!sm.is_spilled(&v2));
        assert!(sm.has_permanent_slot(&v2));
        assert!(sm.ensure_permanent_spill(&v2));
        assert!(sm.is_spilled(&v2));
        assert!(sm.has_permanent_slot(&v2));
    }
}
