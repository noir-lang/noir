use noirc_errors::{CustomDiagnostic, FileDiagnostic, Location};

use crate::{hir::comptime::InterpreterError, Type};

#[derive(Debug)]
pub enum MonomorphizationError {
    UnknownArrayLength { length: Type, location: Location },
    NoDefaultType { location: Location },
    InternalError { message: &'static str, location: Location },
    InterpreterError(InterpreterError),
}

impl MonomorphizationError {
    fn location(&self) -> Location {
        match self {
            MonomorphizationError::UnknownArrayLength { location, .. }
            | MonomorphizationError::InternalError { location, .. }
            | MonomorphizationError::NoDefaultType { location, .. } => *location,
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
        let message = match &self {
            MonomorphizationError::UnknownArrayLength { length, .. } => {
                format!("ICE: Could not determine array length `{length}`")
            }
            MonomorphizationError::NoDefaultType { location } => {
                let message = "Type annotation needed".into();
                let secondary = "Could not determine type of generic argument".into();
                return CustomDiagnostic::simple_error(message, secondary, location.span);
            }
            MonomorphizationError::InterpreterError(error) => return error.into(),
            MonomorphizationError::InternalError { message, .. } => message.to_string(),
        };

        let location = self.location();
        CustomDiagnostic::simple_error(message, String::new(), location.span)
    }
}
