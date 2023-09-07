//! Noir Evaluator has two types of errors
//!
//! [RuntimeError]s that should be displayed to the user
//!
//! [InternalError]s that are used for checking internal logics of the SSA
//!
//! An Error of the former is a user Error
//!
//! An Error of the latter is an error in the implementation of the compiler
use acvm::acir::native_types::Expression;
use iter_extended::vecmap;
use noirc_errors::{CustomDiagnostic as Diagnostic, FileDiagnostic};
use thiserror::Error;

use crate::ssa::ir::dfg::CallStack;

#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub enum RuntimeError {
    #[error("{}", format_failed_constraint(.assert_message))]
    FailedConstraint {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        call_stack: CallStack,
        assert_message: Option<String>,
    },
    #[error(transparent)]
    InternalError(#[from] InternalError),
    #[error("Index out of bounds, array has size {index:?}, but index was {array_size:?}")]
    IndexOutOfBounds { index: usize, array_size: usize, call_stack: CallStack },
    #[error("Range constraint of {num_bits} bits is too large for the Field size")]
    InvalidRangeConstraint { num_bits: u32, call_stack: CallStack },
    #[error("Expected array index to fit into a u64")]
    TypeConversion { from: String, into: String, call_stack: CallStack },
    #[error("{name:?} is not initialized")]
    UnInitialized { name: String, call_stack: CallStack },
    #[error("Integer sized {num_bits:?} is over the max supported size of {max_num_bits:?}")]
    UnsupportedIntegerSize { num_bits: u32, max_num_bits: u32, call_stack: CallStack },
    #[error("Could not determine loop bound at compile-time")]
    UnknownLoopBound { call_stack: CallStack },
    #[error("Argument is not constant")]
    AssertConstantFailed { call_stack: CallStack },
}

// We avoid showing the actual lhs and rhs since most of the time they are just 0
// and 1 respectively. This would confuse users if a constraint such as
// assert(foo < bar) fails with "failed constraint: 0 = 1."
fn format_failed_constraint(message: &Option<String>) -> String {
    match message {
        Some(message) => format!("Failed constraint: '{}'", message),
        None => "Failed constraint".to_owned(),
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub enum InternalError {
    #[error("ICE: Both expressions should have degree<=1")]
    DegreeNotReduced { call_stack: CallStack },
    #[error("Try to get element from empty array")]
    EmptyArray { call_stack: CallStack },
    #[error("ICE: {message:?}")]
    General { message: String, call_stack: CallStack },
    #[error("ICE: {name:?} missing {arg:?} arg")]
    MissingArg { name: String, arg: String, call_stack: CallStack },
    #[error("ICE: {name:?} should be a constant")]
    NotAConstant { name: String, call_stack: CallStack },
    #[error("ICE: Undeclared AcirVar")]
    UndeclaredAcirVar { call_stack: CallStack },
    #[error("ICE: Expected {expected:?}, found {found:?}")]
    UnExpected { expected: String, found: String, call_stack: CallStack },
}

impl RuntimeError {
    fn call_stack(&self) -> &CallStack {
        match self {
            RuntimeError::InternalError(
                InternalError::DegreeNotReduced { call_stack }
                | InternalError::EmptyArray { call_stack }
                | InternalError::General { call_stack, .. }
                | InternalError::MissingArg { call_stack, .. }
                | InternalError::NotAConstant { call_stack, .. }
                | InternalError::UndeclaredAcirVar { call_stack }
                | InternalError::UnExpected { call_stack, .. },
            )
            | RuntimeError::FailedConstraint { call_stack, .. }
            | RuntimeError::IndexOutOfBounds { call_stack, .. }
            | RuntimeError::InvalidRangeConstraint { call_stack, .. }
            | RuntimeError::TypeConversion { call_stack, .. }
            | RuntimeError::UnInitialized { call_stack, .. }
            | RuntimeError::UnknownLoopBound { call_stack }
            | RuntimeError::AssertConstantFailed { call_stack }
            | RuntimeError::UnsupportedIntegerSize { call_stack, .. } => call_stack,
        }
    }
}

impl From<RuntimeError> for FileDiagnostic {
    fn from(error: RuntimeError) -> FileDiagnostic {
        let call_stack = vecmap(error.call_stack(), |location| *location);
        let diagnostic = error.into_diagnostic();
        let file_id = call_stack.last().map(|location| location.file).unwrap_or_default();

        diagnostic.in_file(file_id).with_call_stack(call_stack)
    }
}

impl RuntimeError {
    fn into_diagnostic(self) -> Diagnostic {
        match self {
            RuntimeError::InternalError(cause) => {
                Diagnostic::simple_error(
                    "Internal Consistency Evaluators Errors: \n
                    This is likely a bug. Consider Opening an issue at https://github.com/noir-lang/noir/issues".to_owned(),
                    cause.to_string(),
                    noirc_errors::Span::inclusive(0, 0)
                )
            }
            _ => {
                let message = self.to_string();
                let location =
                    self.call_stack().back().expect("Expected RuntimeError to have a location");

                Diagnostic::simple_error(message, String::new(), location.span)
            }
        }
    }
}
