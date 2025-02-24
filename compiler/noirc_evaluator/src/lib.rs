#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

pub mod errors;

mod acir;
pub mod brillig;
pub mod ssa;

pub use ssa::create_program;
pub use ssa::ir::instruction::ErrorType;

/// Trims leading whitespace from each line of the input string
#[cfg(test)]
pub(crate) fn trim_leading_whitespace_from_lines(src: &str) -> String {
    let mut lines = src.trim_end().lines();
    let mut first_line = lines.next().unwrap();
    while first_line.is_empty() {
        first_line = lines.next().unwrap();
    }
    let mut result = first_line.trim_start().to_string();
    for line in lines {
        result.push('\n');
        result.push_str(line.trim_start());
    }
    result
}

/// Trim comments from the lines, ie. content starting with `//`.
#[cfg(test)]
pub(crate) fn trim_comments_from_lines(src: &str) -> String {
    let mut result = String::new();
    let mut first = true;
    for line in src.lines() {
        if !first {
            result.push('\n');
        }
        if let Some(comment) = line.find("//") {
            result.push_str(line[..comment].trim_end());
        } else {
            result.push_str(line);
        }
        first = false;
    }
    result
}
