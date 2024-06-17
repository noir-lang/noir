use noirc_errors::{CustomDiagnostic, FileDiagnostic, Location};

use crate::hir::comptime::InterpreterError;

#[derive(Debug)]
pub enum MonomorphizationError {
    UnknownArrayLength { location: Location },
    TypeAnnotationsNeeded { location: Location },
    InterpreterError(InterpreterError),
}

impl MonomorphizationError {
    fn location(&self) -> Location {
        match self {
            MonomorphizationError::UnknownArrayLength { location }
            | MonomorphizationError::TypeAnnotationsNeeded { location } => *location,
            MonomorphizationError::InterpreterError(error) => error.get_location(),
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
        let message = match self {
            MonomorphizationError::UnknownArrayLength { .. } => {
                "Length of generic array could not be determined."
            }
            MonomorphizationError::TypeAnnotationsNeeded { .. } => "Type annotations needed",
            MonomorphizationError::InterpreterError(error) => return (&error).into(),
        };

        let location = self.location();
        CustomDiagnostic::simple_error(message.into(), String::new(), location.span)
    }
}
