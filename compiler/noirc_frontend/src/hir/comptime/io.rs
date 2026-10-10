use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use super::EvaluationTracker;

/// What running comptime code may write to, besides the program being elaborated.
///
/// It is handed to each operation that runs comptime code, so the caller decides per operation
/// where the output goes and whether evaluations are tracked.
#[derive(Default)]
pub struct ComptimeIo {
    /// Receives whatever comptime code prints. `None` discards it.
    pub output: Option<Rc<RefCell<dyn Write>>>,

    /// Records the locations of the comptime expressions which are evaluated, to facilitate code
    /// coverage. `None` records nothing.
    pub evaluation_tracker: Option<EvaluationTracker>,
}

impl ComptimeIo {
    /// Prints to stdout and tracks nothing.
    pub fn stdout() -> Self {
        Self::printing_to(Rc::new(RefCell::new(std::io::stdout())))
    }

    /// Discards prints and tracks nothing.
    pub fn silent() -> Self {
        Self::default()
    }

    /// Prints to `output` and tracks nothing.
    pub fn printing_to(output: Rc<RefCell<dyn Write>>) -> Self {
        Self { output: Some(output), evaluation_tracker: None }
    }
}
