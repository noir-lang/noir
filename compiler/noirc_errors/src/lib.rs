#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

pub mod debug_info;
mod position;
pub mod reporter;
pub use position::{Location, Position, Span, Spanned};
pub use reporter::{CustomDiagnostic, DiagnosticKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiagnostic {
    pub file_id: fm::FileId,
    pub diagnostic: CustomDiagnostic,
}

impl FileDiagnostic {
    pub fn new(file_id: fm::FileId, diagnostic: CustomDiagnostic) -> FileDiagnostic {
        FileDiagnostic { file_id, diagnostic }
    }
}

impl From<FileDiagnostic> for Vec<FileDiagnostic> {
    fn from(value: FileDiagnostic) -> Self {
        vec![value]
    }
}
