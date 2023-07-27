use acvm::FieldElement;
use noirc_errors::{CustomDiagnostic as Diagnostic, FileDiagnostic, Location};
use thiserror::Error;

// #[derive(Debug)]
// pub struct RuntimeError {
//     pub location: Option<Location>,
//     pub kind: RuntimeErrorKind,
// }

// impl RuntimeError {
//     // XXX: In some places, we strip the span because we do not want span to
//     // be introduced into the binary op or low level function code, for simplicity.
//     //
//     // It's possible to have it there, but it means we will need to proliferate the code with span
//     //
//     // This does make error reporting, less specific!
//     pub fn remove_span(self) -> RuntimeErrorKind {
//         self.kind
//     }

//     pub fn new(kind: RuntimeErrorKind, location: Option<Location>) -> RuntimeError {
//         RuntimeError { location, kind }
//     }

//     // Keep one of the two location which is Some, if possible
//     // This is used when we optimize instructions so that we do not lose track of location
//     pub fn merge_location(a: Option<Location>, b: Option<Location>) -> Option<Location> {
//         match (a, b) {
//             (Some(loc), _) | (_, Some(loc)) => Some(loc),
//             (None, None) => None,
//         }
//     }
// }

// impl From<RuntimeErrorKind> for RuntimeError {
//     fn from(kind: RuntimeErrorKind) -> RuntimeError {
//         RuntimeError { location: None, kind }
//     }
// }

// impl From<RuntimeError> for FileDiagnostic {
//     fn from(err: RuntimeError) -> Self {
//         let file_id = err.location.map(|loc| loc.file).unwrap();
//         FileDiagnostic { file_id, diagnostic: err.into() }
//     }
// }

// #[derive(Error, Debug)]
// pub enum RuntimeErrorKind {
//     // Array errors
//     #[error("Out of bounds")]
//     ArrayOutOfBounds { index: u128, bound: u128 },

//     #[error("cannot call {func_name} function in non main function")]
//     FunctionNonMainContext { func_name: String },

//     // Environment errors
//     #[error("Cannot find Array")]
//     ArrayNotFound { found_type: String, name: String },

//     #[error("Not an object")]
//     NotAnObject,

//     #[error("Invalid id")]
//     InvalidId,

//     #[error("Attempt to divide by zero")]
//     DivisionByZero,

//     #[error(
//         "All Witnesses are by default u{0}. Applying this type does not apply any constraints."
//     )]
//     DefaultWitnesses(u32),

//     #[error("Constraint is always false")]
//     ConstraintIsAlwaysFalse,

//     #[error("ICE: cannot convert signed {0} bit size into field")]
//     CannotConvertSignedIntoField(u32),

//     #[error("we do not allow private ABI inputs to be returned as public outputs")]
//     PrivateAbiInput,

//     #[error("unimplemented")]
//     Unimplemented(String),

//     #[error("Unsupported operation error")]
//     UnsupportedOp { op: String, first_type: String, second_type: String },

//     #[error(transparent)]
//     AcirGenError(#[from] AcirGenError),
// }

// impl From<RuntimeError> for Diagnostic {
//     fn from(error: RuntimeError) -> Diagnostic {
//         let span =
//             if let Some(loc) = error.location { loc.span } else { noirc_errors::Span::new(0..0) };
//         match &error.kind {
//             RuntimeErrorKind::ArrayOutOfBounds { index, bound } => Diagnostic::simple_error(
//                 "index out of bounds".to_string(),
//                 format!("out of bounds error, index is {index} but length is {bound}"),
//                 span,
//             ),
//             RuntimeErrorKind::ArrayNotFound { found_type, name } => Diagnostic::simple_error(
//                 format!("cannot find an array with name {name}"),
//                 format!("{found_type} has type"),
//                 span,
//             ),
//             RuntimeErrorKind::NotAnObject
//             | RuntimeErrorKind::InvalidId
//             | RuntimeErrorKind::DivisionByZero
//             | RuntimeErrorKind::AcirGenError(_)
//             | RuntimeErrorKind::DefaultWitnesses(_)
//             | RuntimeErrorKind::CannotConvertSignedIntoField(_)
//             | RuntimeErrorKind::PrivateAbiInput => {
//                 Diagnostic::simple_error("".to_owned(), error.kind.to_string(), span)
//             }
//             RuntimeErrorKind::UnsupportedOp { op, first_type, second_type } => {
//                 Diagnostic::simple_error(
//                     "unsupported operation".to_owned(),
//                     format!("no support for {op} with types {first_type} and {second_type}"),
//                     span,
//                 )
//             }
//             RuntimeErrorKind::ConstraintIsAlwaysFalse if error.location.is_some() => {
//                 Diagnostic::simple_error("".to_owned(), error.kind.to_string(), span)
//             }
//             RuntimeErrorKind::ConstraintIsAlwaysFalse => {
//                 Diagnostic::from_message(&error.kind.to_string())
//             }
//             RuntimeErrorKind::Unimplemented(message) => Diagnostic::from_message(message),
//             RuntimeErrorKind::FunctionNonMainContext { func_name } => Diagnostic::simple_error(
//                 "cannot call function outside of main".to_owned(),
//                 format!("function {func_name} can only be called in main"),
//                 span,
//             ),
//         }
//     }
// }

#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub enum RuntimeError {
    // We avoid showing the actual lhs and rhs since most of the time they are just 0
    // and 1 respectively. This would confuse users if a constraint such as
    // assert(foo < bar) fails with "failed constraint: 0 = 1."
    #[error("Failed constraint")]
    FailedConstraint { lhs: FieldElement, rhs: FieldElement, location: Option<Location> },
    #[error(transparent)]
    ICEError(#[from] ICEError),
    #[error("Index out of bounds, array has size {index:?}, but index was {array_size:?}")]
    IndexOutOfBounds { index: usize, array_size: usize, location: Option<Location> },
    #[error("All Witnesses are by default u{num_bits:?} Applying this type does not apply any constraints.\n We also currently do not allow integers of size more than {num_bits:?}, this will be handled by BigIntegers.")]
    InvalidRangeConstraint { num_bits: u32, location: Option<Location> },
    #[error("Expected array index to fit into a u64")]
    TypeConversion { from: String, into: String, location: Option<Location> },
    #[error("{name:?} is not initialized")]
    UnInitialized { name: String, location: Option<Location> },
    #[error("Integer sized {num_bits:?} is over the max supported size of {max_num_bits:?}")]
    UnsupportedIntegerSize { num_bits: u32, max_num_bits: u32, location: Option<Location> },
}

#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub enum ICEError {
    #[error("ICE: Both expressions are reduced to be degree<=1")]
    DegreeNotReduced { location: Option<Location> },
    #[error("ICE: {message:?}")]
    General { message: String, location: Option<Location> },
    #[error("ICE: {name:?} missing {arg:?} arg")]
    MissingArg { name: String, arg: String, location: Option<Location> },
    #[error("ICE: {name:?} should be a constant")]
    NotAConstant { name: String, location: Option<Location> },
    #[error("{name:?} is not implmented yet")]
    NotImplemented { name: String, location: Option<Location> },
    #[error("ICE: Undeclared AcirVar")]
    UndeclaredAcirVar { location: Option<Location> },
    #[error("ICE: Expected {expected:?}, found {found:?}")]
    UnExpected { expected: String, found: String, location: Option<Location> },
}

impl From<RuntimeError> for FileDiagnostic {
    fn from(error: RuntimeError) -> Self {
        match error {
            RuntimeError::ICEError(ref ice_error) => match ice_error {
                ICEError::DegreeNotReduced { location }
                | ICEError::General { location, .. }
                | ICEError::MissingArg { location, .. }
                | ICEError::NotAConstant { location, .. }
                | ICEError::NotImplemented { location, .. }
                | ICEError::UndeclaredAcirVar { location }
                | ICEError::UnExpected { location, .. } => {
                    let file_id = location.map(|loc| loc.file).unwrap();
                    FileDiagnostic { file_id, diagnostic: error.into() }
                }
            },
            RuntimeError::FailedConstraint { location, .. }
            | RuntimeError::IndexOutOfBounds { location, .. }
            | RuntimeError::InvalidRangeConstraint { location, .. }
            | RuntimeError::TypeConversion { location, .. }
            | RuntimeError::UnInitialized { location, .. }
            | RuntimeError::UnsupportedIntegerSize { location, .. } => {
                let file_id = location.map(|loc| loc.file).unwrap();
                FileDiagnostic { file_id, diagnostic: error.into() }
            }
        }
    }
}

impl From<RuntimeError> for Diagnostic {
    fn from(error: RuntimeError) -> Diagnostic {
        match error {
            RuntimeError::ICEError(_) => Diagnostic::simple_error(
                "Internal Consistency Evaluators Errors \n This is likely a bug. Consider Openning an issue at https://github.com/noir-lang/noir/issues".to_owned(),
                "".to_string(),
                noirc_errors::Span::new(0..0)
            ),
            RuntimeError::FailedConstraint { location, .. }
            | RuntimeError::IndexOutOfBounds { location, .. }
            | RuntimeError::InvalidRangeConstraint { location, .. }
            | RuntimeError::TypeConversion { location, .. }
            | RuntimeError::UnInitialized { location, .. }
            | RuntimeError::UnsupportedIntegerSize { location, .. }  => {
                let span = if let Some(loc) = location { loc.span } else { noirc_errors::Span::new(0..0) };
                Diagnostic::simple_error("".to_owned(), error.to_string(), span)
            }
        }
    }
}
