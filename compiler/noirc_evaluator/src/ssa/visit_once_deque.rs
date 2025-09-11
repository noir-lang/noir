use std::collections::{HashSet, VecDeque};

use crate::ssa::ir::basic_block::BasicBlockId;

/// A wrapper around `VecDeque` to ensure that we never process the same [`BasicBlockId`] more than once.
#[derive(Debug, Default)]
pub(crate) struct VisitOnceDeque {
    visited_blocks: HashSet<BasicBlockId>,
    block_queue: VecDeque<BasicBlockId>,
}

impl VisitOnceDeque {
    pub(crate) fn push_back(&mut self, item: BasicBlockId) {
        self.block_queue.push_back(item);
    }

    pub(crate) fn extend<T: IntoIterator<Item = BasicBlockId>>(&mut self, items: T) {
        self.block_queue.extend(items);
    }

    pub(crate) fn pop_front(&mut self) -> Option<BasicBlockId> {
        let item = self.block_queue.pop_front()?;
        if self.visited_blocks.insert(item) { Some(item) } else { self.pop_front() }
    }

    pub(crate) fn pop_back(&mut self) -> Option<BasicBlockId> {
        let item = self.block_queue.pop_back()?;
        if self.visited_blocks.insert(item) { Some(item) } else { self.pop_back() }
    }
}
