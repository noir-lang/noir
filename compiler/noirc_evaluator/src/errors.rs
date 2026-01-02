//! Noir Evaluator has two types of errors
//!
//! [RuntimeError]s that should be displayed to the user
//!
//! [InternalError]s that are used for checking internal logics of the SSA
//!
//! An Error of the former is a user Error
//!
//! An Error of the latter is an error in the implementation of the compiler
use iter_extended::vecmap;
use noirc_errors::{CustomDiagnostic, Location, call_stack::CallStack};

use noirc_frontend::signed_field::SignedField;
use thiserror::Error;

use crate::ssa::{ir::types::NumericType, ssa_gen::SHOW_INVALID_SSA_ENV_KEY};
use serde::{Deserialize, Serialize};

pub type RtResult<T> = Result<T, RuntimeError>;

#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub enum RuntimeError {
    #[error(transparent)]
    InternalError(#[from] InternalError),
    #[error("Range constraint of {num_bits} bits is too large for the Field size")]
    InvalidRangeConstraint { num_bits: u32, call_stack: CallStack },
    #[error("The value `{value}` cannot fit into `{typ}` which has range `{range}`")]
    IntegerOutOfBounds {
        value: SignedField,
        typ: NumericType,
        range: String,
        call_stack: CallStack,
    },
    #[error(
        "Attempted to recurse more than {limit} times during inlining function '{function_name}'"
    )]
    RecursionLimit { limit: u32, function_name: String, call_stack: CallStack },
    #[error("Expected array index to fit into a u64")]
    TypeConversion { from: String, into: String, call_stack: CallStack },
    #[error(
        "Integer {value}, sized {num_bits:?}, is over the max supported size of {max_num_bits:?} for the blackbox function's inputs"
    )]
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
    #[error(
        "Failed because the predicate is dynamic:\n{message}\nThe predicate must be known at compile time to be evaluated."
    )]
    StaticAssertDynamicPredicate { message: String, call_stack: CallStack },
    #[error("{message}")]
    StaticAssertFailed { message: String, call_stack: CallStack },
    #[error("Nested vectors, i.e. vectors within an array or vector, are not supported")]
    NestedVector { call_stack: CallStack },
    #[error("Big Integer modulus do no match")]
    BigIntModulus { call_stack: CallStack },
    #[error("Vectors cannot be returned from an unconstrained runtime to a constrained runtime")]
    UnconstrainedVectorReturnToConstrained { call_stack: CallStack },
    #[error(
        "Could not resolve some references to the array. All references must be resolved at compile time"
    )]
    UnknownReference { call_stack: CallStack },
    #[error(
        "Cannot return references from an if or match expression, or assignment within these expressions"
    )]
    ReturnedReferenceFromDynamicIf { call_stack: CallStack },
    #[error(
        "Cannot return a function from an if or match expression, or assignment within these expressions"
    )]
    ReturnedFunctionFromDynamicIf { call_stack: CallStack },
    /// This case is not an error. It's used during codegen to prevent inserting instructions after
    /// code when a break or continue is generated.
    #[error("Break or continue")]
    BreakOrContinue { call_stack: CallStack },

    #[error(
        "Only constant indices are supported when indexing an array containing reference values"
    )]
    DynamicIndexingWithReference { call_stack: CallStack },
    #[error(
        "Calling constrained function '{constrained}' from the unconstrained function '{unconstrained}'"
    )]
    UnconstrainedCallingConstrained {
        call_stack: CallStack,
        constrained: String,
        unconstrained: String,
    },
    #[error("SSA validation failed: {message}")]
    SsaValidationError { message: String, call_stack: CallStack },
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum SsaReport {
    Warning(InternalWarning),
    Bug(InternalBug),
}

impl From<SsaReport> for CustomDiagnostic {
    fn from(error: SsaReport) -> CustomDiagnostic {
        match error {
            SsaReport::Warning(warning) => {
                let message = warning.to_string();
                let (secondary_message, call_stack) = match warning {
                    InternalWarning::ReturnConstant { call_stack } => {
                        ("This variable contains a value which is constrained to be a constant. Consider removing this value as additional return values increase proving/verification time".to_string(), call_stack)
                    },
                };
                let call_stack = vecmap(call_stack, |location| location);
                let location = call_stack.last().expect("Expected RuntimeError to have a location");
                let diagnostic =
                    CustomDiagnostic::simple_warning(message, secondary_message, *location);
                diagnostic.with_call_stack(call_stack)
            }
            SsaReport::Bug(bug) => {
                let mut message = bug.to_string();
                let (secondary_message, call_stack) = match bug {
                    InternalBug::IndependentSubgraph { call_stack } => {
                        ("There is no path from the output of this Brillig call to either return values or inputs of the circuit, which creates an independent subgraph. This is quite likely a soundness vulnerability".to_string(), call_stack)
                    }
                    InternalBug::UncheckedBrilligCall { call_stack } => {
                        ("This Brillig call's inputs and its return values haven't been sufficiently constrained. This should be done to prevent potential soundness vulnerabilities".to_string(), call_stack)
                    }
                    InternalBug::AssertFailed { call_stack, message: assertion_failure_message } => {
                        if let Some(assertion_failure_message) = assertion_failure_message {
                            message.push_str(&format!(": {assertion_failure_message}"));
                        }
                        ("As a result, the compiled circuit is ensured to fail. Other assertions may also fail during execution".to_string(), call_stack)
                    }
                };
                let call_stack = vecmap(call_stack, |location| location);
                let location = call_stack.last().expect("Expected RuntimeError to have a location");
                let diagnostic =
                    CustomDiagnostic::simple_bug(message, secondary_message, *location);
                diagnostic.with_call_stack(call_stack)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Error, Serialize, Deserialize, Hash)]
pub enum InternalWarning {
    #[error("Return variable contains a constant value")]
    ReturnConstant { call_stack: CallStack },
}

#[derive(Debug, PartialEq, Eq, Clone, Error, Serialize, Deserialize, Hash)]
pub enum InternalBug {
    #[error("Input to Brillig function is in a separate subgraph to output")]
    IndependentSubgraph { call_stack: CallStack },
    #[error("Brillig function call isn't properly covered by a manual constraint")]
    UncheckedBrilligCall { call_stack: CallStack },
    #[error("Assertion is always false")]
    AssertFailed { call_stack: CallStack, message: Option<String> },
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
            | RuntimeError::UnknownLoopBound { call_stack }
            | RuntimeError::AssertConstantFailed { call_stack }
            | RuntimeError::StaticAssertDynamicMessage { call_stack }
            | RuntimeError::StaticAssertDynamicPredicate { call_stack, .. }
            | RuntimeError::StaticAssertFailed { call_stack, .. }
            | RuntimeError::IntegerOutOfBounds { call_stack, .. }
            | RuntimeError::InvalidBlackBoxInputBitSize { call_stack, .. }
            | RuntimeError::NestedVector { call_stack, .. }
            | RuntimeError::BigIntModulus { call_stack, .. }
            | RuntimeError::UnconstrainedVectorReturnToConstrained { call_stack }
            | RuntimeError::ReturnedReferenceFromDynamicIf { call_stack }
            | RuntimeError::ReturnedFunctionFromDynamicIf { call_stack }
            | RuntimeError::BreakOrContinue { call_stack }
            | RuntimeError::DynamicIndexingWithReference { call_stack }
            | RuntimeError::UnknownReference { call_stack }
            | RuntimeError::RecursionLimit { call_stack, .. }
            | RuntimeError::UnconstrainedCallingConstrained { call_stack, .. }
            | RuntimeError::SsaValidationError { call_stack, .. } => call_stack,
        }
    }
}

impl From<RuntimeError> for CustomDiagnostic {
    fn from(error: RuntimeError) -> CustomDiagnostic {
        let call_stack = vecmap(error.call_stack(), |location| *location);
        let diagnostic = error.into_diagnostic();
        diagnostic.with_call_stack(call_stack)
    }
}

impl RuntimeError {
    fn into_diagnostic(self) -> CustomDiagnostic {
        match self {
            RuntimeError::InternalError(cause) => {
                CustomDiagnostic::simple_error(
                    "Internal Consistency Evaluators Errors: \n
                    This is likely a bug. Consider opening an issue at https://github.com/noir-lang/noir/issues".to_owned(),
                    cause.to_string(),
                    Location::dummy(),
                )
            }
            RuntimeError::SsaValidationError { message, call_stack} => {
                // At the moment SSA validation error is just a caught panic, it doesn't have a call stack.
                let location =
                    call_stack.last().cloned().unwrap_or_else(Location::dummy);

                let mut diagnostic = CustomDiagnostic::simple_error(
                    format!("SSA validation error: {message}"),
                    String::new(),
                    location,
                );

                if std::env::var(SHOW_INVALID_SSA_ENV_KEY).is_err() {
                    diagnostic.notes.push(format!("Set the {SHOW_INVALID_SSA_ENV_KEY} env var to see the SSA."));
                }

                if call_stack.is_empty() {
                    // Clear it otherwise it points to the top of the file.
                    diagnostic.secondaries.clear();
                }

                diagnostic
            }
            RuntimeError::UnknownLoopBound { .. } => {
                let primary_message = self.to_string();
                // Unrolling sometimes has to produce an empty call stack.
                let location =
                    self.call_stack().last().cloned().unwrap_or_else(Location::dummy);

                CustomDiagnostic::simple_error(
                    primary_message,
                    "If attempting to fetch the length of a vector, try converting to an array. Vectors only use dynamic lengths.".to_string(),
                    location,
                )
            }
            _ => {
                let message = self.to_string();
                let location =
                    self.call_stack().last().unwrap_or_else(|| panic!("Expected RuntimeError to have a location. Error message: {message}"));

                CustomDiagnostic::simple_error(message, String::new(), *location)
            }
        }
    }
}
