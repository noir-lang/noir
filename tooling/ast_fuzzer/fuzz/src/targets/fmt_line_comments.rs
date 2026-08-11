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
use noirc_errors::Location;
use noirc_frontend::{
    lexer::Lexer,
    parser,
    token::{FmtStrFragment, Token},
};

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
        .map(|token| without_locations(token.expect("formatter output should lex").into_token()))
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

/// Erase the source positions a token carries, so that tokens compare on code alone.
///
/// A format string is the one token that records where in the source it was written: the lexer
/// stores the location of every `{...}` interpolation inside it. Inserting a comment moves the
/// code that follows, which changes those locations without changing the program.
fn without_locations(token: Token) -> Token {
    let Token::FmtStr(fragments, length) = token else {
        return token;
    };
    let fragments = fragments
        .into_iter()
        .map(|fragment| match fragment {
            FmtStrFragment::Interpolation(name, _) => {
                FmtStrFragment::Interpolation(name, Location::dummy())
            }
            fragment @ FmtStrFragment::String(_) => fragment,
        })
        .collect();
    Token::FmtStr(fragments, length)
}

#[cfg(test)]
mod tests {
    use super::{significant_tokens, try_format};

    /// A line comment shifts every token after it, and a format string carries the source
    /// location of each of its interpolations. Those locations are not code, so moving one
    /// must not read as the formatter having changed the program.
    #[test]
    fn line_comment_before_a_format_string_is_not_a_change() {
        let source = "unconstrained fn main() {\n    let x: u32 = 0;\n    println(f\"v{x}\");\n}\n";
        let baseline = try_format(source).expect("source should format");

        let boundary = baseline.find(';').expect("the let statement should be formatted") + 1;
        let mut with_comment = baseline.clone();
        with_comment.insert_str(boundary, " // fuzzed line comment\n");
        let formatted = try_format(&with_comment).expect("commented source should format");

        assert_eq!(significant_tokens(&formatted), significant_tokens(&baseline));
    }

    #[test]
    fn fuzz_with_arbtest() {
        crate::targets::tests::fuzz_with_arbtest(super::fuzz, 1000);
    }
}
