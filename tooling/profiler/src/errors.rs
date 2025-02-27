use fm::FileMap;
use noirc_errors::{CustomDiagnostic, Location};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum CliError {
    #[error("Failed to run profiler command")]
    Generic,
}

/// Report an error from the CLI that is not reliant on a stack trace.
pub(crate) fn report_error(message: String) -> Result<(), CliError> {
    let error = CustomDiagnostic::simple_error(message.clone(), String::new(), Location::dummy());
    noirc_errors::reporter::report(&FileMap::default(), &error, false);
    Err(CliError::Generic)
}
