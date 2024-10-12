#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

/// A Rust code formatting utility designed to manage and format untouched fragments of source code,
/// including comments, whitespace, and other characters. While the module doesn't directly address comments,
/// it treats them as unchanged fragments, ensuring their original placement and content remain preserved.
///
/// Key methods include:
/// - `format_missing`: Addresses characters between the last processed position and a given end position,
///   capturing comments and other untouched sequences.
/// - `format_missing_indent`: Functions similarly to `format_missing`, but introduces added indentation.
/// - `format_missing_inner`: The core method for handling missing fragments, appending them to the output buffer.
///   Pure whitespace fragments might be replaced or adjusted based on context.
/// - `push_vertical_spaces`: Standardizes vertical spacing, eliminating potential excessive empty lines
///   or ensuring adequate vertical separation.
///
/// By recognizing and properly handling these untouched fragments, the utility ensures comments remain intact
/// in both placement and content during the formatting process.
mod config;
pub mod errors;
mod formatter;

use formatter::Formatter;
use noirc_frontend::ParsedModule;

pub use config::Config;

pub fn format(source: &str, parsed_module: ParsedModule, config: &Config) -> String {
    let mut formatter = Formatter::new(source, config);
    formatter.format_program(parsed_module);
    formatter.buffer
}

#[cfg(test)]
pub(crate) fn assert_format(src: &str, expected: &str) {
    assert_format_with_config(src, expected, Config::default());
}

#[cfg(test)]
pub(crate) fn assert_format_with_max_width(src: &str, expected: &str, max_width: usize) {
    let config = Config { max_width, ..Config::default() };
    assert_format_with_config(src, expected, config);
}

#[cfg(test)]
pub(crate) fn assert_format_with_config(src: &str, expected: &str, config: Config) {
    use noirc_frontend::parser;

    let (parsed_module, errors) = parser::parse_program(src);
    if !errors.is_empty() {
        panic!("Expected no errors, got: {:?}", errors);
    }
    let result = format(src, parsed_module, &config);
    similar_asserts::assert_eq!(result, expected);

    let src = &result;
    let (parsed_module, errors) = parser::parse_program(src);
    if !errors.is_empty() {
        panic!("Expected no errors in idempotent check, got: {:?}", errors);
    }
    let result = format(src, parsed_module, &config);
    similar_asserts::assert_eq!(result, expected, "idempotent check failed");
}
