use std::{
    collections::{HashSet, VecDeque},
    hash::Hash,
};

use crate::ssa::ir::basic_block::BasicBlockId;

/// A wrapper around `VecDeque` to ensure that we never process the same item more than once.
#[derive(Debug)]
pub(crate) struct VisitOnceDeque<T = BasicBlockId> {
    visited_blocks: HashSet<T>,
    block_queue: VecDeque<T>,
}

impl<T: Hash + Eq + Copy> VisitOnceDeque<T> {
    pub(crate) fn new() -> Self {
        Self { visited_blocks: HashSet::new(), block_queue: VecDeque::new() }
    }

    pub(crate) fn push_back(&mut self, item: T) {
        self.block_queue.push_back(item);
    }

    pub(crate) fn extend(&mut self, items: impl IntoIterator<Item = T>) {
        self.block_queue.extend(items);
    }

    /// Skipping already-visited items is done with a loop rather than a recursive call: the queue
    /// can hold an arbitrarily long run of duplicates (a `[v; N]` array literal enqueues one value
    /// N times), and one stack frame per skipped item overflows the stack on large N.
    pub(crate) fn pop_front(&mut self) -> Option<T> {
        while let Some(item) = self.block_queue.pop_front() {
            if self.visited_blocks.insert(item) {
                return Some(item);
            }
        }
        None
    }

    pub(crate) fn pop_back(&mut self) -> Option<T> {
        while let Some(item) = self.block_queue.pop_back() {
            if self.visited_blocks.insert(item) {
                return Some(item);
            }
        }
        None
    }
}

impl<T: Hash + Eq + Copy> Default for VisitOnceDeque<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::ssa::visit_once_deque::VisitOnceDeque;

    #[test]
    fn does_not_return_duplicates() {
        let mut deque = VisitOnceDeque::default();
        deque.extend([0, 1, 2, 0, 1]);

        assert_eq!(deque.pop_front(), Some(0));
        assert_eq!(deque.pop_back(), Some(1));
        assert_eq!(deque.pop_front(), Some(2));
        assert_eq!(deque.pop_front(), None);
        assert_eq!(deque.pop_back(), None);
    }

    #[test]
    fn draining_a_long_run_of_duplicates_does_not_consume_stack() {
        let mut deque = VisitOnceDeque::default();
        deque.extend(std::iter::repeat_n(7u32, 1_000_000));

        assert_eq!(deque.pop_front(), Some(7));
        assert_eq!(deque.pop_front(), None);
    }
}
