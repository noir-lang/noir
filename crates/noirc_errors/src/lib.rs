#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

mod position;
pub mod reporter;
pub use position::{Location, Position, Span, Spanned};
pub use reporter::{CustomDiagnostic, DiagnosticKind};
use serde::{Deserialize, Serialize};

/// Returned when the Reporter finishes after reporting errors
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct ReportedError;

#[derive(Debug, PartialEq, Eq)]
pub struct FileDiagnostic {
    pub file_id: fm::FileId,
    pub diagnostic: CustomDiagnostic,
}
