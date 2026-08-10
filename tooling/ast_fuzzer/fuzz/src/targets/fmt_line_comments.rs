//! Verify that line comments never cause the formatter to swallow code.
//!
//! The AST fuzzer generates a valid program, which is formatted once to establish a normalized
//! baseline. We then insert a line comment at a fuzz-selected token boundary, format again, and
//! require the non-comment token stream to remain equal to the baseline. This catches both invalid
//! formatter output and the more dangerous case where swallowed code still parses.

use arbitrary::Unstructured;
use color_eyre::eyre;
use nargo_fmt::{Config as FormatterConfig, format};
use noir_ast_fuzzer::{DisplayAstAsNoir, arb_program};
use noirc_frontend::{lexer::Lexer, parser, token::Token};

use super::default_config;

pub fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let config = default_config(u)?;
    let program = arb_program(u, config)?;
    let source = DisplayAstAsNoir(&program).to_string();
    let Some(baseline) = try_format(&source) else { return Ok(()) };

    let mut boundaries = significant_token_boundaries(&baseline);
    boundaries.pop(); // A comment after the final token cannot swallow code.
    if boundaries.is_empty() {
        return Ok(());
    }

    let boundary = boundaries[u.choose_index(boundaries.len())?];
    let mut with_comment = baseline.clone();
    with_comment.insert_str(boundary, " // fuzzed line comment\n");

    // A newline is not valid at every token boundary. Invalid mutations do not exercise the
    // formatter and are discarded like other unsuitable fuzz inputs.
    let Some(formatted) = try_format(&with_comment) else { return Ok(()) };

    let expected_tokens = significant_tokens(&baseline);
    let actual_tokens = significant_tokens(&formatted);
    if actual_tokens != expected_tokens {
        eyre::bail!(
            "formatter changed code tokens after a line comment at byte {boundary}\n\
             baseline:\n{baseline}\nwith comment:\n{with_comment}\nformatted:\n{formatted}"
        );
    }

    Ok(())
}

fn try_format(source: &str) -> Option<String> {
    let (parsed_module, errors) = parser::parse_program_with_dummy_file(source);
    if errors.iter().any(|error| !error.is_warning()) {
        return None;
    }
    Some(format(source, parsed_module, &FormatterConfig::default()))
}

fn significant_token_boundaries(source: &str) -> Vec<usize> {
    Lexer::new_with_dummy_file(source)
        .skip_comments(true)
        .skip_whitespaces(true)
        .filter_map(|token| {
            let token = token.expect("formatter output should lex");
            (!matches!(token.token(), Token::EOF)).then(|| token.span().end() as usize)
        })
        .collect()
}

fn significant_tokens(source: &str) -> Vec<Token> {
    let tokens: Vec<_> = Lexer::new_with_dummy_file(source)
        .skip_comments(true)
        .skip_whitespaces(true)
        .map(|token| token.expect("formatter output should lex").into_token())
        .collect();

    // A comment can make a group multiline, in which case the formatter may add a trailing comma.
    // Ignore those layout-only tokens while checking that all source code survived.
    (0..tokens.len())
        .filter_map(|index| {
            let token = &tokens[index];
            let is_layout_comma = *token == Token::Comma
                && matches!(
                    tokens.get(index + 1),
                    Some(Token::RightParen | Token::RightBracket | Token::RightBrace)
                );
            (!is_layout_comma).then(|| token.clone())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn fuzz_with_arbtest() {
        crate::targets::tests::fuzz_with_arbtest(super::fuzz, 1000);
    }
}
