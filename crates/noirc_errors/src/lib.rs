mod position;
mod reporter;

pub use position::{Location, Position, Span, Spanned};
pub use reporter::*;

pub trait DiagnosableError {
    fn to_diagnostic(&self) -> CustomDiagnostic;
}

/// Returned when the Reporter finishes after reporting errors
#[derive(Copy, Clone)]
pub struct ReportedError;

#[derive(Debug, PartialEq, Eq)]
pub struct CollectedErrors {
    pub file_id: fm::FileId,
    pub errors: Vec<CustomDiagnostic>,
}
