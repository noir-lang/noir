//! Utility to help catch stack overflows before they happen.

const MIN_REMAINING_STACK_PERCENT: usize = 10;

pub(crate) struct StackLimiter {
    /// Remaining stack depth when the tracker was created.
    start: Option<usize>,
}

impl StackLimiter {
    pub(crate) fn new() -> Self {
        Self { start: stacker::remaining_stack() }
    }

    /// Check if it's okay to recurse more.
    ///
    /// Returns false if we have less than 10 percent of the stack left.
    pub(crate) fn can_recur(&self) -> bool {
        self.start
            .filter(|s| *s > 0)
            .and_then(|s| {
                stacker::remaining_stack().map(|r| r * 100 / s >= MIN_REMAINING_STACK_PERCENT)
            })
            .unwrap_or(true)
    }
}

impl Default for StackLimiter {
    fn default() -> Self {
        Self::new()
    }
}
