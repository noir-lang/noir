#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

pub mod errors;

// SSA code to create the SSA based IR
// for functions and execute different optimizations.
pub mod ssa;

pub mod brillig;

pub use ssa::create_program;
pub use ssa::ir::instruction::ErrorType;

/// Trims leading whitespace from each line of the input string, according to
/// how much leading whitespace there is on the first non-empty line.
#[cfg(test)]
pub(crate) fn trim_leading_whitespace_from_lines(src: &str) -> String {
    let mut lines = src.trim_end().lines();
    let mut first_line = lines.next().unwrap();
    while first_line.is_empty() {
        first_line = lines.next().unwrap();
    }
    let indent = first_line.len() - first_line.trim_start().len();
    let mut result = first_line.trim_start().to_string();
    for line in lines {
        result.push('\n');
        result.push_str(&line[indent..]);
    }
    result
}
