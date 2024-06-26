use std::rc::Rc;

use crate::{
    hir::def_collector::dc_crate::CompilationError, parser::ParserError, token::Tokens, Type,
};
use acvm::{acir::AcirField, FieldElement};
use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::{CustomDiagnostic, Location};

use super::value::Value;

/// The possible errors that can halt the interpreter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpreterError {
    ArgumentCountMismatch {
        expected: usize,
        actual: usize,
        location: Location,
    },
    TypeMismatch {
        expected: Type,
        value: Value,
        location: Location,
    },
    NonComptimeVarReferenced {
        name: String,
        location: Location,
    },
    IntegerOutOfRangeForType {
        value: FieldElement,
        typ: Type,
        location: Location,
    },
    ErrorNodeEncountered {
        location: Location,
    },
    FailingConstraint {
        message: Option<Value>,
        location: Location,
    },
    NoMethodFound {
        name: String,
        typ: Type,
        location: Location,
    },
    NonNumericCasted {
        value: Value,
        location: Location,
    },
    IndexOutOfBounds {
        index: usize,
        length: usize,
        location: Location,
    },
    TypeUnsupported {
        typ: Type,
        location: Location,
    },
    QuoteInRuntimeCode {
        location: Location,
    },
    CannotInlineMacro {
        value: Value,
        location: Location,
    },
    UnquoteFoundDuringEvaluation {
        location: Location,
    },
    FailedToParseMacro {
        error: ParserError,
        tokens: Rc<Tokens>,
        rule: &'static str,
        file: FileId,
    },
    NonComptimeFnCallInSameCrate {
        function: String,
        location: Location,
    },

    /// SilentFail is for silently failing without reporting an error. This is used to avoid
    /// issuing repeated errors, e.g. for cases the type checker is already expected to catch.
    SilentFail,

    Unimplemented {
        item: String,
        location: Location,
    },

    // Perhaps this should be unreachable! due to type checking also preventing this error?
    // Currently it and the Continue variant are the only interpreter errors without a Location field
    BreakNotInLoop {
        location: Location,
    },
    ContinueNotInLoop {
        location: Location,
    },

    // These cases are not errors, they are just used to prevent us from running more code
    // until the loop can be resumed properly. These cases will never be displayed to users.
    Break,
    Continue,
}

#[allow(unused)]
pub(super) type IResult<T> = std::result::Result<T, InterpreterError>;

impl From<InterpreterError> for CompilationError {
    fn from(error: InterpreterError) -> Self {
        CompilationError::InterpreterError(error)
    }
}

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
            | InterpreterError::FailingConstraint { location, .. }
            | InterpreterError::NoMethodFound { location, .. }
            | InterpreterError::NonNumericCasted { location, .. }
            | InterpreterError::IndexOutOfBounds { location, .. }
            | InterpreterError::TypeUnsupported { location, .. }
            | InterpreterError::QuoteInRuntimeCode { location, .. }
            | InterpreterError::CannotInlineMacro { location, .. }
            | InterpreterError::UnquoteFoundDuringEvaluation { location, .. }
            | InterpreterError::NonComptimeFnCallInSameCrate { location, .. }
            | InterpreterError::Unimplemented { location, .. }
            | InterpreterError::BreakNotInLoop { location, .. }
            | InterpreterError::ContinueNotInLoop { location, .. } => *location,
            InterpreterError::FailedToParseMacro { error, file, .. } => {
                Location::new(error.span(), *file)
            }
            InterpreterError::Break | InterpreterError::Continue => {
                panic!("Tried to get the location of Break/Continue error!")
            }
            InterpreterError::SilentFail => {
                panic!("Tried to get the location of InterpreterError::SilentFail!")
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
            InterpreterError::NonNumericCasted { value, location } => {
                let msg = "Only numeric types may be casted".into();
                let secondary = format!("`{}` is non-numeric", value.get_type());
                CustomDiagnostic::simple_error(msg, secondary, location.span)
            }
            InterpreterError::IndexOutOfBounds { index, length, location } => {
                let msg = format!("{index} is out of bounds for the array of length {length}");
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::TypeUnsupported { typ, location } => {
                let msg =
                    format!("The type `{typ}` is currently unsupported in comptime expressions");
                CustomDiagnostic::simple_error(msg, String::new(), location.span)
            }
            InterpreterError::QuoteInRuntimeCode { location } => {
                let msg = "`quote` may only be used in comptime code".into();
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
            InterpreterError::FailedToParseMacro { error, tokens, rule, file: _ } => {
                let message = format!("Failed to parse macro's token stream into {rule}");
                let tokens = vecmap(&tokens.0, ToString::to_string).join(" ");

                // 10 is an aribtrary number of tokens here chosen to fit roughly onto one line
                let token_stream = if tokens.len() > 10 {
                    format!("The resulting token stream was: {tokens}")
                } else {
                    format!(
                        "The resulting token stream was: (stream starts on next line)\n  {tokens}"
                    )
                };

                let push_the_problem_on_the_library_author = "To avoid this error in the future, try adding input validation to your macro. Erroring out early with an `assert` can be a good way to provide a user-friendly error message".into();

                let mut diagnostic = CustomDiagnostic::from(error);
                // Swap the parser's primary note to become the secondary note so that it is
                // more clear this error originates from failing to parse a macro.
                let secondary = std::mem::take(&mut diagnostic.message);
                diagnostic.add_secondary(secondary, error.span());
                diagnostic.message = message;
                diagnostic.add_note(token_stream);
                diagnostic.add_note(push_the_problem_on_the_library_author);
                diagnostic
            }
            InterpreterError::NonComptimeFnCallInSameCrate { function, location } => {
                let msg = format!("`{function}` cannot be called in a `comptime` context here");
                let secondary =
                    "This function must be `comptime` or in a separate crate to be called".into();
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
            InterpreterError::SilentFail => unreachable!("Uncaught InterpreterError::SilentFail"),
        }
    }
}
