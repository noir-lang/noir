#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

//! The Noir formatter.
//!
//! It works by using two techniques:
//!
//! 1. Surfing a parsed module by using a lexer.
//! 2. Trying to not exceed the maximum line width by formatting to intermediate chunk (see chunks.rs)
//!
//! What is lexer surfing?
//!
//! Suppose we need to format this code:
//!
//! fn foo ( ) { let x : Field = ( 2 , 3 ) ; }
//!
//! We first parse the above code so we end up with a ParsedModule. Next we traverse this module
//! contents and process each item, statement, expression, type, etc., we find.
//!
//! For example, the first thing we'll find is a function. We know it has no visibility and no doc
//! comments so we can expect an `fn` keyword to be there. We write it. Next will come the identifier.
//! For that we "skip" any spaces and comments between `fn` and `foo`, writing only one space instead
//! of possibly multiple spaces, then write "foo". If there were comments between `fn` and `foo`,
//! we write them (the formatter will never lose comments).
//!
//! Next we know there are no generics, so we can expect a `(`, etc.
//!
//! In this way we go token by token, inserting newlines when needed, removing extra spaces,
//! indenting things as we go deep inside structures, etc.
//!
//! But that's not all. The formatter will try to not exceed the configurable maximum width.
//! It will do that but, for simplicity, only for function parameters list, statements and expressions
//! (we assume an `impl Foo ...` line won't exceed the maximum length, and if it does it's not a big deal,
//! or we can always improve things later). For this, read the comments in chunks.rs.
mod chunks;
mod config;
pub mod errors;
mod formatter;

use formatter::Formatter;
use noirc_frontend::ParsedModule;

pub use config::Config;

pub fn format(source: &str, parsed_module: ParsedModule, config: &Config) -> String {
    let mut formatter = Formatter::new(source, config);
    formatter.format_program(parsed_module);
    formatter.buffer.contents()
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
    let errors: Vec<_> = errors.into_iter().filter(|error| !error.is_warning()).collect();
    if !errors.is_empty() {
        panic!("Expected no errors, got: {:?}", errors);
    }
    let result = format(src, parsed_module, &config);
    if result != expected {
        println!("Expected:\n~~~\n{}\n~~~\nGot:\n~~~\n{}\n~~~", expected, result);
    }

    similar_asserts::assert_eq!(result, expected);

    let src = &result;
    let (parsed_module, errors) = parser::parse_program(src);
    if !errors.is_empty() {
        panic!("Expected no errors in idempotent check, got: {:?}", errors);
    }
    let result = format(src, parsed_module, &config);
    if result != expected {
        println!("Expected (idempotent):\n~~~\n{}\n~~~\nGot:\n~~~\n{}\n~~~", expected, result);
    }
    similar_asserts::assert_eq!(result, expected, "idempotent check failed");
}
