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

    /// An optional call stack to display the full runtime call stack
    /// leading up to a runtime error. If this is empty it will not be displayed.
    pub call_stack: Vec<Location>,
}

impl FileDiagnostic {
    pub fn new(file_id: fm::FileId, diagnostic: CustomDiagnostic) -> FileDiagnostic {
        FileDiagnostic { file_id, diagnostic, call_stack: Vec::new() }
    }

    pub fn with_call_stack(mut self, call_stack: Vec<Location>) -> Self {
        self.call_stack = call_stack;
        self
    }
}

impl From<FileDiagnostic> for Vec<FileDiagnostic> {
    fn from(value: FileDiagnostic) -> Self {
        vec![value]
    }
}
