use std::collections::BTreeMap;
use std::fmt;

use alloy_primitives::U256;
use noirc_abi::Abi;
use noirc_errors::{
    CustomDiagnostic, call_stack::CallStackId, debug_info::DebugInfo, reporter::ReportedErrors,
};

pub use noirc_errors::Location;

use noirc_driver::CrateName;
use thiserror::Error;

use crate::foreign_calls::ForeignCallError;

/// Errors covering situations where a package cannot be compiled.
#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Package `{0}` has type `lib` but only `bin` types can be compiled")]
    LibraryCrate(CrateName),

    #[error("Package `{0}` is expected to have a `main` function but it does not")]
    MissingMainFunction(CrateName),

    #[error("Noir compilation failed: {} errors reported", .0.error_count)]
    ReportedErrors(ReportedErrors),

    #[error("{0}")]
    Generic(String),
}

impl From<ReportedErrors> for CompileError {
    fn from(errors: ReportedErrors) -> Self {
        Self::ReportedErrors(errors)
    }
}

/// Errors encountered during execution of the circuit.
#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("Failed assertion")]
    AssertionFailed {
        error_diagnostic: Option<CustomDiagnostic>,
    },

    #[error("Foreign call error: {0}")]
    ForeignCallError(#[from] ForeignCallError),

    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(String),

    #[error("Unexpected error: {0}")]
    General(String),

    #[error("Unsolved constraint")]
    UnsatisfiedConstrain,
}

/// Errors encountered when compiling or executing a Noir program.
#[derive(Debug, Error)]
pub enum NargoError {
    #[error(transparent)]
    CompileError(#[from] CompileError),

    #[error(transparent)]
    ExecutionError(#[from] ExecutionError),
}

/// Stub for resolved opcode location since we don't have ACVM
#[derive(Debug, Clone)]
pub struct ResolvedOpcodeLocation {
    pub source_location: Location,
    pub call_stack: Vec<Location>,
}

/// Stub function to extract location
pub fn extract_locations_from_error<F>(
    _error: &ExecutionError,
    _debug: &[DebugInfo],
) -> Option<ResolvedOpcodeLocation> {
    None
}

/// Stub function to create execution error
pub fn execution_error_from(
    error: String,
) -> ExecutionError {
    ExecutionError::General(error)
}

/// Stub function to try to diagnose runtime error
pub fn try_to_diagnose_runtime_error(
    _err: &ExecutionError,
    _abi: &Abi,
    _debug: &[DebugInfo],
) -> Option<CustomDiagnostic> {
    None
}