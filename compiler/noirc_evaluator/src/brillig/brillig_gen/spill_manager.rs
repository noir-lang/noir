use rustc_hash::FxHashMap as HashMap;

use crate::brillig::brillig_ir::brillig_variable::BrilligVariable;
use crate::ssa::ir::value::ValueId;

/// Tracks register values that have been spilled to the spill region in heap memory.
///
/// When register pressure exceeds the stack frame limit, the SpillManager coordinates
/// evicting the least-recently-used values from registers to the spill region,
/// and reloading them when needed.
pub(crate) struct SpillManager {
    /// SSA values currently spilled to the spill region.
    spilled: HashMap<ValueId, SpillInfo>,
    /// LRU order: front = least recently used, back = most recently used.
    /// Only tracks local SSA values (not globals, not hoisted constants).
    lru_order: Vec<ValueId>,
    /// Next offset within the spill region (relative to spill base register).
    next_spill_offset: usize,
    /// Free list of spill slots that have been reclaimed.
    free_spill_slots: Vec<usize>,
    /// Permanent spill slots for successor block params. These params are
    /// eagerly spilled in the dominator block and must always be accessed
    /// through the spill slot to ensure consistency across all Jmp sites
    /// and the target block's reload code.
    successor_param_spills: HashMap<ValueId, SpillInfo>,
}

#[derive(Clone, Copy)]
pub(crate) struct SpillInfo {
    /// Offset relative to the per-frame heap-allocated spill region base.
    pub(crate) offset: usize,
    /// Original variable (for type/register info on reload).
    pub(crate) variable: BrilligVariable,
}

impl SpillManager {
    pub(crate) fn new() -> Self {
        Self {
            spilled: HashMap::default(),
            lru_order: Vec::new(),
            next_spill_offset: 0,
            free_spill_slots: Vec::new(),
            successor_param_spills: HashMap::default(),
        }
    }

    /// Check if a value is currently spilled.
    pub(crate) fn is_spilled(&self, value_id: &ValueId) -> bool {
        self.spilled.contains_key(value_id)
    }

    /// Move a value to the back of the LRU (most recently used).
    /// If the value isn't tracked yet, add it.
    pub(crate) fn touch(&mut self, value_id: ValueId) {
        if let Some(pos) = self.lru_order.iter().position(|v| *v == value_id) {
            self.lru_order.remove(pos);
        }
        self.lru_order.push(value_id);
    }

    /// Remove a value from LRU tracking entirely.
    pub(crate) fn remove_from_lru(&mut self, value_id: &ValueId) {
        if let Some(pos) = self.lru_order.iter().position(|v| v == value_id) {
            self.lru_order.remove(pos);
        }
    }

    /// Remove a value from the spill map, reclaiming its slot for reuse.
    pub(crate) fn remove_spill(&mut self, value_id: &ValueId) {
        if let Some(info) = self.spilled.remove(value_id) {
            self.free_spill_slots.push(info.offset);
        }
    }

    /// Allocate a spill slot offset, reusing a freed slot if available.
    pub(crate) fn allocate_spill_offset(&mut self) -> usize {
        if let Some(offset) = self.free_spill_slots.pop() {
            offset
        } else {
            let offset = self.next_spill_offset;
            self.next_spill_offset += 1;
            offset
        }
    }

    /// Return the least recently used value that is not currently spilled.
    /// Returns None if there are no eligible values in the LRU.
    pub(crate) fn lru_victim(&self) -> Option<ValueId> {
        for value_id in &self.lru_order {
            if !self.spilled.contains_key(value_id) {
                return Some(*value_id);
            }
        }
        None
    }

    /// Record that a value has been spilled.
    pub(crate) fn record_spill(
        &mut self,
        value_id: ValueId,
        offset: usize,
        variable: BrilligVariable,
    ) {
        self.spilled.insert(value_id, SpillInfo { offset, variable });
    }

    /// Get the spill info for a value.
    pub(crate) fn get_spill(&self, value_id: &ValueId) -> Option<&SpillInfo> {
        self.spilled.get(value_id)
    }

    /// Reset the LRU for a new block, populating it with live-in values that are not spilled.
    ///
    /// The LRU is inherently per-block (tracks which in-register values can be evicted),
    /// so resetting at block entry is correct. As instructions execute, `touch()` establishes
    /// real usage order.
    pub(crate) fn reset_lru_for_block(&mut self, live_in: impl Iterator<Item = ValueId>) {
        self.lru_order.clear();
        for value_id in live_in {
            if !self.spilled.contains_key(&value_id) {
                self.lru_order.push(value_id);
            }
        }
    }

    /// Record a successor block param as eagerly spilled.
    ///
    /// These params are always accessed through the spill slot: every Jmp writes
    /// to the slot, and every target block reloads from it. The permanent record
    /// in `successor_param_spills` ensures this consistency regardless of what
    /// happens to the transient `spilled` map during block processing.
    pub(crate) fn record_successor_param_spill(
        &mut self,
        value_id: ValueId,
        offset: usize,
        variable: BrilligVariable,
    ) {
        let info = SpillInfo { offset, variable };
        self.spilled.insert(value_id, info);
        self.successor_param_spills.insert(value_id, info);
    }

    /// Get the spill slot offset for a successor block param, if any.
    pub(crate) fn get_successor_param_spill_offset(&self, value_id: &ValueId) -> Option<usize> {
        self.successor_param_spills.get(value_id).map(|info| info.offset)
    }

    /// Re-mark eagerly-spilled successor params as spilled at block entry.
    ///
    /// A reload in a previous block removes the value from `spilled`, but the
    /// value must be reloaded again in every block that uses it (since the Jmp
    /// always writes to the spill slot, not to a register).
    pub(crate) fn restore_successor_param_spills(&mut self) {
        for (value_id, info) in &self.successor_param_spills {
            if !self.spilled.contains_key(value_id) {
                self.spilled.insert(*value_id, *info);
            }
        }
    }

    /// Remove a value from the spilled map WITHOUT freeing the spill slot.
    ///
    /// This is used when reloading a spilled value. The slot's data must remain valid
    /// because the reload code may execute again in a loop iteration. Only `remove_spill`
    /// (used when the value is truly dead) should free the slot.
    pub(crate) fn unmark_spilled(&mut self, value_id: &ValueId) {
        self.spilled.remove(value_id);
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

        let info = sm.get_spill(&v0).unwrap();
        assert_eq!(info.offset, 0);

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
}
