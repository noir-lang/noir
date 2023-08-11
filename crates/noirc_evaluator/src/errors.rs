//! Noir Evaluator has two types of errors
//!
//! [RuntimeError]s that should be displayed to the user
//!
//! [InternalError]s that are used for checking internal logics of the SSA
//!
//! An Error of the former is a user Error
//!
//! An Error of the latter is an error in the implementation of the compiler
use acvm::FieldElement;
use iter_extended::vecmap;
use noirc_errors::{CustomDiagnostic as Diagnostic, FileDiagnostic, Location};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub enum RuntimeError {
    // We avoid showing the actual lhs and rhs since most of the time they are just 0
    // and 1 respectively. This would confuse users if a constraint such as
    // assert(foo < bar) fails with "failed constraint: 0 = 1."
    #[error("Failed constraint")]
    FailedConstraint { lhs: FieldElement, rhs: FieldElement, location: Vec<Location> },
    #[error(transparent)]
    InternalError(#[from] InternalError),
    #[error("Index out of bounds, array has size {index:?}, but index was {array_size:?}")]
    IndexOutOfBounds { index: usize, array_size: usize, location: Vec<Location> },
    #[error("Range constraint of {num_bits} bits is too large for the Field size")]
    InvalidRangeConstraint { num_bits: u32, location: Vec<Location> },
    #[error("Expected array index to fit into a u64")]
    TypeConversion { from: String, into: String, location: Vec<Location> },
    #[error("{name:?} is not initialized")]
    UnInitialized { name: String, location: Vec<Location> },
    #[error("Integer sized {num_bits:?} is over the max supported size of {max_num_bits:?}")]
    UnsupportedIntegerSize { num_bits: u32, max_num_bits: u32, location: Vec<Location> },
    #[error("Could not determine loop bound at compile-time")]
    UnknownLoopBound { location: Vec<Location> },
    #[error("Argument is not constant")]
    AssertConstantFailed { location: Vec<Location> },
}

#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub enum InternalError {
    #[error("ICE: Both expressions should have degree<=1")]
    DegreeNotReduced { location: Vec<Location> },
    #[error("Try to get element from empty array")]
    EmptyArray { location: Vec<Location> },
    #[error("ICE: {message:?}")]
    General { message: String, location: Vec<Location> },
    #[error("ICE: {name:?} missing {arg:?} arg")]
    MissingArg { name: String, arg: String, location: Vec<Location> },
    #[error("ICE: {name:?} should be a constant")]
    NotAConstant { name: String, location: Vec<Location> },
    #[error("ICE: Undeclared AcirVar")]
    UndeclaredAcirVar { location: Vec<Location> },
    #[error("ICE: Expected {expected:?}, found {found:?}")]
    UnExpected { expected: String, found: String, location: Vec<Location> },
}

impl RuntimeError {
    fn into_location(self) -> Vec<Location> {
        match self {
            RuntimeError::InternalError(
                InternalError::DegreeNotReduced { location }
                | InternalError::EmptyArray { location }
                | InternalError::General { location, .. }
                | InternalError::MissingArg { location, .. }
                | InternalError::NotAConstant { location, .. }
                | InternalError::UndeclaredAcirVar { location }
                | InternalError::UnExpected { location, .. },
            )
            | RuntimeError::FailedConstraint { location, .. }
            | RuntimeError::IndexOutOfBounds { location, .. }
            | RuntimeError::InvalidRangeConstraint { location, .. }
            | RuntimeError::TypeConversion { location, .. }
            | RuntimeError::UnInitialized { location, .. }
            | RuntimeError::UnknownLoopBound { location }
            | RuntimeError::AssertConstantFailed { location }
            | RuntimeError::UnsupportedIntegerSize { location, .. } => location,
        }
    }

    fn location(&self) -> &[Location] {
        match self {
            RuntimeError::InternalError(
                InternalError::DegreeNotReduced { location }
                | InternalError::EmptyArray { location }
                | InternalError::General { location, .. }
                | InternalError::MissingArg { location, .. }
                | InternalError::NotAConstant { location, .. }
                | InternalError::UndeclaredAcirVar { location }
                | InternalError::UnExpected { location, .. },
            )
            | RuntimeError::FailedConstraint { location, .. }
            | RuntimeError::IndexOutOfBounds { location, .. }
            | RuntimeError::InvalidRangeConstraint { location, .. }
            | RuntimeError::TypeConversion { location, .. }
            | RuntimeError::UnInitialized { location, .. }
            | RuntimeError::UnknownLoopBound { location }
            | RuntimeError::AssertConstantFailed { location }
            | RuntimeError::UnsupportedIntegerSize { location, .. } => location,
        }
    }
}

impl From<RuntimeError> for Vec<FileDiagnostic> {
    fn from(error: RuntimeError) -> Vec<FileDiagnostic> {
        let file_ids = vecmap(error.location(), |loc| loc.file);
        vecmap(error.as_diagnostics().into_iter().zip(file_ids), |(diagnostic, file_id)| {
            FileDiagnostic { file_id, diagnostic }
        })
    }
}

impl RuntimeError {
    fn as_diagnostics(self) -> Vec<Diagnostic> {
        match self {
            RuntimeError::InternalError(_) => {
                vec![Diagnostic::simple_error(
                    "Internal Consistency Evaluators Errors: \n 
                    This is likely a bug. Consider Opening an issue at https://github.com/noir-lang/noir/issues".to_owned(),
                    "".to_string(),
                    noirc_errors::Span::new(0..0)
                )]
            }
            _ => {
                let message = self.to_string();
                let mut locations = self.into_location();
                let mut errors = Vec::new();

                if let Some(location) = locations.pop() {
                    errors.push(Diagnostic::simple_error(message, String::new(), location.span));
                }

                for location in locations.into_iter().rev() {
                    let message = "Called from here".into();
                    errors.push(Diagnostic::simple_error(message, String::new(), location.span));
                }

                errors
            }
        }
    }
}
