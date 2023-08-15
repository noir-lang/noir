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
use noirc_errors::{CustomDiagnostic as Diagnostic, FileDiagnostic, Location};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub enum RuntimeError {
    // We avoid showing the actual lhs and rhs since most of the time they are just 0
    // and 1 respectively. This would confuse users if a constraint such as
    // assert(foo < bar) fails with "failed constraint: 0 = 1."
    #[error("Failed constraint")]
    FailedConstraint { lhs: FieldElement, rhs: FieldElement, location: Option<Location> },
    #[error(transparent)]
    InternalError(#[from] InternalError),
    #[error("Index out of bounds, array has size {index:?}, but index was {array_size:?}")]
    IndexOutOfBounds { index: usize, array_size: usize, location: Option<Location> },
    #[error("Range constraint of {num_bits} bits is too large for the Field size")]
    InvalidRangeConstraint { num_bits: u32, location: Option<Location> },
    #[error("Expected array index to fit into a u64")]
    TypeConversion { from: String, into: String, location: Option<Location> },
    #[error("{name:?} is not initialized")]
    UnInitialized { name: String, location: Option<Location> },
    #[error("Integer sized {num_bits:?} is over the max supported size of {max_num_bits:?}")]
    UnsupportedIntegerSize { num_bits: u32, max_num_bits: u32, location: Option<Location> },
    #[error("Could not determine loop bound at compile-time")]
    UnknownLoopBound { location: Option<Location> },
    #[error("Argument is not constant")]
    AssertConstantFailed { location: Option<Location> },
}

#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub enum InternalError {
    #[error("ICE: Both expressions should have degree<=1")]
    DegreeNotReduced { location: Option<Location> },
    #[error("Try to get element from empty array")]
    EmptyArray { location: Option<Location> },
    #[error("ICE: {message:?}")]
    General { message: String, location: Option<Location> },
    #[error("ICE: {name:?} missing {arg:?} arg")]
    MissingArg { name: String, arg: String, location: Option<Location> },
    #[error("ICE: {name:?} should be a constant")]
    NotAConstant { name: String, location: Option<Location> },
    #[error("ICE: Undeclared AcirVar")]
    UndeclaredAcirVar { location: Option<Location> },
    #[error("ICE: Expected {expected:?}, found {found:?}")]
    UnExpected { expected: String, found: String, location: Option<Location> },
}

impl RuntimeError {
    fn location(&self) -> Option<Location> {
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
            | RuntimeError::UnsupportedIntegerSize { location, .. } => *location,
        }
    }
}

impl From<RuntimeError> for FileDiagnostic {
    fn from(error: RuntimeError) -> Self {
        dbg!(error.clone());
        let file_id = error.location().map(|loc| loc.file).unwrap();
        FileDiagnostic { file_id, diagnostic: error.into() }
    }
}

impl From<RuntimeError> for Diagnostic {
    fn from(error: RuntimeError) -> Diagnostic {
        match error {
            RuntimeError::InternalError(_) => Diagnostic::simple_error(
                "Internal Consistency Evaluators Errors: \n 
                This is likely a bug. Consider Opening an issue at https://github.com/noir-lang/noir/issues".to_owned(),
                "".to_string(),
                noirc_errors::Span::new(0..0)
            ),
            _ => {
                let span = if let Some(loc) = error.location() { loc.span } else { noirc_errors::Span::new(0..0) };
                Diagnostic::simple_error("".to_owned(), error.to_string(), span)
            }
        }
    }
}
