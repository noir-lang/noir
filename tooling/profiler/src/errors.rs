use fm::{FileId, FileMap};
use noirc_errors::{CustomDiagnostic, function_locations::FunctionLocations};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum CliError {
    #[error("Failed to run profiler command")]
    Generic,
}

/// Report an error from the CLI that is not reliant on a stack trace.
pub(crate) fn report_error(message: &str) -> Result<(), CliError> {
    let error = CustomDiagnostic::from_message(message, FileId::dummy());
    let function_locations = FunctionLocations::new();
    noirc_errors::reporter::report(&FileMap::default(), &function_locations, &error, false);
    Err(CliError::Generic)
}
