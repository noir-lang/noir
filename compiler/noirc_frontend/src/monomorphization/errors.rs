use noirc_errors::{CustomDiagnostic, Location};

use crate::{
    Type,
    hir::{comptime::InterpreterError, type_check::TypeCheckError},
};

#[derive(Debug)]
pub enum MonomorphizationError {
    UnknownArrayLength { length: Type, err: TypeCheckError, location: Location },
    UnknownConstant { location: Location },
    NoDefaultType { location: Location },
    InternalError { message: &'static str, location: Location },
    InterpreterError(InterpreterError),
    ComptimeFnInRuntimeCode { name: String, location: Location },
    ComptimeTypeInRuntimeCode { typ: String, location: Location },
    CheckedTransmuteFailed { actual: Type, expected: Type, location: Location },
    CheckedCastFailed { actual: Type, expected: Type, location: Location },
}

impl MonomorphizationError {
    fn location(&self) -> Location {
        match self {
            MonomorphizationError::UnknownArrayLength { location, .. }
            | MonomorphizationError::UnknownConstant { location }
            | MonomorphizationError::InternalError { location, .. }
            | MonomorphizationError::ComptimeFnInRuntimeCode { location, .. }
            | MonomorphizationError::ComptimeTypeInRuntimeCode { location, .. }
            | MonomorphizationError::CheckedTransmuteFailed { location, .. }
            | MonomorphizationError::CheckedCastFailed { location, .. }
            | MonomorphizationError::NoDefaultType { location, .. } => *location,
            MonomorphizationError::InterpreterError(error) => error.location(),
        }
    }
}

impl From<MonomorphizationError> for CustomDiagnostic {
    fn from(error: MonomorphizationError) -> CustomDiagnostic {
        let message = match &error {
            MonomorphizationError::UnknownArrayLength { length, err, .. } => {
                format!("Could not determine array length `{length}`, encountered error: `{err}`")
            }
            MonomorphizationError::UnknownConstant { .. } => {
                "Could not resolve constant".to_string()
            }
            MonomorphizationError::CheckedTransmuteFailed { actual, expected, .. } => {
                format!("checked_transmute failed: `{actual:?}` != `{expected:?}`")
            }
            MonomorphizationError::CheckedCastFailed { actual, expected, .. } => {
                format!("Arithmetic generics simplification failed: `{actual:?}` != `{expected:?}`")
            }
            MonomorphizationError::NoDefaultType { location } => {
                let message = "Type annotation needed".into();
                let secondary = "Could not determine type of generic argument".into();
                return CustomDiagnostic::simple_error(message, secondary, *location);
            }
            MonomorphizationError::InterpreterError(error) => return error.into(),
            MonomorphizationError::InternalError { message, .. } => message.to_string(),
            MonomorphizationError::ComptimeFnInRuntimeCode { name, location } => {
                let message = format!("Comptime function {name} used in runtime code");
                let secondary =
                    "Comptime functions must be in a comptime block to be called".into();
                return CustomDiagnostic::simple_error(message, secondary, *location);
            }
            MonomorphizationError::ComptimeTypeInRuntimeCode { typ, location } => {
                let message = format!("Comptime-only type `{typ}` used in runtime code");
                let secondary = "Comptime type used here".into();
                return CustomDiagnostic::simple_error(message, secondary, *location);
            }
        };

        let location = error.location();
        CustomDiagnostic::simple_error(message, String::new(), location)
    }
}
