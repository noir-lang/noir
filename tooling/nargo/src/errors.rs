use acvm::{
    acir::circuit::OpcodeLocation,
    pwg::{ErrorLocation, OpcodeResolutionError},
};
use noirc_errors::{debug_info::DebugInfo, CustomDiagnostic, FileDiagnostic};

pub use noirc_errors::Location;

use noirc_printable_type::ForeignCallError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NargoError {
    /// Error while compiling Noir into ACIR.
    #[error("Failed to compile circuit")]
    CompilationError,

    /// ACIR circuit execution error
    #[error(transparent)]
    ExecutionError(#[from] ExecutionError),

    /// Oracle handling error
    #[error(transparent)]
    ForeignCallError(#[from] ForeignCallError),
}

impl From<acvm::compiler::CompileError> for NargoError {
    fn from(_: acvm::compiler::CompileError) -> Self {
        NargoError::CompilationError
    }
}

impl NargoError {
    /// Extracts the user defined failure message from the ExecutionError
    /// If one exists.
    ///
    /// We want to extract the user defined error so that we can compare it
    /// in tests to expected failure messages
    pub fn user_defined_failure_message(&self) -> Option<&str> {
        let execution_error = match self {
            NargoError::ExecutionError(error) => error,
            _ => return None,
        };

        match execution_error {
            ExecutionError::AssertionFailed(message, _) => Some(message),
            ExecutionError::SolvingError(error) => match error {
                OpcodeResolutionError::IndexOutOfBounds { .. }
                | OpcodeResolutionError::UnsupportedBlackBoxFunc(_)
                | OpcodeResolutionError::OpcodeNotSolvable(_)
                | OpcodeResolutionError::UnsatisfiedConstrain { .. } => None,
                OpcodeResolutionError::BrilligFunctionFailed { message, .. } => Some(message),
                OpcodeResolutionError::BlackBoxFunctionFailed(_, reason) => Some(reason),
            },
        }
    }
}

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("Failed assertion: '{}'", .0)]
    AssertionFailed(String, Vec<OpcodeLocation>),

    #[error(transparent)]
    SolvingError(#[from] OpcodeResolutionError),
}

/// Extracts the opcode locations from a nargo error.
fn extract_locations_from_error(
    error: &ExecutionError,
    debug: &DebugInfo,
) -> Option<Vec<Location>> {
    let mut opcode_locations = match error {
        ExecutionError::SolvingError(OpcodeResolutionError::BrilligFunctionFailed {
            call_stack,
            ..
        })
        | ExecutionError::AssertionFailed(_, call_stack) => Some(call_stack.clone()),
        ExecutionError::SolvingError(OpcodeResolutionError::IndexOutOfBounds {
            opcode_location: error_location,
            ..
        })
        | ExecutionError::SolvingError(OpcodeResolutionError::UnsatisfiedConstrain {
            opcode_location: error_location,
        }) => match error_location {
            ErrorLocation::Unresolved => {
                unreachable!("Cannot resolve index for unsatisfied constraint")
            }
            ErrorLocation::Resolved(opcode_location) => Some(vec![*opcode_location]),
        },
        _ => None,
    }?;

    if let Some(OpcodeLocation::Brillig { acir_index, .. }) = opcode_locations.get(0) {
        opcode_locations.insert(0, OpcodeLocation::Acir(*acir_index));
    }

    Some(
        opcode_locations
            .iter()
            .flat_map(|opcode_location| debug.opcode_location(opcode_location).unwrap_or_default())
            .collect(),
    )
}

/// Tries to generate a runtime diagnostic from a nargo error. It will successfully do so if it's a runtime error with a call stack.
pub fn try_to_diagnose_runtime_error(
    nargo_err: &NargoError,
    debug: &DebugInfo,
) -> Option<FileDiagnostic> {
    let execution_error = match nargo_err {
        NargoError::ExecutionError(execution_error) => execution_error,
        _ => return None,
    };

    let source_locations = extract_locations_from_error(execution_error, debug)?;

    // The location of the error itself will be the location at the top
    // of the call stack (the last item in the Vec).
    let location = source_locations.last()?;

    let message = match nargo_err {
        NargoError::ExecutionError(ExecutionError::AssertionFailed(message, _)) => {
            format!("Assertion failed: '{message}'")
        }
        NargoError::ExecutionError(ExecutionError::SolvingError(
            OpcodeResolutionError::IndexOutOfBounds { index, array_size, .. },
        )) => {
            format!("Index out of bounds, array has size {array_size:?}, but index was {index:?}")
        }
        NargoError::ExecutionError(ExecutionError::SolvingError(
            OpcodeResolutionError::UnsatisfiedConstrain { .. },
        )) => "Failed constraint".into(),
        _ => nargo_err.to_string(),
    };

    Some(
        CustomDiagnostic::simple_error(message, String::new(), location.span)
            .in_file(location.file)
            .with_call_stack(source_locations),
    )
}
