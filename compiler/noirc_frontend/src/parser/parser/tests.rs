#![cfg(test)]

use noirc_errors::{CustomDiagnostic, Span};

use crate::parser::{Parser, ParserError};

/// Run `parse` on `src` (which may contain inline `^^^ message` annotations) and verify
/// that the parser produces exactly those errors at those locations. Returns the value
/// returned by `parse` so the test can make further assertions on the parsed AST.
///
/// An annotation line directly follows the line it refers to, for example:
///
/// ```ignore
/// let src = "
///     { let x = 1 }
///               ^ Expected a ; after `let` statement
///     ";
/// check_errors(src, |parser| parser.parse_expression_or_error());
/// ```
///
/// The caret(s) indicate the span of the error, and the text that follows is the
/// expected primary diagnostic message produced by the parser at that span.
pub(super) fn check_errors<T>(src: &str, parse: impl FnOnce(&mut Parser<'_>) -> T) -> T {
    let mut code_lines = Vec::new();
    let mut expected_errors: Vec<(Span, String)> = Vec::new();

    // Trim trailing whitespace so that "end of input" error spans line up with the last
    // code character the test wrote, rather than the indentation of the closing `";`.
    let trimmed = src.trim_end();

    let mut byte = 0;
    let mut last_line_length = 0;
    for line in trimmed.lines() {
        if let Some((span, message)) = get_error_line_span_and_message(line, byte, last_line_length)
        {
            expected_errors.push((span, message));
            continue;
        }

        code_lines.push(line);
        byte += line.len() + 1; // For '\n'
        last_line_length = line.len();
    }

    let src = code_lines.join("\n");
    let mut parser = Parser::for_str_with_dummy_file(&src);
    let result = parse(&mut parser);

    let mut actual_errors: Vec<(Span, String)> = parser
        .errors
        .iter()
        .map(|error| {
            let diagnostic: CustomDiagnostic = error.into();
            let span = diagnostic.secondaries.first().map_or_else(
                || panic!("Expected error to have a secondary label: {error}"),
                |label| label.location.span,
            );
            (span, diagnostic.message)
        })
        .collect();

    expected_errors.sort();
    actual_errors.sort();

    if expected_errors != actual_errors {
        panic!(
            "Parser errors didn't match annotations.\n\nExpected:\n{}\n\nActual:\n{}\n",
            format_errors(&expected_errors),
            format_errors(&actual_errors),
        );
    }

    result
}

/// Helper for `check_errors` that returns the span of the `^^^^` annotation, together
/// with the message that follows it on the same line.
fn get_error_line_span_and_message(
    line: &str,
    byte: usize,
    last_line_length: usize,
) -> Option<(Span, String)> {
    if !line.trim().starts_with('^') {
        return None;
    }

    let chars = line.chars().collect::<Vec<_>>();
    let first_caret = chars.iter().position(|c| *c == '^').unwrap();
    let last_caret = chars.iter().rposition(|c| *c == '^').unwrap();
    let start = byte - last_line_length;
    let span = Span::from((start + first_caret - 1) as u32..(start + last_caret) as u32);
    let message = line.trim().trim_start_matches('^').trim().to_string();
    Some((span, message))
}

fn format_errors(errors: &[(Span, String)]) -> String {
    if errors.is_empty() {
        return "  (no errors)".to_string();
    }
    errors.iter().map(|(span, msg)| format!("  {span:?}: {msg}")).collect::<Vec<_>>().join("\n")
}

pub(super) fn expect_no_errors(errors: &[ParserError]) {
    if errors.is_empty() || errors.iter().all(|error| error.is_warning()) {
        return;
    }

    for error in errors {
        println!("{error}");
    }
    panic!("Expected no errors, found {} errors (printed above)", errors.len());
}
