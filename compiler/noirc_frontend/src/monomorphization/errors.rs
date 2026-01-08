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
    RecursiveType { typ: Type, location: Location },
    CannotComputeAssociatedConstant { name: String, err: TypeCheckError, location: Location },
    ReferenceReturnedFromIfOrMatch { typ: String, location: Location },
    AssignedToVarContainingReference { typ: String, location: Location },
    NestedVectors { location: Location },
    InvalidTypeInErrorMessage { typ: String, location: Location },
    ConstrainedReferenceToUnconstrained { typ: String, location: Location },
    UnconstrainedReferenceReturnToConstrained { typ: String, location: Location },
    UnconstrainedVectorReturnToConstrained { typ: String, location: Location },
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
            | MonomorphizationError::RecursiveType { location, .. }
            | MonomorphizationError::NoDefaultType { location, .. }
            | MonomorphizationError::ReferenceReturnedFromIfOrMatch { location, .. }
            | MonomorphizationError::AssignedToVarContainingReference { location, .. }
            | MonomorphizationError::NestedVectors { location }
            | MonomorphizationError::CannotComputeAssociatedConstant { location, .. }
            | MonomorphizationError::InvalidTypeInErrorMessage { location, .. }
            | MonomorphizationError::ConstrainedReferenceToUnconstrained { location, .. }
            | MonomorphizationError::UnconstrainedReferenceReturnToConstrained {
                location, ..
            }
            | MonomorphizationError::UnconstrainedVectorReturnToConstrained { location, .. } => {
                *location
            }
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
            MonomorphizationError::RecursiveType { typ, location } => {
                let message = format!("Type `{typ}` is recursive");
                let secondary = "All types in Noir must have a known size at compile-time".into();
                return CustomDiagnostic::simple_error(message, secondary, *location);
            }
            MonomorphizationError::CannotComputeAssociatedConstant { name, err, .. } => {
                format!(
                    "Could not determine the value of associated constant `{name}`, encountered error: `{err}`"
                )
            }
            MonomorphizationError::ReferenceReturnedFromIfOrMatch { typ, location } => {
                let message =
                    "Cannot return a reference type from an if or match expression".to_string();
                let secondary = if typ.starts_with("&") {
                    format!("`{typ}` returned here")
                } else {
                    format!("`{typ}`, which contains a reference type internally, returned here")
                };
                return CustomDiagnostic::simple_error(message, secondary, *location);
            }
            MonomorphizationError::AssignedToVarContainingReference { typ, location } => {
                let message =
                    "Cannot assign to a mutable variable which contains a reference internally"
                        .to_string();
                let secondary = if typ.starts_with("&") {
                    format!("Assigned expression has the type `{typ}`")
                } else {
                    format!(
                        "Assigned expression has the type `{typ}`, which contains a reference type internally"
                    )
                };
                return CustomDiagnostic::simple_error(message, secondary, *location);
            }
            MonomorphizationError::NestedVectors { .. } => {
                "Nested vectors, i.e. vectors within an array or vector, are not supported"
                    .to_string()
            }
            MonomorphizationError::InvalidTypeInErrorMessage { typ, location } => {
                let message = format!("Invalid type {typ} used in the error message");
                let secondary = "Error message fragments must be ABI compatible".into();
                return CustomDiagnostic::simple_error(message, secondary, *location);
            }
            MonomorphizationError::ConstrainedReferenceToUnconstrained { typ, .. } => {
                format!(
                    "Cannot pass mutable reference `{typ}` from a constrained runtime to an unconstrained runtime"
                )
            }
            MonomorphizationError::UnconstrainedReferenceReturnToConstrained { typ, .. } => {
                format!(
                    "Mutable reference `{typ}` cannot be returned from an unconstrained runtime to a constrained runtime"
                )
            }
            MonomorphizationError::UnconstrainedVectorReturnToConstrained { typ, .. } => {
                format!(
                    "Vector `{typ}` cannot be returned from an unconstrained runtime to a constrained runtime"
                )
            }
        };

        let location = error.location();
        CustomDiagnostic::simple_error(message, String::new(), location)
    }
}
