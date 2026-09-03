use fm::FileMap;
use imbl::Vector;
use noirc_errors::Location;

use crate::Type;
use crate::node_interner::{FuncId, NodeInterner};

use super::errors::InterpreterError;
use super::value::Value;

/// Snapshot of the interpreter's execution state, passed to the debugger
/// at each statement boundary.
pub struct DebugContext<'a> {
    pub location: Location,
    pub interner: &'a NodeInterner,
    pub files: &'a FileMap,
    pub call_stack: &'a Vector<Location>,
    pub current_function: Option<FuncId>,
    pub call_stack_functions: &'a Vector<Option<FuncId>>,
}

/// Callback for debugging the comptime interpreter.
/// Called at each statement boundary during interpretation.
pub trait ComptimeDebugger {
    /// Called before each statement is evaluated.
    fn on_statement(&mut self, context: DebugContext<'_>);
}

/// Executor for oracle (foreign) calls during comptime interpretation.
/// Bridges comptime `Value` to the existing `ForeignCallExecutor` infrastructure.
pub trait ComptimeOracleExecutor {
    fn execute_oracle(
        &mut self,
        name: &str,
        arguments: Vec<Value>,
        return_type: &Type,
        location: Location,
    ) -> Result<Value, InterpreterError>;
}
