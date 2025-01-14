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
use noirc_errors::{CustomDiagnostic as Diagnostic, FileDiagnostic};
use thiserror::Error;

use crate::ssa::ir::{call_stack::CallStack, types::NumericType};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub enum RuntimeError {
    #[error(transparent)]
    InternalError(#[from] InternalError),
    #[error("Range constraint of {num_bits} bits is too large for the Field size")]
    InvalidRangeConstraint { num_bits: u32, call_stack: CallStack },
    #[error("The value `{value:?}` cannot fit into `{typ}` which has range `{range}`")]
    IntegerOutOfBounds {
        value: FieldElement,
        typ: NumericType,
        range: String,
        call_stack: CallStack,
    },
    #[error("Expected array index to fit into a u64")]
    TypeConversion { from: String, into: String, call_stack: CallStack },
    #[error("{name:?} is not initialized")]
    UnInitialized { name: String, call_stack: CallStack },
    #[error("Integer sized {num_bits:?} is over the max supported size of {max_num_bits:?}")]
    UnsupportedIntegerSize { num_bits: u32, max_num_bits: u32, call_stack: CallStack },
    #[error("Integer {value}, sized {num_bits:?}, is over the max supported size of {max_num_bits:?} for the blackbox function's inputs")]
    InvalidBlackBoxInputBitSize {
        value: String,
        num_bits: u32,
        max_num_bits: u32,
        call_stack: CallStack,
    },
    #[error("Could not determine loop bound at compile-time")]
    UnknownLoopBound { call_stack: CallStack },
    #[error("Argument is not constant")]
    AssertConstantFailed { call_stack: CallStack },
    #[error("The static_assert message is not constant")]
    StaticAssertDynamicMessage { call_stack: CallStack },
    #[error("Argument is dynamic")]
    StaticAssertDynamicPredicate { call_stack: CallStack },
    #[error("{message}")]
    StaticAssertFailed { message: String, call_stack: CallStack },
    #[error("Nested slices, i.e. slices within an array or slice, are not supported")]
    NestedSlice { call_stack: CallStack },
    #[error("Big Integer modulus do no match")]
    BigIntModulus { call_stack: CallStack },
    #[error("Slices cannot be returned from an unconstrained runtime to a constrained runtime")]
    UnconstrainedSliceReturnToConstrained { call_stack: CallStack },
    #[error("All `oracle` methods should be wrapped in an unconstrained fn")]
    UnconstrainedOracleReturnToConstrained { call_stack: CallStack },
    #[error("Could not resolve some references to the array. All references must be resolved at compile time")]
    UnknownReference { call_stack: CallStack },
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum SsaReport {
    Warning(InternalWarning),
    Bug(InternalBug),
}

impl From<SsaReport> for FileDiagnostic {
    fn from(error: SsaReport) -> FileDiagnostic {
        match error {
            SsaReport::Warning(warning) => {
                let message = warning.to_string();
                let (secondary_message, call_stack) = match warning {
                    InternalWarning::ReturnConstant { call_stack } => {
                        ("This variable contains a value which is constrained to be a constant. Consider removing this value as additional return values increase proving/verification time".to_string(), call_stack)
                    },
                    InternalWarning::VerifyProof { call_stack } => {
                        ("verify_proof(...) aggregates data for the verifier, the actual verification will be done when the full proof is verified using nargo verify. nargo prove may generate an invalid proof if bad data is used as input to verify_proof".to_string(), call_stack)
                    },
                };
                let call_stack = vecmap(call_stack, |location| location);
                let file_id = call_stack.last().map(|location| location.file).unwrap_or_default();
                let location = call_stack.last().expect("Expected RuntimeError to have a location");
                let diagnostic =
                    Diagnostic::simple_warning(message, secondary_message, location.span);
                diagnostic.with_call_stack(call_stack).in_file(file_id)
            }
            SsaReport::Bug(bug) => {
                let message = bug.to_string();
                let (secondary_message, call_stack) = match bug {
                    InternalBug::IndependentSubgraph { call_stack } => {
                        ("There is no path from the output of this Brillig call to either return values or inputs of the circuit, which creates an independent subgraph. This is quite likely a soundness vulnerability".to_string(), call_stack)
                    }
                    InternalBug::UncheckedBrilligCall { call_stack } => {
                        ("This Brillig call's inputs and its return values haven't been sufficiently constrained. This should be done to prevent potential soundness vulnerabilities".to_string(), call_stack)
                    }
                    InternalBug::AssertFailed { call_stack } => ("As a result, the compiled circuit is ensured to fail. Other assertions may also fail during execution".to_string(), call_stack)
                };
                let call_stack = vecmap(call_stack, |location| location);
                let file_id = call_stack.last().map(|location| location.file).unwrap_or_default();
                let location = call_stack.last().expect("Expected RuntimeError to have a location");
                let diagnostic = Diagnostic::simple_bug(message, secondary_message, location.span);
                diagnostic.with_call_stack(call_stack).in_file(file_id)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Error, Serialize, Deserialize, Hash)]
pub enum InternalWarning {
    #[error("Return variable contains a constant value")]
    ReturnConstant { call_stack: CallStack },
    #[error("Calling std::verify_proof(...) does not verify a proof")]
    VerifyProof { call_stack: CallStack },
}

#[derive(Debug, PartialEq, Eq, Clone, Error, Serialize, Deserialize, Hash)]
pub enum InternalBug {
    #[error("Input to Brillig function is in a separate subgraph to output")]
    IndependentSubgraph { call_stack: CallStack },
    #[error("Brillig function call isn't properly covered by a manual constraint")]
    UncheckedBrilligCall { call_stack: CallStack },
    #[error("Assertion is always false")]
    AssertFailed { call_stack: CallStack },
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
    Unexpected { expected: String, found: String, call_stack: CallStack },
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
                | InternalError::Unexpected { call_stack, .. },
            )
            | RuntimeError::InvalidRangeConstraint { call_stack, .. }
            | RuntimeError::TypeConversion { call_stack, .. }
            | RuntimeError::UnInitialized { call_stack, .. }
            | RuntimeError::UnknownLoopBound { call_stack }
            | RuntimeError::AssertConstantFailed { call_stack }
            | RuntimeError::StaticAssertDynamicMessage { call_stack }
            | RuntimeError::StaticAssertDynamicPredicate { call_stack }
            | RuntimeError::StaticAssertFailed { call_stack, .. }
            | RuntimeError::IntegerOutOfBounds { call_stack, .. }
            | RuntimeError::UnsupportedIntegerSize { call_stack, .. }
            | RuntimeError::InvalidBlackBoxInputBitSize { call_stack, .. }
            | RuntimeError::NestedSlice { call_stack, .. }
            | RuntimeError::BigIntModulus { call_stack, .. }
            | RuntimeError::UnconstrainedSliceReturnToConstrained { call_stack }
            | RuntimeError::UnconstrainedOracleReturnToConstrained { call_stack }
            | RuntimeError::UnknownReference { call_stack } => call_stack,
        }
    }
}

impl From<RuntimeError> for FileDiagnostic {
    fn from(error: RuntimeError) -> FileDiagnostic {
        let call_stack = vecmap(error.call_stack(), |location| *location);
        let file_id = call_stack.last().map(|location| location.file).unwrap_or_default();
        let diagnostic = error.into_diagnostic();
        diagnostic.with_call_stack(call_stack).in_file(file_id)
    }
}

impl RuntimeError {
    fn into_diagnostic(self) -> Diagnostic {
        match self {
            RuntimeError::InternalError(cause) => {
                Diagnostic::simple_error(
                    "Internal Consistency Evaluators Errors: \n
                    This is likely a bug. Consider opening an issue at https://github.com/noir-lang/noir/issues".to_owned(),
                    cause.to_string(),
                    noirc_errors::Span::inclusive(0, 0)
                )
            }
            RuntimeError::UnknownLoopBound { .. } => {
                let primary_message = self.to_string();
                let location =
                    self.call_stack().last().expect("Expected RuntimeError to have a location");

                Diagnostic::simple_error(
                    primary_message,
                    "If attempting to fetch the length of a slice, try converting to an array. Slices only use dynamic lengths.".to_string(),
                    location.span,
                )
            }
            _ => {
                let message = self.to_string();
                let location =
                    self.call_stack().last().unwrap_or_else(|| panic!("Expected RuntimeError to have a location. Error message: {message}"));

                Diagnostic::simple_error(message, String::new(), location.span)
            }
        }
    }
}
