use noirc_errors::{CustomDiagnostic as Diagnostic, DiagnosableError, Location};
use thiserror::Error;

#[derive(Debug)]
pub struct RuntimeError {
    pub location: Option<Location>,
    pub kind: RuntimeErrorKind,
}

impl RuntimeError {
    // XXX: In some places, we strip the span because we do not want span to
    // be introduced into the binary op or low level function code, for simplicity.
    //
    // It's possible to have it there, but it means we will need to proliferate the code with span
    //
    // This does make error reporting, less specific!
    pub fn remove_span(self) -> RuntimeErrorKind {
        self.kind
    }

    pub fn new(kind: RuntimeErrorKind, location: Option<Location>) -> RuntimeError {
        RuntimeError { location, kind }
    }
}

impl From<RuntimeErrorKind> for RuntimeError {
    fn from(kind: RuntimeErrorKind) -> RuntimeError {
        RuntimeError { location: None, kind }
    }
}

#[derive(Error, Debug)]
pub enum RuntimeErrorKind {
    // Array errors
    #[error("Out of bounds")]
    ArrayOutOfBounds { index: u128, bound: u128 },

    #[error("cannot call {func_name} function in non main function")]
    FunctionNonMainContext { func_name: String },

    // Environment errors
    #[error("Cannot find Array")]
    ArrayNotFound { found_type: String, name: String },

    #[error("Unstructured Error")]
    UnstructuredError { message: String },

    #[error("Spanless")]
    // This is here due to the fact we don't have full coverage for span
    Spanless(String),

    #[error("unimplemented")]
    Unimplemented(String),

    #[error("Unsupported operation error")]
    UnsupportedOp { op: String, first_type: String, second_type: String },
}

impl RuntimeErrorKind {
    pub fn expected_type(expected_type: &'static str, found_type: &str) -> RuntimeErrorKind {
        RuntimeErrorKind::UnstructuredError {
            message: format!("Expected a {expected_type}, but found {found_type}"),
        }
    }
}

impl DiagnosableError for RuntimeError {
    fn to_diagnostic(&self) -> Diagnostic {
        let span =
            if let Some(loc) = self.location { loc.span } else { noirc_errors::Span::new(0..0) };
        match &self.kind {
            RuntimeErrorKind::ArrayOutOfBounds { index, bound } => Diagnostic::simple_error(
                "index out of bounds".to_string(),
                format!("out of bounds error, index is {index} but length is {bound}"),
                span,
            ),
            RuntimeErrorKind::ArrayNotFound { found_type, name } => Diagnostic::simple_error(
                format!("cannot find an array with name {name}"),
                format!("{found_type} has type"),
                span,
            ),
            RuntimeErrorKind::UnstructuredError { message } => {
                Diagnostic::simple_error("".to_owned(), message.to_string(), span)
            }
            RuntimeErrorKind::UnsupportedOp { op, first_type, second_type } => {
                Diagnostic::simple_error(
                    "unsupported operation".to_owned(),
                    format!("no support for {op} with types {first_type} and {second_type}"),
                    span,
                )
            }
            RuntimeErrorKind::Spanless(message) => Diagnostic::from_message(message),
            RuntimeErrorKind::Unimplemented(message) => Diagnostic::from_message(message),
            RuntimeErrorKind::FunctionNonMainContext { func_name } => Diagnostic::simple_error(
                "cannot call function outside of main".to_owned(),
                format!("function {func_name} can only be called in main"),
                span,
            ),
        }
    }
}
