use std::collections::BTreeSet;

/// A priority queue that serves the next item with the lowest or highest priority
/// and never serves the same item twice.
#[derive(Debug)]
pub(crate) struct VisitOncePriorityQueue<P, T> {
    visited: BTreeSet<T>,
    queue: BTreeSet<(P, T)>,
}

impl<P: Ord, T: Ord + Copy> VisitOncePriorityQueue<P, T> {
    pub(crate) fn new() -> Self {
        Self { visited: BTreeSet::new(), queue: BTreeSet::new() }
    }

    pub(crate) fn push(&mut self, priority: P, item: T) {
        if !self.visited.contains(&item) {
            self.queue.insert((priority, item));
        }
    }

    #[allow(unused)]
    pub(crate) fn extend(&mut self, items: impl IntoIterator<Item = (P, T)>) {
        for (p, i) in items {
            self.push(p, i);
        }
    }

    pub(crate) fn pop_front(&mut self) -> Option<T> {
        let (_, item) = self.queue.pop_first()?;
        if self.visited.insert(item) { Some(item) } else { self.pop_front() }
    }

    #[allow(unused)]
    pub(crate) fn pop_back(&mut self) -> Option<T> {
        let (_, item) = self.queue.pop_last()?;
        if self.visited.insert(item) { Some(item) } else { self.pop_back() }
    }
}

impl<P: Ord, T: Ord + Copy> Default for VisitOncePriorityQueue<P, T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::ssa::visit_once_priority_queue::VisitOncePriorityQueue;

    #[test]
    fn does_not_return_duplicates() {
        let mut deque = VisitOncePriorityQueue::default();
        deque.extend([(0, "Foo"), (0, "Bar"), (3, "Bar"), (2, "Baz"), (1, "Qux")]);

        assert_eq!(deque.pop_front(), Some("Bar"));
        assert_eq!(deque.pop_back(), Some("Baz"));
        assert_eq!(deque.pop_front(), Some("Foo"));
        assert_eq!(deque.pop_front(), Some("Qux"));
        assert_eq!(deque.pop_back(), None);
    }
}
