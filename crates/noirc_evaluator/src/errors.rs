use noirc_errors::{CustomDiagnostic as Diagnostic, FileDiagnostic, Location};
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

    // Keep one of the two location which is Some, if possible
    // This is used when we optimize instructions so that we do not lose track of location
    pub fn merge_location(a: Option<Location>, b: Option<Location>) -> Option<Location> {
        match (a, b) {
            (Some(loc), _) | (_, Some(loc)) => Some(loc),
            (None, None) => None,
        }
    }
}

impl From<RuntimeErrorKind> for RuntimeError {
    fn from(kind: RuntimeErrorKind) -> RuntimeError {
        RuntimeError { location: None, kind }
    }
}

impl From<RuntimeError> for FileDiagnostic {
    fn from(err: RuntimeError) -> Self {
        let file_id = err.location.map(|loc| loc.file).unwrap();
        FileDiagnostic { file_id, diagnostic: err.into() }
    }
}

#[derive(Error, Debug)]
pub enum RuntimeErrorKind {
    // Array errors
    #[error("Out of bounds")]
    ArrayOutOfBounds { index: u128, bound: u128 },

    #[error("index out of bounds: the len is {index} but the index is {bound}")]
    IndexOutOfBounds { index: u32, bound: u128 },

    #[error("cannot call {func_name} function in non main function")]
    FunctionNonMainContext { func_name: String },

    // Environment errors
    #[error("Cannot find Array")]
    ArrayNotFound { found_type: String, name: String },

    #[error("Not an object")]
    NotAnObject,

    #[error("Invalid id")]
    InvalidId,

    #[error("Attempt to divide by zero")]
    DivisionByZero,

    #[error("Failed range constraint when constraining to {0} bits")]
    FailedRangeConstraint(u32),

    #[error("Unsupported integer size of {num_bits} bits. The maximum supported size is {max_num_bits} bits.")]
    UnsupportedIntegerSize { num_bits: u32, max_num_bits: u32 },

    #[error("Failed constraint")]
    FailedConstraint,

    #[error(
        "All Witnesses are by default u{0}. Applying this type does not apply any constraints."
    )]
    DefaultWitnesses(u32),

    #[error("Constraint is always false")]
    ConstraintIsAlwaysFalse { spanless: bool },

    #[error("ICE: cannot convert signed {0} bit size into field")]
    CannotConvertSignedIntoField(u32),

    #[error("we do not allow private ABI inputs to be returned as public outputs")]
    PrivateAbiInput,

    #[error("unimplemented")]
    Unimplemented(String),

    #[error("Unsupported operation error")]
    UnsupportedOp { op: String, first_type: String, second_type: String },
}

impl From<RuntimeError> for Diagnostic {
    fn from(error: RuntimeError) -> Diagnostic {
        let span =
            if let Some(loc) = error.location { loc.span } else { noirc_errors::Span::new(0..0) };
        match &error.kind {
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
            RuntimeErrorKind::NotAnObject
            | RuntimeErrorKind::InvalidId
            | RuntimeErrorKind::DivisionByZero
            | RuntimeErrorKind::FailedRangeConstraint(_)
            | RuntimeErrorKind::UnsupportedIntegerSize { .. }
            | RuntimeErrorKind::FailedConstraint
            | RuntimeErrorKind::DefaultWitnesses(_)
            | RuntimeErrorKind::ConstraintIsAlwaysFalse { spanless: false }
            | RuntimeErrorKind::CannotConvertSignedIntoField(_)
            | RuntimeErrorKind::IndexOutOfBounds { .. }
            | RuntimeErrorKind::PrivateAbiInput => {
                Diagnostic::simple_error("".to_owned(), error.kind.to_string(), span)
            }
            RuntimeErrorKind::UnsupportedOp { op, first_type, second_type } => {
                Diagnostic::simple_error(
                    "unsupported operation".to_owned(),
                    format!("no support for {op} with types {first_type} and {second_type}"),
                    span,
                )
            }
            RuntimeErrorKind::ConstraintIsAlwaysFalse { spanless: true } => {
                Diagnostic::from_message(&error.kind.to_string())
            }
            RuntimeErrorKind::Unimplemented(message) => Diagnostic::from_message(message),
            RuntimeErrorKind::FunctionNonMainContext { func_name } => Diagnostic::simple_error(
                "cannot call function outside of main".to_owned(),
                format!("function {func_name} can only be called in main"),
                span,
            ),
        }
    }
}
