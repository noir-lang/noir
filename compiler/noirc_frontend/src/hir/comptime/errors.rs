use crate::{hir::def_collector::dc_crate::CompilationError, Type};
use acvm::{acir::AcirField, FieldElement};
use noirc_errors::{CustomDiagnostic, Location};

use super::value::Value;

/// The possible errors that can halt the interpreter.
#[derive(Debug, Clone)]
pub enum InterpreterError {
    ArgumentCountMismatch { expected: usize, actual: usize, location: Location },
    TypeMismatch { expected: Type, value: Value, location: Location },
    NonComptimeVarReferenced { name: String, location: Location },
    IntegerOutOfRangeForType { value: FieldElement, typ: Type, location: Location },
    ErrorNodeEncountered { location: Location },
    NonFunctionCalled { value: Value, location: Location },
    NonBoolUsedInIf { value: Value, location: Location },
    NonBoolUsedInConstrain { value: Value, location: Location },
    FailingConstraint { message: Option<Value>, location: Location },
    NoMethodFound { name: String, typ: Type, location: Location },
    NonIntegerUsedInLoop { value: Value, location: Location },
    NonPointerDereferenced { value: Value, location: Location },
    NonTupleOrStructInMemberAccess { value: Value, location: Location },
    NonArrayIndexed { value: Value, location: Location },
    NonIntegerUsedAsIndex { value: Value, location: Location },
    NonIntegerIntegerLiteral { typ: Type, location: Location },
    NonIntegerArrayLength { typ: Type, location: Location },
    NonNumericCasted { value: Value, location: Location },
    IndexOutOfBounds { index: usize, length: usize, location: Location },
    ExpectedStructToHaveField { value: Value, field_name: String, location: Location },
    TypeUnsupported { typ: Type, location: Location },
    InvalidValueForUnary { value: Value, operator: &'static str, location: Location },
    InvalidValuesForBinary { lhs: Value, rhs: Value, operator: &'static str, location: Location },
    CastToNonNumericType { typ: Type, location: Location },
    QuoteInRuntimeCode { location: Location },
    NonStructInConstructor { typ: Type, location: Location },
    CannotInlineMacro { value: Value, location: Location },
    UnquoteFoundDuringEvaluation { location: Location },

    Unimplemented { item: &'static str, location: Location },

    // Perhaps this should be unreachable! due to type checking also preventing this error?
    // Currently it and the Continue variant are the only interpreter errors without a Location field
    BreakNotInLoop { location: Location },
    ContinueNotInLoop { location: Location },

    // These cases are not errors, they are just used to prevent us from running more code
    // until the loop can be resumed properly. These cases will never be displayed to users.
    Break,
    Continue,
}

#[allow(unused)]
pub(super) type IResult<T> = std::result::Result<T, InterpreterError>;

impl InterpreterError {
    pub fn into_compilation_error_pair(self) -> (CompilationError, fm::FileId) {
        let location = self.get_location();
        (CompilationError::InterpreterError(self), location.file)
    }

    pub fn get_location(&self) -> Location {
        match self {
            InterpreterError::ArgumentCountMismatch { location, .. }
            | InterpreterError::TypeMismatch { location, .. }
            | InterpreterError::NonComptimeVarReferenced { location, .. }
            | InterpreterError::IntegerOutOfRangeForType { location, .. }
            | InterpreterError::ErrorNodeEncountered { location, .. }
            | InterpreterError::NonFunctionCalled { location, .. }
            | InterpreterError::NonBoolUsedInIf { location, .. }
            | InterpreterError::NonBoolUsedInConstrain { location, .. }
            | InterpreterError::FailingConstraint { location, .. }
            | InterpreterError::NoMethodFound { location, .. }
            | InterpreterError::NonIntegerUsedInLoop { location, .. }
            | InterpreterError::NonPointerDereferenced { location, .. }
            | InterpreterError::NonTupleOrStructInMemberAccess { location, .. }
            | InterpreterError::NonArrayIndexed { location, .. }
            | InterpreterError::NonIntegerUsedAsIndex { location, .. }
            | InterpreterError::NonIntegerIntegerLiteral { location, .. }
            | InterpreterError::NonIntegerArrayLength { location, .. }
            | InterpreterError::NonNumericCasted { location, .. }
            | InterpreterError::IndexOutOfBounds { location, .. }
            | InterpreterError::ExpectedStructToHaveField { location, .. }
            | InterpreterError::TypeUnsupported { location, .. }
            | InterpreterError::InvalidValueForUnary { location, .. }
            | InterpreterError::InvalidValuesForBinary { location, .. }
            | InterpreterError::CastToNonNumericType { location, .. }
            | InterpreterError::QuoteInRuntimeCode { location, .. }
            | InterpreterError::NonStructInConstructor { location, .. }
            | InterpreterError::CannotInlineMacro { location, .. }
            | InterpreterError::UnquoteFoundDuringEvaluation { location, .. }
            | InterpreterError::Unimplemented { location, .. }
            | InterpreterError::BreakNotInLoop { location, .. }
            | InterpreterError::ContinueNotInLoop { location, .. } => *location,
            InterpreterError::Break | InterpreterError::Continue => {
                panic!("Tried to get the location of Break/Continue error!")
            }
        }
    }
}

impl<'a> From<&'a InterpreterError> for CustomDiagnostic {
    fn from(error: &'a InterpreterError) -> Self {
        match error {
            InterpreterError::ArgumentCountMismatch { expected, actual, location } => {
                let only = if expected > actual { "only " } else { "" };
                let plural = if *expected == 1 { "" } else { "s" };
                let was_were = if *actual == 1 { "was" } else { "were" };
                let msg = format!(
                    "Expected {expected} argument{plural}, but {only}{actual} {was_were} provided"
                );

                let few_many = if actual < expected { "few" } else { "many" };
                let secondary = format!("Too {few_many} arguments");
                CustomDiagnostic::simple_error(msg, secondary, location.span)
            }
            InterpreterError::TypeMismatch { expected, value, location } => {
                let typ = value.get_type();
                let msg = format!("Expected `{expected}` but a value of type `{typ}` was given");
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::NonComptimeVarReferenced { name, location } => {
                let msg = format!("Non-comptime variable `{name}` referenced in comptime code");
                let secondary = "Non-comptime variables can't be used in comptime code".to_string();
                CustomDiagnostic::simple_error(msg, secondary, location.span)
            }
            InterpreterError::IntegerOutOfRangeForType { value, typ, location } => {
                let int = match value.try_into_u128() {
                    Some(int) => int.to_string(),
                    None => value.to_string(),
                };
                let msg = format!("{int} is outside the range of the {typ} type");
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::ErrorNodeEncountered { location } => {
                let msg = "Internal Compiler Error: Error node encountered".to_string();
                let secondary = "This is a bug, please report this if found!".to_string();
                CustomDiagnostic::simple_error(msg, secondary, location.span)
            }
            InterpreterError::NonFunctionCalled { value, location } => {
                let msg = "Only functions may be called".to_string();
                let secondary = format!("Expression has type {}", value.get_type());
                CustomDiagnostic::simple_error(msg, secondary, location.span)
            }
            InterpreterError::NonBoolUsedInIf { value, location } => {
                let msg = format!("Expected a `bool` but found `{}`", value.get_type());
                let secondary = "If conditions must be a boolean value".to_string();
                CustomDiagnostic::simple_error(msg, secondary, location.span)
            }
            InterpreterError::NonBoolUsedInConstrain { value, location } => {
                let msg = format!("Expected a `bool` but found `{}`", value.get_type());
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::FailingConstraint { message, location } => {
                let (primary, secondary) = match message {
                    Some(msg) => (format!("{msg:?}"), "Assertion failed".into()),
                    None => ("Assertion failed".into(), String::new()),
                };
                CustomDiagnostic::simple_error(primary, secondary, location.span)
            }
            InterpreterError::NoMethodFound { name, typ, location } => {
                let msg = format!("No method named `{name}` found for type `{typ}`");
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::NonIntegerUsedInLoop { value, location } => {
                let typ = value.get_type();
                let msg = format!("Non-integer type `{typ}` used in for loop");
                let secondary = if matches!(typ.as_ref(), &Type::FieldElement) {
                    "`field` is not an integer type, try `u64` instead".to_string()
                } else {
                    String::new()
                };
                CustomDiagnostic::simple_error(msg, secondary, location.span)
            }
            InterpreterError::NonPointerDereferenced { value, location } => {
                let typ = value.get_type();
                let msg = format!("Only references may be dereferenced, but found `{typ}`");
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::NonTupleOrStructInMemberAccess { value, location } => {
                let msg = format!("The type `{}` has no fields to access", value.get_type());
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::NonArrayIndexed { value, location } => {
                let msg = format!("Expected an array or slice but found a(n) {}", value.get_type());
                let secondary = "Only arrays or slices may be indexed".into();
                CustomDiagnostic::simple_error(msg, secondary, location.span)
            }
            InterpreterError::NonIntegerUsedAsIndex { value, location } => {
                let msg = format!("Expected an integer but found a(n) {}", value.get_type());
                let secondary =
                    "Only integers may be indexed. Note that this excludes `field`s".into();
                CustomDiagnostic::simple_error(msg, secondary, location.span)
            }
            InterpreterError::NonIntegerIntegerLiteral { typ, location } => {
                let msg = format!("This integer literal somehow has the type `{typ}`");
                let secondary = "This is likely a bug".into();
                CustomDiagnostic::simple_error(msg, secondary, location.span)
            }
            InterpreterError::NonIntegerArrayLength { typ, location } => {
                let msg = format!("Non-integer array length: `{typ}`");
                let secondary = "Array lengths must be integers".into();
                CustomDiagnostic::simple_error(msg, secondary, location.span)
            }
            InterpreterError::NonNumericCasted { value, location } => {
                let msg = "Only numeric types may be casted".into();
                let secondary = format!("`{}` is non-numeric", value.get_type());
                CustomDiagnostic::simple_error(msg, secondary, location.span)
            }
            InterpreterError::IndexOutOfBounds { index, length, location } => {
                let msg = format!("{index} is out of bounds for the array of length {length}");
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::ExpectedStructToHaveField { value, field_name, location } => {
                let typ = value.get_type();
                let msg = format!("The type `{typ}` has no field named `{field_name}`");
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::TypeUnsupported { typ, location } => {
                let msg =
                    format!("The type `{typ}` is currently unsupported in comptime expressions");
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::InvalidValueForUnary { value, operator, location } => {
                let msg = format!("`{}` cannot be used with unary {operator}", value.get_type());
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::InvalidValuesForBinary { lhs, rhs, operator, location } => {
                let lhs = lhs.get_type();
                let rhs = rhs.get_type();
                let msg = format!("No implementation for `{lhs}` {operator} `{rhs}`",);
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::CastToNonNumericType { typ, location } => {
                let msg = format!("Cannot cast to non-numeric type `{typ}`");
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::QuoteInRuntimeCode { location } => {
                let msg = "`quote` may only be used in comptime code".into();
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::NonStructInConstructor { typ, location } => {
                let msg = format!("`{typ}` is not a struct type");
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::CannotInlineMacro { value, location } => {
                let msg = "Cannot inline value into runtime code if it contains references".into();
                let secondary = format!("Cannot inline value {value:?}");
                CustomDiagnostic::simple_error(msg, secondary, location.span)
            }
            InterpreterError::UnquoteFoundDuringEvaluation { location } => {
                let msg = "Unquote found during comptime evaluation".into();
                let secondary = "This is a bug".into();
                CustomDiagnostic::simple_error(msg, secondary, location.span)
            }
            InterpreterError::Unimplemented { item, location } => {
                let msg = format!("{item} is currently unimplemented");
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::BreakNotInLoop { location } => {
                let msg = "There is no loop to break out of!".into();
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::ContinueNotInLoop { location } => {
                let msg = "There is no loop to continue!".into();
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::Break => unreachable!("Uncaught InterpreterError::Break"),
            InterpreterError::Continue => unreachable!("Uncaught InterpreterError::Continue"),
        }
    }
}
