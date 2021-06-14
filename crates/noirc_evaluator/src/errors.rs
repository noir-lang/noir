use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::DiagnosableError;
use noirc_errors::Span;
use thiserror::Error;

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

impl DiagnosableError for RuntimeErrorKind {
    fn to_diagnostic(&self) -> Diagnostic {
        match self {
            RuntimeErrorKind::ArrayOutOfBounds { index, bound } => Diagnostic::simple_error(
                "index out of bounds".to_string(),
                format!(
                    "out of bounds error, index is {} but length is {}",
                    index, bound
                ),
                Span::default(),
            ),
            RuntimeErrorKind::ArrayNotFound { found_type, name } => Diagnostic::simple_error(
                format!("cannot find an array with name {}", name),
                format!("{} has type", found_type),
                Span::default(),
            ),
            RuntimeErrorKind::UnstructuredError { message } => {
                Diagnostic::simple_error("".to_owned(), message.to_string(), Span::default())
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
                Span::default(),
            ),
            RuntimeErrorKind::Spanless(message) => Diagnostic::from_message(message),
            RuntimeErrorKind::Unimplemented(message) => Diagnostic::from_message(message),
            RuntimeErrorKind::FunctionNonMainContext { func_name } => Diagnostic::simple_error(
                "cannot call function outside of main".to_owned(),
                format!("function {} can only be called in main", func_name),
                Span::default(),
            ),
        }
    }
}
