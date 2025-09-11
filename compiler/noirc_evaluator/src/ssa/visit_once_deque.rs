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

#[cfg(test)]
mod tests {
    use crate::ssa::{ir::basic_block::BasicBlockId, visit_once_deque::VisitOnceDeque};

    #[test]
    fn does_not_return_duplicates() {
        let mut deque = VisitOnceDeque::default();
        deque.extend([
            BasicBlockId::test_new(0),
            BasicBlockId::test_new(1),
            BasicBlockId::test_new(2),
            BasicBlockId::test_new(0),
            BasicBlockId::test_new(1),
        ]);

        assert_eq!(deque.pop_front(), Some(BasicBlockId::test_new(0)));
        assert_eq!(deque.pop_back(), Some(BasicBlockId::test_new(1)));
        assert_eq!(deque.pop_front(), Some(BasicBlockId::test_new(2)));
        assert_eq!(deque.pop_front(), None);
        assert_eq!(deque.pop_back(), None);
    }
}
