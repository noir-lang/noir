use std::collections::BTreeMap;

use acvm::{
    acir::circuit::{
        brillig::BrilligFunctionId, ErrorSelector, OpcodeLocation, RawAssertionPayload,
        ResolvedAssertionPayload, ResolvedOpcodeLocation,
    },
    pwg::{ErrorLocation, OpcodeResolutionError},
    AcirField, FieldElement,
};
use noirc_abi::{display_abi_error, Abi, AbiErrorType};
use noirc_errors::{
    debug_info::DebugInfo, reporter::ReportedErrors, CustomDiagnostic, FileDiagnostic,
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

    /// Errors encountered while compiling the Noir program.
    /// These errors are already written to stderr.
    #[error("Aborting due to {} previous error{}", .0.error_count, if .0.error_count == 1 { "" } else { "s" })]
    ReportedErrors(ReportedErrors),
}
impl From<ReportedErrors> for CompileError {
    fn from(errors: ReportedErrors) -> Self {
        Self::ReportedErrors(errors)
    }
}

#[derive(Debug, Error)]
pub enum NargoError<F: AcirField> {
    /// Error while compiling Noir into ACIR.
    #[error("Failed to compile circuit")]
    CompilationError,

    /// ACIR circuit execution error
    #[error(transparent)]
    ExecutionError(#[from] ExecutionError<F>),

    /// Oracle handling error
    #[error(transparent)]
    ForeignCallError(#[from] ForeignCallError),
}

impl<F: AcirField> NargoError<F> {
    /// Extracts the user defined failure message from the ExecutionError
    /// If one exists.
    ///
    /// We want to extract the user defined error so that we can compare it
    /// in tests to expected failure messages
    pub fn user_defined_failure_message(
        &self,
        error_types: &BTreeMap<ErrorSelector, AbiErrorType>,
    ) -> Option<String> {
        match self {
            NargoError::ExecutionError(error) => match error {
                ExecutionError::AssertionFailed(payload, _, _) => match payload {
                    ResolvedAssertionPayload::String(message) => Some(message.to_string()),
                    ResolvedAssertionPayload::Raw(raw) => {
                        let abi_type = error_types.get(&raw.selector)?;
                        let decoded = display_abi_error(&raw.data, abi_type.clone());
                        Some(decoded.to_string())
                    }
                },
                ExecutionError::SolvingError(error, _) => match error {
                    OpcodeResolutionError::BlackBoxFunctionFailed(_, reason) => {
                        Some(reason.to_string())
                    }
                    _ => None,
                },
            },
            NargoError::ForeignCallError(error) => Some(error.to_string()),
            _ => None,
        }
    }
}

#[derive(Debug, Error)]
pub enum ExecutionError<F: AcirField> {
    #[error("Failed assertion")]
    AssertionFailed(
        ResolvedAssertionPayload<F>,
        Vec<ResolvedOpcodeLocation>,
        Option<BrilligFunctionId>,
    ),

    #[error("Failed to solve program: '{}'", .0)]
    SolvingError(OpcodeResolutionError<F>, Option<Vec<ResolvedOpcodeLocation>>),
}

/// Extracts the opcode locations from a nargo error.
fn extract_locations_from_error<F: AcirField>(
    error: &ExecutionError<F>,
    debug: &[DebugInfo],
) -> Option<Vec<Location>> {
    let mut opcode_locations = match error {
        ExecutionError::SolvingError(
            OpcodeResolutionError::BrilligFunctionFailed { .. },
            acir_call_stack,
        ) => acir_call_stack.clone(),
        ExecutionError::AssertionFailed(_, call_stack, _) => Some(call_stack.clone()),
        ExecutionError::SolvingError(
            OpcodeResolutionError::IndexOutOfBounds { opcode_location: error_location, .. },
            acir_call_stack,
        )
        | ExecutionError::SolvingError(
            OpcodeResolutionError::InvalidInputBitSize { opcode_location: error_location, .. },
            acir_call_stack,
        )
        | ExecutionError::SolvingError(
            OpcodeResolutionError::UnsatisfiedConstrain { opcode_location: error_location, .. },
            acir_call_stack,
        ) => match error_location {
            ErrorLocation::Unresolved => {
                unreachable!("Cannot resolve index for unsatisfied constraint")
            }
            ErrorLocation::Resolved(_) => acir_call_stack.clone(),
        },
        _ => None,
    }?;

    // Insert the top-level Acir location where the Brillig function failed
    for (i, resolved_location) in opcode_locations.iter().enumerate() {
        if let ResolvedOpcodeLocation {
            acir_function_index,
            opcode_location: OpcodeLocation::Brillig { acir_index, .. },
        } = resolved_location
        {
            let acir_location = ResolvedOpcodeLocation {
                acir_function_index: *acir_function_index,
                opcode_location: OpcodeLocation::Acir(*acir_index),
            };

            opcode_locations.insert(i, acir_location);
            // Go until the first brillig opcode as that means we have the start of a Brillig call stack.
            // We have to loop through the opcode locations in case we had ACIR calls
            // before the brillig function failure.
            break;
        }
    }

    let brillig_function_id = match error {
        ExecutionError::SolvingError(
            OpcodeResolutionError::BrilligFunctionFailed { function_id, .. },
            _,
        ) => Some(*function_id),
        ExecutionError::AssertionFailed(_, _, function_id) => *function_id,
        _ => None,
    };

    Some(
        opcode_locations
            .iter()
            .flat_map(|resolved_location| {
                debug[resolved_location.acir_function_index]
                    .opcode_location(&resolved_location.opcode_location)
                    .unwrap_or_else(|| {
                        if let (Some(brillig_function_id), Some(brillig_location)) = (
                            brillig_function_id,
                            &resolved_location.opcode_location.to_brillig_location(),
                        ) {
                            let brillig_locations = debug[resolved_location.acir_function_index]
                                .brillig_locations
                                .get(&brillig_function_id);
                            brillig_locations
                                .unwrap()
                                .get(brillig_location)
                                .cloned()
                                .unwrap_or_default()
                        } else {
                            vec![]
                        }
                    })
            })
            .collect(),
    )
}

fn extract_message_from_error(
    error_types: &BTreeMap<ErrorSelector, AbiErrorType>,
    nargo_err: &NargoError<FieldElement>,
) -> String {
    match nargo_err {
        NargoError::ExecutionError(ExecutionError::AssertionFailed(
            ResolvedAssertionPayload::String(message),
            _,
            _,
        )) => {
            format!("Assertion failed: '{message}'")
        }
        NargoError::ExecutionError(ExecutionError::AssertionFailed(
            ResolvedAssertionPayload::Raw(RawAssertionPayload { selector, data }),
            ..,
        )) => {
            if let Some(error_type) = error_types.get(selector) {
                format!("Assertion failed: {}", display_abi_error(data, error_type.clone()))
            } else {
                "Assertion failed".to_string()
            }
        }
        NargoError::ExecutionError(ExecutionError::SolvingError(
            OpcodeResolutionError::IndexOutOfBounds { index, array_size, .. },
            _,
        )) => {
            format!("Index out of bounds, array has size {array_size:?}, but index was {index:?}")
        }
        NargoError::ExecutionError(ExecutionError::SolvingError(
            OpcodeResolutionError::UnsatisfiedConstrain { .. },
            _,
        )) => "Failed constraint".into(),
        _ => nargo_err.to_string(),
    }
}

/// Tries to generate a runtime diagnostic from a nargo error. It will successfully do so if it's a runtime error with a call stack.
pub fn try_to_diagnose_runtime_error(
    nargo_err: &NargoError<FieldElement>,
    abi: &Abi,
    debug: &[DebugInfo],
) -> Option<FileDiagnostic> {
    let source_locations = match nargo_err {
        NargoError::ExecutionError(execution_error) => {
            extract_locations_from_error(execution_error, debug)?
        }
        _ => return None,
    };
    // The location of the error itself will be the location at the top
    // of the call stack (the last item in the Vec).
    let location = *source_locations.last()?;
    let message = extract_message_from_error(&abi.error_types, nargo_err);
    let error = CustomDiagnostic::simple_error(message, String::new(), location.span);
    Some(error.with_call_stack(source_locations).in_file(location.file))
}
