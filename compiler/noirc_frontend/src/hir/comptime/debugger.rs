use fm::FileMap;
use imbl::Vector;
use noirc_errors::Location;

use crate::node_interner::{FuncId, NodeInterner};

/// Callback for debugging the comptime interpreter.
/// Called at each statement boundary during interpretation.
pub trait ComptimeDebugger {
    /// Called before each statement is evaluated.
    /// Receives the current execution state for inspection.
    fn on_statement(
        &mut self,
        location: Location,
        interner: &NodeInterner,
        files: &FileMap,
        call_stack: &Vector<Location>,
        current_function: Option<FuncId>,
        call_stack_functions: &Vector<Option<FuncId>>,
    );
}
