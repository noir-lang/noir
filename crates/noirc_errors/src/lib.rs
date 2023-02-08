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

/// Extension trait just to enable the syntax err.in_file(..) for CustomDiagnostic
pub trait IntoFileDiagnostic {
    fn in_file(self, id: fm::FileId) -> FileDiagnostic;
}

impl IntoFileDiagnostic for CustomDiagnostic {
    fn in_file(self, file_id: fm::FileId) -> FileDiagnostic {
        FileDiagnostic { file_id, diagnostic: self }
    }
}
