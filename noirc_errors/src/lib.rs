mod position;
mod reporter;

pub use position::{Position, Span, Spanned};
pub use reporter::*;

pub trait DiagnosableError {
    fn to_diagnostic(&self) -> CustomDiagnostic;
}
