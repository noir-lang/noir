#![cfg(test)]

use noirc_errors::Span;

use crate::parser::{ParserError, ParserErrorReason};

pub(super) fn get_source_with_error_span(src: &str) -> (String, Span) {
    let mut lines: Vec<&str> = src.trim_end().lines().collect();
    let squiggles_line = lines.pop().expect("Expected at least two lines in src (the last one should have squiggles for the error location)");
    let squiggle_index = squiggles_line
        .chars()
        .position(|char| char == '^')
        .expect("Expected at least one `^` character in the last line of the src");
    let squiggle_length = squiggles_line.len() - squiggle_index;
    let last_line = lines.last().expect("Expected at least two lines in src");
    let src = lines.join("\n");
    let span_start = src.len() - last_line.len() + squiggle_index;
    let span_end = span_start + squiggle_length;
    let span = Span::from(span_start as u32..span_end as u32);
    (src, span)
}

pub(super) fn get_single_error(errors: &[ParserError], expected_span: Span) -> &ParserError {
    if errors.is_empty() {
        panic!("Expected an error, found none");
    }

    if errors.len() > 1 {
        for error in errors {
            println!("{}", error);
        }
        panic!("Expected one error, found {} errors (printed above)", errors.len());
    }

    assert_eq!(errors[0].span(), expected_span);
    &errors[0]
}

pub(super) fn get_single_error_reason(
    errors: &[ParserError],
    expected_span: Span,
) -> &ParserErrorReason {
    get_single_error(errors, expected_span).reason().unwrap()
}

pub(super) fn expect_no_errors(errors: &[ParserError]) {
    if errors.is_empty() {
        return;
    }

    for error in errors {
        println!("{}", error);
    }
    panic!("Expected no errors, found {} errors (printed above)", errors.len());
}
