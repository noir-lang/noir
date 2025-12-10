#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]

mod position;
pub use position::{Position, Span, Spanned};
