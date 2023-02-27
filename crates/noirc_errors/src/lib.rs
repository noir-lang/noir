#![forbid(unsafe_code)]
#![warn(unreachable_pub)]

mod position;
pub mod reporter;
pub use position::{Location, Position, Span, Spanned};
pub use reporter::{CustomDiagnostic, DiagnosticKind};

/// Returned when the Reporter finishes after reporting errors
#[derive(Copy, Clone)]
pub struct ReportedError;

#[derive(Debug, PartialEq, Eq)]
pub struct FileDiagnostic {
    pub file_id: fm::FileId,
    pub diagnostic: CustomDiagnostic,
}
