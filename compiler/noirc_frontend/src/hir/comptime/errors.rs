use crate::Type;
use acvm::FieldElement;
use noirc_errors::Location;

use super::value::Value;

/// The possible errors that can halt the interpreter.
#[derive(Debug)]
pub enum InterpreterError {
    ArgumentCountMismatch { expected: usize, actual: usize, call_location: Location },
    TypeMismatch { expected: Type, value: Value, location: Location },
    NonComptimeVarReferenced { name: String, location: Location },
    IntegerOutOfRangeForType { value: FieldElement, typ: Type, location: Location },
    ErrorNodeEncountered { location: Location },
    NonFunctionCalled { value: Value, location: Location },
    NonBoolUsedInIf { value: Value, location: Location },
    NonBoolUsedInConstrain { value: Value, location: Location },
    FailingConstraint { message: Option<Value>, location: Location },
    NoMethodFound { object: Value, typ: Type, location: Location },
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
