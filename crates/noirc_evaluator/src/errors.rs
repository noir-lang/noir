use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::DiagnosableError;
use noirc_errors::Span;
use thiserror::Error;

#[derive(Debug)]
pub struct RuntimeError {
    pub span: Span,
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
}

impl RuntimeErrorKind {
    pub fn add_span(self, span: Span) -> RuntimeError {
        RuntimeError { span, kind: self }
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
    UnsupportedOp {
        op: String,
        first_type: String,
        second_type: String,
    },
}

impl RuntimeErrorKind {
    pub fn expected_type(expected_type: &'static str, found_type: &str) -> RuntimeErrorKind {
        RuntimeErrorKind::UnstructuredError {
            message: format!("Expected a {}, but found {}", expected_type, found_type),
        }
    }
}

impl DiagnosableError for RuntimeError {
    fn to_diagnostic(&self) -> Diagnostic {
        let span = self.span;
        match &self.kind {
            RuntimeErrorKind::ArrayOutOfBounds { index, bound } => Diagnostic::simple_error(
                "index out of bounds".to_string(),
                format!(
                    "out of bounds error, index is {} but length is {}",
                    index, bound
                ),
                span,
            ),
            RuntimeErrorKind::ArrayNotFound { found_type, name } => Diagnostic::simple_error(
                format!("cannot find an array with name {}", name),
                format!("{} has type", found_type),
                span,
            ),
            RuntimeErrorKind::UnstructuredError { message } => {
                Diagnostic::simple_error("".to_owned(), message.to_string(), span)
            }
            RuntimeErrorKind::UnsupportedOp {
                op,
                first_type,
                second_type,
            } => Diagnostic::simple_error(
                "unsupported operation".to_owned(),
                format!(
                    "no support for {} with types {} and {}",
                    op, first_type, second_type
                ),
                span,
            ),
            RuntimeErrorKind::Spanless(message) => Diagnostic::from_message(message),
            RuntimeErrorKind::Unimplemented(message) => Diagnostic::from_message(message),
            RuntimeErrorKind::FunctionNonMainContext { func_name } => Diagnostic::simple_error(
                "cannot call function outside of main".to_owned(),
                format!("function {} can only be called in main", func_name),
                span,
            ),
        }
    }
}
