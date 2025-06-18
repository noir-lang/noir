#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]

pub mod call_stack;
pub mod debug_info;
mod position;
pub mod reporter;
pub use position::{Located, Location, Position, Span, Spanned};
pub use reporter::{CustomDiagnostic, DiagnosticKind};
