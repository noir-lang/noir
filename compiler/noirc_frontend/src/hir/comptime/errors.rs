use std::fmt::Display;
use std::rc::Rc;

use crate::{
    Type,
    ast::{Ident, TraitBound},
    hir::{
        def_collector::dc_crate::CompilationError,
        type_check::{NoMatchingImplFoundError, TypeCheckError},
    },
    parser::ParserError,
    signed_field::SignedField,
    token::Token,
};
use acvm::BlackBoxResolutionError;
use noirc_errors::{CustomDiagnostic, Location};

/// The possible errors that can halt the interpreter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpreterError {
    ArgumentCountMismatch {
        expected: usize,
        actual: usize,
        location: Location,
    },
    TypeMismatch {
        expected: String,
        actual: Type,
        location: Location,
    },
    UnexpectedZeroedValue {
        expected: String,
        location: Location,
    },
    NonComptimeVarReferenced {
        name: String,
        location: Location,
    },
    VariableNotInScope {
        location: Location,
    },
    IntegerOutOfRangeForType {
        value: SignedField,
        typ: Type,
        location: Location,
    },
    ErrorNodeEncountered {
        location: Location,
    },
    NonFunctionCalled {
        typ: Type,
        location: Location,
    },
    NonBoolUsedInIf {
        typ: Type,
        location: Location,
    },
    NonBoolUsedInWhile {
        typ: Type,
        location: Location,
    },
    NonBoolUsedInConstrain {
        typ: Type,
        location: Location,
    },
    FailingConstraint {
        message: Option<String>,
        location: Location,
        call_stack: im::Vector<Location>,
    },
    NonIntegerUsedInLoop {
        typ: Type,
        location: Location,
    },
    RangeBoundsTypeMismatch {
        start_type: Type,
        end_type: Type,
        location: Location,
    },
    NonPointerDereferenced {
        typ: Type,
        location: Location,
    },
    NonTupleOrStructInMemberAccess {
        typ: Type,
        location: Location,
    },
    NonArrayIndexed {
        typ: Type,
        location: Location,
    },
    NonIntegerUsedAsIndex {
        typ: Type,
        location: Location,
    },
    NonIntegerIntegerLiteral {
        typ: Type,
        location: Location,
    },
    InvalidArrayLength {
        err: Box<TypeCheckError>,
        location: Location,
    },
    InvalidAssociatedConstant {
        err: Box<TypeCheckError>,
        location: Location,
    },
    InvalidNumericGeneric {
        err: Box<TypeCheckError>,
        location: Location,
    },
    NonNumericCasted {
        typ: Type,
        location: Location,
    },
    IndexOutOfBounds {
        index: usize,
        length: usize,
        location: Location,
    },
    ExpectedStructToHaveField {
        typ: Type,
        field_name: String,
        location: Location,
    },
    TypeUnsupported {
        typ: Type,
        location: Location,
    },
    InvalidValueForUnary {
        typ: Type,
        operator: &'static str,
        location: Location,
    },
    InvalidValuesForBinary {
        lhs: Type,
        rhs: Type,
        operator: &'static str,
        location: Location,
    },
    BinaryOperationOverflow {
        operator: &'static str,
        location: Location,
    },
    NegateWithOverflow {
        location: Location,
    },
    CannotApplyMinusToType {
        location: Location,
        typ: &'static str,
    },
    CastToNonNumericType {
        typ: Type,
        location: Location,
    },
    NonStructInConstructor {
        typ: Type,
        location: Location,
    },
    NonEnumInConstructor {
        typ: Type,
        location: Location,
    },
    CannotInlineMacro {
        value: String,
        typ: Type,
        location: Location,
    },
    UnquoteFoundDuringEvaluation {
        location: Location,
    },
    DebugEvaluateComptime {
        diagnostic: CustomDiagnostic,
        location: Location,
    },
    FailedToParseMacro {
        error: Box<ParserError>,
        tokens: String,
        rule: &'static str,
        location: Location,
    },
    UnsupportedTopLevelItemUnquote {
        item: String,
        location: Location,
    },
    ComptimeDependencyCycle {
        function: String,
        location: Location,
    },
    NoImpl {
        location: Location,
    },
    NoMatchingImplFound {
        error: NoMatchingImplFoundError,
    },
    ImplMethodTypeMismatch {
        expected: Type,
        actual: Type,
        location: Location,
    },
    BreakNotInLoop {
        location: Location,
    },
    ContinueNotInLoop {
        location: Location,
    },
    BlackBoxError(BlackBoxResolutionError, Location),
    FailedToResolveTraitBound {
        trait_bound: TraitBound,
        location: Location,
    },
    TraitDefinitionMustBeAPath {
        location: Location,
    },
    FailedToResolveTraitDefinition {
        location: Location,
    },
    FunctionAlreadyResolved {
        location: Location,
    },
    MultipleMatchingImpls {
        object_type: Type,
        candidates: Vec<String>,
        location: Location,
    },
    Unimplemented {
        item: String,
        location: Location,
    },
    InvalidInComptimeContext {
        item: String,
        location: Location,
        explanation: String,
    },
    TypeAnnotationsNeededForMethodCall {
        location: Location,
    },
    ExpectedIdentForStructField {
        value: String,
        index: usize,
        location: Location,
    },
    InvalidAttribute {
        attribute: String,
        location: Location,
    },
    GenericNameShouldBeAnIdent {
        name: Rc<String>,
        location: Location,
    },
    DuplicateGeneric {
        name: Rc<String>,
        struct_name: String,
        duplicate_location: Location,
        existing_location: Location,
    },
    CannotResolveExpression {
        location: Location,
        expression: String,
    },
    CannotSetFunctionBody {
        location: Location,
        expression: String,
    },
    UnknownArrayLength {
        length: Type,
        err: Box<TypeCheckError>,
        location: Location,
    },
    CannotInterpretFormatStringWithErrors {
        location: Location,
    },
    GlobalsDependencyCycle {
        location: Location,
    },
    GlobalCouldNotBeResolved {
        location: Location,
    },
    LoopHaltedForUiResponsiveness {
        location: Location,
    },
    DuplicateStructFieldInSetFields {
        name: Ident,
        index: usize,
        previous_index: usize,
    },
    CheckedTransmuteFailed {
        actual: Type,
        expected: Type,
        location: Location,
    },
    StackOverflow {
        location: Location,
        call_stack: im::Vector<Location>,
    },
    EvaluationDepthOverflow {
        location: Location,
        call_stack: im::Vector<Location>,
    },
    AttributeRecursionLimitExceeded {
        location: Location,
    },
    UnexpectedEscapedTokenInQuote {
        token: Option<Token>,
        location: Location,
    },

    // These cases are not errors, they are just used to prevent us from running more code
    // until the loop can be resumed properly. These cases will never be displayed to users.
    Break,
    Continue,
    SkippedDueToEarlierErrors,
}

#[allow(unused)]
pub(super) type IResult<T> = Result<T, InterpreterError>;

impl From<InterpreterError> for CompilationError {
    fn from(error: InterpreterError) -> Self {
        CompilationError::InterpreterError(error)
    }
}

impl InterpreterError {
    /// Returns true if this error should be filtered out and not displayed to the user.
    /// This is used for internal control flow errors and errors that indicate the interpreter
    /// was skipped due to earlier errors that were already reported.
    pub(crate) fn should_be_filtered(&self) -> bool {
        matches!(
            self,
            InterpreterError::Break
                | InterpreterError::Continue
                | InterpreterError::SkippedDueToEarlierErrors
        )
    }

    pub fn location(&self) -> Location {
        match self {
            InterpreterError::ArgumentCountMismatch { location, .. }
            | InterpreterError::TypeMismatch { location, .. }
            | InterpreterError::UnexpectedZeroedValue { location, .. }
            | InterpreterError::NonComptimeVarReferenced { location, .. }
            | InterpreterError::VariableNotInScope { location, .. }
            | InterpreterError::IntegerOutOfRangeForType { location, .. }
            | InterpreterError::ErrorNodeEncountered { location, .. }
            | InterpreterError::NonFunctionCalled { location, .. }
            | InterpreterError::NonBoolUsedInIf { location, .. }
            | InterpreterError::NonBoolUsedInWhile { location, .. }
            | InterpreterError::NonBoolUsedInConstrain { location, .. }
            | InterpreterError::FailingConstraint { location, .. }
            | InterpreterError::NonIntegerUsedInLoop { location, .. }
            | InterpreterError::RangeBoundsTypeMismatch { location, .. }
            | InterpreterError::NonPointerDereferenced { location, .. }
            | InterpreterError::NonTupleOrStructInMemberAccess { location, .. }
            | InterpreterError::NonArrayIndexed { location, .. }
            | InterpreterError::NonIntegerUsedAsIndex { location, .. }
            | InterpreterError::NonIntegerIntegerLiteral { location, .. }
            | InterpreterError::InvalidArrayLength { location, .. }
            | InterpreterError::InvalidAssociatedConstant { location, .. }
            | InterpreterError::InvalidNumericGeneric { location, .. }
            | InterpreterError::NonNumericCasted { location, .. }
            | InterpreterError::IndexOutOfBounds { location, .. }
            | InterpreterError::ExpectedStructToHaveField { location, .. }
            | InterpreterError::TypeUnsupported { location, .. }
            | InterpreterError::InvalidValueForUnary { location, .. }
            | InterpreterError::InvalidValuesForBinary { location, .. }
            | InterpreterError::BinaryOperationOverflow { location, .. }
            | InterpreterError::NegateWithOverflow { location, .. }
            | InterpreterError::CannotApplyMinusToType { location, .. }
            | InterpreterError::CastToNonNumericType { location, .. }
            | InterpreterError::NonStructInConstructor { location, .. }
            | InterpreterError::NonEnumInConstructor { location, .. }
            | InterpreterError::CannotInlineMacro { location, .. }
            | InterpreterError::UnquoteFoundDuringEvaluation { location, .. }
            | InterpreterError::UnsupportedTopLevelItemUnquote { location, .. }
            | InterpreterError::ComptimeDependencyCycle { location, .. }
            | InterpreterError::Unimplemented { location, .. }
            | InterpreterError::InvalidInComptimeContext { location, .. }
            | InterpreterError::NoImpl { location, .. }
            | InterpreterError::ImplMethodTypeMismatch { location, .. }
            | InterpreterError::DebugEvaluateComptime { location, .. }
            | InterpreterError::BlackBoxError(_, location)
            | InterpreterError::BreakNotInLoop { location, .. }
            | InterpreterError::ContinueNotInLoop { location, .. }
            | InterpreterError::TraitDefinitionMustBeAPath { location }
            | InterpreterError::FailedToResolveTraitDefinition { location }
            | InterpreterError::FailedToResolveTraitBound { location, .. }
            | InterpreterError::FunctionAlreadyResolved { location, .. }
            | InterpreterError::MultipleMatchingImpls { location, .. }
            | InterpreterError::ExpectedIdentForStructField { location, .. }
            | InterpreterError::InvalidAttribute { location, .. }
            | InterpreterError::GenericNameShouldBeAnIdent { location, .. }
            | InterpreterError::DuplicateGeneric { duplicate_location: location, .. }
            | InterpreterError::TypeAnnotationsNeededForMethodCall { location }
            | InterpreterError::CannotResolveExpression { location, .. }
            | InterpreterError::CannotSetFunctionBody { location, .. }
            | InterpreterError::UnknownArrayLength { location, .. }
            | InterpreterError::CannotInterpretFormatStringWithErrors { location }
            | InterpreterError::GlobalsDependencyCycle { location }
            | InterpreterError::LoopHaltedForUiResponsiveness { location }
            | InterpreterError::GlobalCouldNotBeResolved { location }
            | InterpreterError::StackOverflow { location, .. }
            | InterpreterError::EvaluationDepthOverflow { location, .. }
            | InterpreterError::CheckedTransmuteFailed { location, .. }
            | InterpreterError::UnexpectedEscapedTokenInQuote { location, .. }
            | InterpreterError::AttributeRecursionLimitExceeded { location } => *location,
            InterpreterError::FailedToParseMacro { error, .. } => error.location(),
            InterpreterError::NoMatchingImplFound { error } => error.location,
            InterpreterError::DuplicateStructFieldInSetFields { name, .. } => name.location(),
            InterpreterError::Break
            | InterpreterError::Continue
            | InterpreterError::SkippedDueToEarlierErrors => {
                panic!("Tried to get the location of Break/Continue/SkippedDueToTypeErrors error!")
            }
        }
    }

    pub(crate) fn debug_evaluate_comptime(expr: impl Display, location: Location) -> Self {
        let mut formatted_result = format!("{expr}");
        // if multi-line, display on a separate line from the message
        if formatted_result.contains('\n') {
            formatted_result.insert(0, '\n');
        }
        let diagnostic = CustomDiagnostic::simple_info(
            "`comptime` expression ran:".to_string(),
            format!("After evaluation: {formatted_result}"),
            location,
        );
        InterpreterError::DebugEvaluateComptime { diagnostic, location }
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
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::TypeMismatch { expected, actual, location } => {
                let msg = format!("Expected `{expected}` but a value of type `{actual}` was given");
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::UnexpectedZeroedValue { expected, location } => {
                let msg = format!("Expected a concrete `{expected}` but a zeroed value was given");
                let secondary = format!(
                    "A zeroed value of `{expected}` may be created to satisfy the type system, but it's not expected to be used"
                );
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::NonComptimeVarReferenced { name, location } => {
                let msg = format!("Non-comptime variable `{name}` referenced in comptime code");
                let secondary = "Non-comptime variables can't be used in comptime code".to_string();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::VariableNotInScope { location } => {
                let msg = "Variable not in scope".to_string();
                let secondary = "Could not find variable".to_string();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::IntegerOutOfRangeForType { value, typ, location } => {
                let msg = format!("{value} is outside the range of the {typ} type");
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::ErrorNodeEncountered { location } => {
                let msg = "Internal Compiler Error: Error node encountered".to_string();
                let secondary = "This is a bug, please report this if found!".to_string();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::NonFunctionCalled { typ, location } => {
                let msg = "Only functions may be called".to_string();
                let secondary = format!("Expression has type {typ}");
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::NonBoolUsedInIf { typ, location } => {
                let msg = format!("Expected a `bool` but found `{typ}`");
                let secondary = "If conditions must be a boolean value".to_string();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::NonBoolUsedInWhile { typ, location } => {
                let msg = format!("Expected a `bool` but found `{typ}`");
                let secondary = "While conditions must be a boolean value".to_string();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::NonBoolUsedInConstrain { typ, location } => {
                let msg = format!("Expected a `bool` but found `{typ}`");
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::FailingConstraint { message, location, call_stack } => {
                let (primary, secondary) = match message {
                    Some(msg) => (msg.clone(), "Assertion failed".into()),
                    None => ("Assertion failed".into(), String::new()),
                };
                let diagnostic = CustomDiagnostic::simple_error(primary, secondary, *location);

                diagnostic.with_call_stack(call_stack.into_iter().copied().collect())
            }
            InterpreterError::NonIntegerUsedInLoop { typ, location } => {
                let msg = format!("Non-integer type `{typ}` used in for loop");
                let secondary = if matches!(typ, Type::FieldElement) {
                    "`field` is not an integer type, try `u32` instead".to_string()
                } else {
                    String::new()
                };
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::RangeBoundsTypeMismatch { start_type, end_type, location } => {
                let msg = format!(
                    "Range bounds have mismatched types: start is `{start_type}` but end is `{end_type}`"
                );
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::NonPointerDereferenced { typ, location } => {
                let msg = format!("Only references may be dereferenced, but found `{typ}`");
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::NonTupleOrStructInMemberAccess { typ, location } => {
                let msg = format!("The type `{typ}` has no fields to access");
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::NonArrayIndexed { typ, location } => {
                let msg = format!("Expected an array or vector but found a(n) {typ}");
                let secondary = "Only arrays or vectors may be indexed".into();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::NonIntegerUsedAsIndex { typ, location } => {
                let msg = format!("Expected an integer but found a(n) {typ}");
                let secondary =
                    "Only integers may be indexed. Note that this excludes `field`s".into();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::NonIntegerIntegerLiteral { typ, location } => {
                let msg = format!("This integer literal somehow has the type `{typ}`");
                let secondary = "This is likely a bug".into();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::InvalidArrayLength { err, location } => {
                let msg = "Invalid array length".to_string();
                let secondary = err.to_string();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::InvalidAssociatedConstant { err, location } => {
                let msg = "Invalid associated constant".to_string();
                let secondary = err.to_string();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::InvalidNumericGeneric { err, location } => {
                let msg = "Invalid numeric generic".to_string();
                let secondary = err.to_string();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::NonNumericCasted { typ, location } => {
                let msg = "Only numeric types may be casted".into();
                let secondary = format!("`{typ}` is non-numeric");
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::IndexOutOfBounds { index, length, location } => {
                let msg = format!("{index} is out of bounds for the array of length {length}");
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::ExpectedStructToHaveField { typ, field_name, location } => {
                let msg = format!("The type `{typ}` has no field named `{field_name}`");
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::TypeUnsupported { typ, location } => {
                let msg =
                    format!("The type `{typ}` is currently unsupported in comptime expressions");
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::InvalidValueForUnary { typ, operator, location } => {
                let msg = format!("`{typ}` cannot be used with unary {operator}");
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::InvalidValuesForBinary { lhs, rhs, operator, location } => {
                let msg = if *operator == "/" {
                    "Attempt to divide by zero".to_string()
                } else if *operator == "%" {
                    "Attempt to calculate the remainder with a divisor of zero".to_string()
                } else {
                    format!("No implementation for `{lhs}` {operator} `{rhs}`")
                };
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::BinaryOperationOverflow { operator, location } => {
                let operator = match *operator {
                    "+" => "add",
                    "-" => "subtract",
                    "*" => "multiply",
                    ">>" | "<<" => "bit-shift",
                    _ => operator,
                };
                let msg = format!("Attempt to {operator} with overflow");

                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::NegateWithOverflow { location } => {
                let msg = "Attempt to negate with overflow".to_string();
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::CannotApplyMinusToType { location, typ } => {
                let msg = format!("Cannot apply unary operator `-` to type `{typ}`");
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::CastToNonNumericType { typ, location } => {
                let msg = format!("Cannot cast to non-numeric type `{typ}`");
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::NonStructInConstructor { typ, location } => {
                let msg = format!("`{typ}` is not a struct type");
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::NonEnumInConstructor { typ, location } => {
                let msg = format!("`{typ}` is not an enum type");
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::CannotInlineMacro { value, typ, location } => {
                let msg = format!("Cannot inline values of type `{typ}` into this position");
                let secondary = format!("Cannot inline value `{value}`");
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::UnquoteFoundDuringEvaluation { location } => {
                let msg = "Unquote found during comptime evaluation".into();
                let secondary = "This is a bug".into();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::DebugEvaluateComptime { diagnostic, .. } => diagnostic.clone(),
            InterpreterError::FailedToParseMacro { error, tokens, rule, location } => {
                // If it's less than 48 chars, the error message fits in a single line (less than 80 chars total)
                let token_stream = if tokens.len() <= 48 && !tokens.contains('\n') {
                    format!("The resulting token stream was: {tokens}")
                } else {
                    format!(
                        "The resulting token stream was: (stream starts on next line)\n  {tokens}"
                    )
                };

                let push_the_problem_on_the_library_author = "To avoid this error in the future, try adding input validation to your macro. Erroring out early with an `assert` can be a good way to provide a user-friendly error message".into();

                let mut diagnostic = CustomDiagnostic::from(&**error);

                // Given more prominence to where the parser error happened, but still show that it's
                // because of a failure to parse a macro's token stream, and where that happens.
                let message = format!("Failed to parse macro's token stream into {rule}");
                diagnostic.add_secondary(message, *location);

                diagnostic.add_note(token_stream);
                diagnostic.add_note(push_the_problem_on_the_library_author);
                diagnostic
            }
            InterpreterError::UnsupportedTopLevelItemUnquote { item, location } => {
                let msg = "Unsupported statement type to unquote".into();
                let secondary =
                    "Only functions, structs, globals, and impls can be unquoted here".into();
                let mut error = CustomDiagnostic::simple_error(msg, secondary, *location);
                error.add_note(format!("Unquoted item was:\n{item}"));
                error
            }
            InterpreterError::ComptimeDependencyCycle { function, location } => {
                let msg = format!("Comptime dependency cycle while resolving `{function}`");
                let secondary =
                    "This function uses comptime code internally which calls into itself".into();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::Unimplemented { item, location } => {
                let msg = format!("{item} is currently unimplemented");
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::InvalidInComptimeContext { item, location, explanation } => {
                let msg = format!("{item} is invalid in comptime context");
                CustomDiagnostic::simple_error(msg, explanation.clone(), *location)
            }
            InterpreterError::BreakNotInLoop { location } => {
                let msg = "There is no loop to break out of!".into();
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::ContinueNotInLoop { location } => {
                let msg = "There is no loop to continue!".into();
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::NoImpl { location } => {
                let msg = "No impl found due to prior type error".into();
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::ImplMethodTypeMismatch { expected, actual, location } => {
                let msg = format!(
                    "Impl method type {actual} does not unify with trait method type {expected}"
                );
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::BlackBoxError(error, location) => {
                CustomDiagnostic::simple_error(error.to_string(), String::new(), *location)
            }
            InterpreterError::FailedToResolveTraitBound { trait_bound, location } => {
                let msg = format!("Failed to resolve trait bound `{trait_bound}`");
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::NoMatchingImplFound { error, .. } => error.into(),
            InterpreterError::Break => unreachable!("Uncaught InterpreterError::Break"),
            InterpreterError::Continue => unreachable!("Uncaught InterpreterError::Continue"),
            InterpreterError::TraitDefinitionMustBeAPath { location } => {
                let msg = "Trait definition arguments must be a variable or path".to_string();
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::FailedToResolveTraitDefinition { location } => {
                let msg = "Failed to resolve to a trait definition".to_string();
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::FunctionAlreadyResolved { location } => {
                let msg = "Function already resolved".to_string();
                let secondary =
                    "The function was previously called at compile-time or is in another crate"
                        .to_string();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::MultipleMatchingImpls { object_type, candidates, location } => {
                let message = format!("Multiple trait impls match the object type `{object_type}`");
                let secondary = "Ambiguous impl".to_string();
                let mut error = CustomDiagnostic::simple_error(message, secondary, *location);
                for (i, candidate) in candidates.iter().enumerate() {
                    error.add_note(format!("Candidate {}: `{candidate}`", i + 1));
                }
                error
            }
            InterpreterError::TypeAnnotationsNeededForMethodCall { location } => {
                let mut error = CustomDiagnostic::simple_error(
                    "Object type is unknown in method call".to_string(),
                    "Type must be known by this point to know which method to call".to_string(),
                    *location,
                );
                let message =
                    "Try adding a type annotation for the object type before this method call";
                error.add_note(message.to_string());
                error
            }
            InterpreterError::ExpectedIdentForStructField { value, index, location } => {
                let msg = format!(
                    "Quoted value in index {index} of this vector is not a valid field name"
                );
                let secondary = format!("`{value}` is not a valid field name for `set_fields`");
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::InvalidAttribute { attribute, location } => {
                let msg = format!("`{attribute}` is not a valid attribute");
                let secondary = "Note that this method expects attribute contents, without the leading `#[` or trailing `]`".to_string();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::GenericNameShouldBeAnIdent { name, location } => {
                let msg =
                            "Generic name needs to be a valid identifier (one word beginning with a letter)"
                                .to_string();
                let secondary = format!("`{name}` is not a valid identifier");
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::DuplicateGeneric {
                name,
                struct_name,
                duplicate_location,
                existing_location,
            } => {
                let msg = format!("`{struct_name}` already has a generic named `{name}`");
                let secondary = format!("`{name}` added here a second time");
                let mut error = CustomDiagnostic::simple_error(msg, secondary, *duplicate_location);

                let existing_msg = format!("`{name}` was previously defined here");
                error.add_secondary(existing_msg, *existing_location);
                error
            }
            InterpreterError::CannotResolveExpression { location, expression } => {
                let msg = format!("Cannot resolve expression `{expression}`");
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::CannotSetFunctionBody { location, expression } => {
                let msg = format!("`{expression}` is not a valid function body");
                CustomDiagnostic::simple_error(msg, String::new(), *location)
            }
            InterpreterError::UnknownArrayLength { length, err, location } => {
                let msg = format!("Could not determine array length `{length}`");
                let secondary = format!("Evaluating the length failed with: `{err}`");
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::CannotInterpretFormatStringWithErrors { location } => {
                let msg = "Cannot interpret format string with errors".to_string();
                let secondary =
                    "Some of the variables to interpolate could not be evaluated".to_string();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::GlobalsDependencyCycle { location } => {
                let msg = "This global recursively depends on itself".to_string();
                let secondary = String::new();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::GlobalCouldNotBeResolved { location } => {
                let msg = "Failed to resolve this global".to_string();
                let secondary = String::new();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::LoopHaltedForUiResponsiveness { location } => {
                let msg = "This loop took too much time to execute so it was halted for UI responsiveness"
                            .to_string();
                let secondary =
                    "This error doesn't happen in normal executions of `nargo`".to_string();
                CustomDiagnostic::simple_warning(msg, secondary, *location)
            }
            InterpreterError::SkippedDueToEarlierErrors => {
                unreachable!(
                    "SkippedDueToTypeErrors should be handled internally like Break/Continue"
                )
            }
            InterpreterError::DuplicateStructFieldInSetFields { name, index, previous_index } => {
                let msg = "Duplicate field name in call to `set_fields`".to_string();
                let secondary = format!(
                    "`{name}` first used as field {} then again as field {}",
                    previous_index + 1,
                    index + 1
                );
                CustomDiagnostic::simple_error(msg, secondary, name.location())
            }
            InterpreterError::CheckedTransmuteFailed { actual, expected, location } => {
                let msg = format!("Checked transmute failed: `{actual:?}` != `{expected:?}`");
                let secondary = String::new();
                CustomDiagnostic::simple_error(msg, secondary, *location)
            }
            InterpreterError::StackOverflow { location, call_stack } => {
                let diagnostic = CustomDiagnostic::simple_error(
                    "Comptime Stack Overflow".to_string(),
                    "Exceeded the recursion limit".to_string(),
                    *location,
                );
                diagnostic.with_call_stack(call_stack.into_iter().copied().collect())
            }
            InterpreterError::EvaluationDepthOverflow { location, call_stack } => {
                let diagnostic = CustomDiagnostic::simple_error(
                    "Comptime Evaluation Depth Overflow".to_string(),
                    "Exceeded the limit on the combined depth of expressions and recursion"
                        .to_string(),
                    *location,
                );
                diagnostic.with_call_stack(call_stack.into_iter().copied().collect())
            }
            InterpreterError::AttributeRecursionLimitExceeded { location } => {
                CustomDiagnostic::simple_error(
                    "Attribute recursion limit exceeded".to_string(),
                    "This attribute generates code with the same attribute, causing infinite recursion".to_string(),
                    *location,
                )
            }
            InterpreterError::UnexpectedEscapedTokenInQuote { token, location } => {
                let primary = match token {
                    Some(token) => format!("`{token}` cannot be escaped in quoted expressions"),
                    None => "Unexpected end of input after escape character in quoted expression".to_string(),
                };
                let secondary = "Only `$` may be escaped in `quote` expressions".to_string();
                CustomDiagnostic::simple_error(primary, secondary, *location)
            }
        }
    }
}

/// Comptime errors always wrap another error to show it together with a
/// comptime call or macro "something" that eventually led to that error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComptimeError {
    ErrorRunningAttribute {
        error: Box<CompilationError>,
        location: Location,
    },
    ErrorEvaluatingComptimeCall {
        method_name: &'static str,
        error: Box<CompilationError>,
        location: Location,
    },
}

impl ComptimeError {
    pub fn location(&self) -> Location {
        match self {
            ComptimeError::ErrorRunningAttribute { location, .. }
            | ComptimeError::ErrorEvaluatingComptimeCall { location, .. } => *location,
        }
    }
}

impl<'a> From<&'a ComptimeError> for CustomDiagnostic {
    fn from(error: &'a ComptimeError) -> Self {
        match error {
            ComptimeError::ErrorRunningAttribute { error, location } => {
                let mut diagnostic = CustomDiagnostic::from(&**error);
                diagnostic.add_secondary("While running this function attribute".into(), *location);
                diagnostic
            }
            ComptimeError::ErrorEvaluatingComptimeCall { method_name, error, location } => {
                let mut diagnostic = CustomDiagnostic::from(&**error);
                diagnostic.add_secondary(format!("While evaluating `{method_name}`"), *location);
                diagnostic
            }
        }
    }
}
