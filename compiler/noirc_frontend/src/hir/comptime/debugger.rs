use fm::FileMap;
use imbl::Vector;
use noirc_errors::Location;

use crate::Type;
use crate::node_interner::{FuncId, NodeInterner};

use super::errors::InterpreterError;
use super::value::Value;

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
