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
}

#[derive(Clone)]
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
    /// Returns None if there are no unspilled values in the LRU.
    pub(crate) fn lru_victim(&self) -> Option<ValueId> {
        // Walk from front (LRU) to find a value that's not already spilled
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
}
