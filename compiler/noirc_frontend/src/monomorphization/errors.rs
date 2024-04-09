use thiserror::Error;

use noirc_errors::{CustomDiagnostic, FileDiagnostic, Location};

#[derive(Debug, Error)]
pub enum MonomorphizationError {
    #[error("Length of generic array could not be determined.")]
    UnknownArrayLength { location: Location },

    #[error("Type annotations needed")]
    TypeAnnotationsNeeded { location: Location },
}

impl MonomorphizationError {
    fn location(&self) -> Location {
        match self {
            MonomorphizationError::UnknownArrayLength { location }
            | MonomorphizationError::TypeAnnotationsNeeded { location } => *location,
        }
    }
}

impl From<MonomorphizationError> for FileDiagnostic {
    fn from(error: MonomorphizationError) -> FileDiagnostic {
        let location = error.location();
        let call_stack = vec![location];
        let diagnostic = error.into_diagnostic();
        diagnostic.in_file(location.file).with_call_stack(call_stack)
    }
}

impl MonomorphizationError {
    fn into_diagnostic(self) -> CustomDiagnostic {
        let message = self.to_string();
        let location = self.location();

        CustomDiagnostic::simple_error(message, String::new(), location.span)
    }
}
