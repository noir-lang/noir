mod position;
mod reporter;

pub use position::{Span, Position, Spanned};
pub use reporter::*;

pub trait DiagnosableError {
    fn to_diagnostic(&self) -> CustomDiagnostic;
}