#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]

pub mod call_stack;
pub mod debug_info;
mod position;
pub mod reporter;
pub use position::{Located, Location, Position, Span, Spanned};
pub use reporter::{CustomDiagnostic, DiagnosticKind};
use std::io::Write;

/// Print the input to stdout, and exit gracefully if `SIGPIPE` is received.
/// Rust ignores `SIGPIPE` by default, converting pipe errors into `ErrorKind::BrokenPipe`
pub fn print_to_stdout(args: std::fmt::Arguments) {
    let mut stdout = std::io::stdout();
    if let Err(e) = stdout.write_fmt(args) {
        if e.kind() == std::io::ErrorKind::BrokenPipe {
            // Gracefully exit on broken pipe
            std::process::exit(0);
        } else {
            panic!("Unexpected error: {e}");
        }
    }
}
