use thiserror::Error;

use noirc_errors::{CustomDiagnostic, FileDiagnostic, Location};

use crate::node_interner::generic_arith::{ArithConstraintError, ArithExprError};

#[derive(Debug, Error)]
pub enum MonomorphizationError {
    #[error("Length of generic array could not be determined.")]
    UnknownArrayLength { location: Location },

    #[error("Type annotations needed")]
    TypeAnnotationsNeeded { location: Location },

    #[error("Failed to prove generic arithmetic equivalent:\n{error}")]
    ArithConstraintError { error: ArithConstraintError },

    #[error("Failed to prove generic arithmetic equivalent:\n{arith_expr_error}")]
    ArithExprError { arith_expr_error: ArithExprError, location: Location },
}

impl MonomorphizationError {
    fn location(&self) -> Location {
        match self {
            MonomorphizationError::UnknownArrayLength { location }
            | MonomorphizationError::TypeAnnotationsNeeded { location }
            | MonomorphizationError::ArithExprError { location, .. } => *location,

            MonomorphizationError::ArithConstraintError { error } => error.location(),
        }
    }

    fn other_locations(&self) -> Vec<Location> {
        match self {
            MonomorphizationError::UnknownArrayLength { .. }
            | MonomorphizationError::TypeAnnotationsNeeded { .. }
            | MonomorphizationError::ArithExprError { .. } => vec![],
            MonomorphizationError::ArithConstraintError { error } => error.other_locations(),
        }
    }
}

impl From<MonomorphizationError> for FileDiagnostic {
    fn from(error: MonomorphizationError) -> FileDiagnostic {
        let location = error.location();
        let call_stack: Vec<_> = std::iter::once(location)
            .chain(error.other_locations())
            .filter(|x| x != &Location::dummy())
            .collect();
        let diagnostic = error.into_diagnostic();
        diagnostic.in_file(location.file).with_call_stack(call_stack)
    }
}

impl From<ArithConstraintError> for MonomorphizationError {
    fn from(error: ArithConstraintError) -> Self {
        Self::ArithConstraintError { error }
    }
}

impl MonomorphizationError {
    fn into_diagnostic(self) -> CustomDiagnostic {
        let message = self.to_string();
        let location = self.location();

        CustomDiagnostic::simple_error(message, String::new(), location.span)
    }
}
