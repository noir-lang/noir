#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]

pub mod errors;

pub mod acir;
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
    let first_line_original_length = first_line.len();
    let mut result = first_line.trim_start().to_string();
    let first_line_trimmed_length = result.len();

    // Try to see how many spaces we chopped off the first line
    let difference = first_line_original_length - first_line_trimmed_length;
    for line in lines {
        result.push('\n');
        // Try to remove just `difference` spaces to preserve indents
        if line.len() - line.trim_start().len() >= difference {
            result.push_str(&line.chars().skip(difference).collect::<String>());
        } else {
            result.push_str(line.trim_start());
        }
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
