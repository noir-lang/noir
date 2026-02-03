#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]

pub mod call_stack;
mod position;
pub mod reporter;
pub use noirc_span::{Span, Spanned};
pub use position::{Located, Location, Position};
pub use reporter::{CustomDiagnostic, DiagnosticKind};
use std::io::Write;

pub fn print_args_or_exit<W: Write>(args: std::fmt::Arguments, mut out: W) {
    if let Err(e) = out.write_fmt(args) {
        if e.kind() == std::io::ErrorKind::BrokenPipe {
            // Gracefully exit on broken pipe
            std::process::exit(0);
        } else {
            panic!("Unexpected error: {e}");
        }
    }
}

/// Print the input to stdout, and exit gracefully if `SIGPIPE` is received.
/// Rust ignores `SIGPIPE` by default, converting pipe errors into `ErrorKind::BrokenPipe`
pub fn print_to_stdout(args: std::fmt::Arguments) {
    print_args_or_exit(args, std::io::stdout());
}

/// Print the input to stderr, and exit gracefully if `SIGPIPE` is received.
/// Rust ignores `SIGPIPE` by default, converting pipe errors into `ErrorKind::BrokenPipe`
pub fn print_to_stderr(args: std::fmt::Arguments) {
    print_args_or_exit(args, std::io::stderr());
}

/// Macro to print formatted output to stdout
#[macro_export]
macro_rules! print_to_stdout {
    ($($arg:tt)*) => {
        noirc_errors::print_to_stdout(format_args!($($arg)*))
    };
}

/// Macro to print formatted output to stdout
#[macro_export]
macro_rules! println_to_stdout {
    ($($arg:tt)*) => {
        noirc_errors::print_to_stdout(format_args!("{}\n", format!($($arg)*)))
    };
}

/// Macro to print formatted output to stderr
#[macro_export]
macro_rules! print_to_stderr {
    ($($arg:tt)*) => {
        noirc_errors::print_to_stderr(format_args!($($arg)*))
    };
}

/// Macro to print formatted output to stderr
#[macro_export]
macro_rules! println_to_stderr {
    ($($arg:tt)*) => {
        noirc_errors::print_to_stderr(format_args!("{}\n", format!($($arg)*)))
    };
}
