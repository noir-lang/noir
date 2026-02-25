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
    /// 4. Resets the LRU with the remaining (non-spilled) live-in values,
    ///    sorted by [`ValueId`] for deterministic eviction order.
    pub(crate) fn begin_block(&mut self, live_in: &mut HashSet<ValueId>) {
        // No transient spills should survive across block boundaries.
        assert!(
            self.records.values().all(|r| !r.is_currently_spilled || r.is_permanent),
            "Transient spill leaked across block boundary"
        );
        self.restore_permanent_spills();
        live_in.retain(|v| !self.is_spilled(v));

        // Sort by ValueId for deterministic LRU initialization order.
        let mut sorted: Vec<ValueId> = live_in.iter().copied().collect();
        sorted.sort();
        self.reset_lru_for_block(sorted.into_iter());
    }

    /// Check if a value is currently spilled.
    pub(crate) fn is_spilled(&self, value_id: &ValueId) -> bool {
        self.records.get(value_id).is_some_and(|r| r.is_currently_spilled)
    }

    /// Check if a value has a permanent spill slot.
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
    pub(crate) fn remove_spill(&mut self, value_id: &ValueId) {
        if let Some(record) = self.records.get(value_id) {
            if record.is_permanent {
                // Keep the record but mark as not currently spilled.
                self.records.get_mut(value_id).unwrap().is_currently_spilled = false;
            } else {
                let offset = record.offset;
                self.records.remove(value_id);
                self.free_spill_slots.push(offset);
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
            if !self.records.get(value_id).is_some_and(|r| r.is_currently_spilled) {
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
        if record.is_permanent {
            // Permanent record — preserve offset, just re-mark as spilled.
            record.is_currently_spilled = true;
        } else {
            // Transient record from a previous spill/reload cycle — update it.
            record.offset = offset;
            record.variable = variable;
            record.is_currently_spilled = true;
        }
    }

    /// Get the spill record for a value if it is currently spilled.
    pub(crate) fn get_spill(&self, value_id: &ValueId) -> Option<&SpillRecord> {
        self.records.get(value_id).filter(|r| r.is_currently_spilled)
    }

    /// Reset the LRU for a new block, populating it with live-in values that are not spilled.
    ///
    /// The LRU is inherently per-block (tracks which in-register values can be evicted),
    /// so resetting at block entry is correct. As instructions execute, `touch()` establishes
    /// real usage order.
    fn reset_lru_for_block(&mut self, live_in: impl Iterator<Item = ValueId>) {
        self.lru_order.clear();
        for value_id in live_in {
            if !self.is_spilled(&value_id) {
                self.lru_order.push(value_id);
            }
        }
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

    /// Promote an existing transient spill to a permanent spill.
    ///
    /// The existing slot offset and variable are preserved; only the
    /// `is_permanent` flag is set. No store is needed — the data is already
    /// in the slot.
    pub(crate) fn promote_to_permanent(&mut self, value_id: &ValueId) {
        if let Some(record) = self.records.get_mut(value_id) {
            assert!(record.is_currently_spilled, "Cannot promote a non-spilled value to permanent");
            record.is_permanent = true;
        }
    }

    /// Re-mark a permanent record as currently spilled.
    ///
    /// Used when a value already has a permanent spill slot from a previous block
    /// but is not currently marked as spilled.
    pub(crate) fn re_mark_as_spilled(&mut self, value_id: &ValueId) {
        if let Some(record) = self.records.get_mut(value_id) {
            assert!(record.is_permanent, "Can only re-mark permanent spills");
            record.is_currently_spilled = true;
        }
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

        // Promote to permanent
        sm.promote_to_permanent(&v0);
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
    #[should_panic(expected = "Cannot promote a non-spilled value to permanent")]
    fn promote_panics_on_non_spilled_value() {
        let mut sm = SpillManager::new();
        let v0 = Id::test_new(0);

        // Record a transient spill, then unmark it (simulating a reload)
        let off = sm.allocate_spill_offset();
        sm.record_spill(v0, off, test_var(0));
        sm.unmark_spilled(&v0);

        // Promoting a non-spilled value should panic
        sm.promote_to_permanent(&v0);
    }

    #[test]
    #[should_panic(expected = "Can only re-mark permanent spills")]
    fn re_mark_panics_on_transient_record() {
        let mut sm = SpillManager::new();
        let v0 = Id::test_new(0);

        // Record a transient spill, then unmark it (simulating a reload)
        let off = sm.allocate_spill_offset();
        sm.record_spill(v0, off, test_var(0));
        sm.unmark_spilled(&v0);

        // Re-marking a transient (non-permanent) record should panic
        sm.re_mark_as_spilled(&v0);
    }
}
