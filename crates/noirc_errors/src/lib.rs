#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

mod position;
pub mod reporter;
pub use position::{Location, Position, Span, Spanned};
pub use reporter::{CustomDiagnostic, DiagnosticKind};
use serde::{Deserialize, Serialize};

pub enum Error {
    /// Indicates that the error has already been reported.
    Reported,
    /// Represents a custom diagnostic error.
    Diagnostic(CustomDiagnostic),
}

impl Error {
    pub fn message(msg: &str) -> Error {
        Error::Diagnostic(CustomDiagnostic::from_message(msg))
    }
}

impl From<ReportedError> for Error {
    fn from(_: ReportedError) -> Self {
        Error::Reported
    }
}

/// Returned when the Reporter finishes after reporting errors
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct ReportedError;

#[derive(Debug, PartialEq, Eq)]
pub struct FileDiagnostic {
    pub file_id: fm::FileId,
    pub diagnostic: CustomDiagnostic,
}
