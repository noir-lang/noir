use crate::hir::def_collector::dc_crate::CompilationError;
use crate::parser::ParserError;
use crate::parser::ParserErrorReason;

use super::token::LocatedToken;
use super::token::Token;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::Location;
use thiserror::Error;

#[derive(Error, Clone, Debug, PartialEq, Eq)]
pub enum LexerErrorKind {
    #[error("An unexpected character {:?} was found.", found)]
    UnexpectedCharacter { location: Location, expected: String, found: Option<char> },
    #[error("Internal error: Tried to lex {:?} as a double char token", found)]
    NotADoubleChar { location: Location, found: Token },
    #[error("Invalid integer literal, {:?} is not a integer", found)]
    InvalidIntegerLiteral { location: Location, found: String },
    #[error("Integer literal is too large")]
    IntegerLiteralTooLarge { location: Location, limit: String },
    #[error("{:?} is not a valid attribute", found)]
    MalformedFuncAttribute { location: Location, found: String },
    #[error("Malformed test attribute")]
    MalformedTestAttribute { location: Location },
    #[error("Malformed fuzz attribute")]
    MalformedFuzzAttribute { location: Location },
    #[error("{:?} is not a valid inner attribute", found)]
    InvalidInnerAttribute { location: Location, found: String },
    #[error("Unterminated block comment")]
    UnterminatedBlockComment { location: Location },
    #[error("Unterminated string literal")]
    UnterminatedStringLiteral { location: Location },
    #[error("Invalid format string: expected '}}', found {found:?}")]
    InvalidFormatString { found: char, location: Location },
    #[error("Invalid format string: expected letter or underscore, found '}}'")]
    EmptyFormatStringInterpolation { location: Location },
    #[error(
        "'\\{escaped}' is not a valid escape sequence. Use '\\' for a literal backslash character."
    )]
    InvalidEscape { escaped: char, location: Location },
    #[error("Invalid quote delimiter `{delimiter}`, valid delimiters are `{{`, `[`, and `(`")]
    InvalidQuoteDelimiter { delimiter: LocatedToken },
    #[error("Non-ASCII characters are invalid in comments")]
    NonAsciiComment { location: Location },
    #[error("Expected `{end_delim}` to close this {start_delim}")]
    UnclosedQuote { start_delim: LocatedToken, end_delim: Token },
    #[error("Unicode character '{}' looks like space, but is it not", char)]
    UnicodeCharacterLooksLikeSpaceButIsItNot { char: char, location: Location },
    #[error(
        "Invalid form of the `must_use` attribute. Valid forms are `#[must_use]` and `#[must_use = \"message\"]`"
    )]
    MalformedMustUseAttribute { location: Location },
}

impl From<LexerErrorKind> for ParserError {
    fn from(value: LexerErrorKind) -> Self {
        let location = value.location();
        ParserError::with_reason(ParserErrorReason::Lexer(value), location)
    }
}

impl From<LexerErrorKind> for CompilationError {
    fn from(error: LexerErrorKind) -> Self {
        ParserError::from(error).into()
    }
}

impl LexerErrorKind {
    pub fn location(&self) -> Location {
        match self {
            LexerErrorKind::UnexpectedCharacter { location, .. } => *location,
            LexerErrorKind::NotADoubleChar { location, .. }
            | LexerErrorKind::InvalidIntegerLiteral { location, .. }
            | LexerErrorKind::IntegerLiteralTooLarge { location, .. }
            | LexerErrorKind::MalformedFuncAttribute { location, .. }
            | LexerErrorKind::MalformedTestAttribute { location, .. }
            | LexerErrorKind::MalformedFuzzAttribute { location, .. }
            | LexerErrorKind::InvalidInnerAttribute { location, .. }
            | LexerErrorKind::UnterminatedBlockComment { location }
            | LexerErrorKind::UnterminatedStringLiteral { location }
            | LexerErrorKind::InvalidFormatString { location, .. }
            | LexerErrorKind::EmptyFormatStringInterpolation { location, .. }
            | LexerErrorKind::InvalidEscape { location, .. }
            | LexerErrorKind::NonAsciiComment { location, .. }
            | LexerErrorKind::MalformedMustUseAttribute { location }
            | LexerErrorKind::UnicodeCharacterLooksLikeSpaceButIsItNot { location, .. } => {
                *location
            }
            LexerErrorKind::InvalidQuoteDelimiter { delimiter } => delimiter.location(),
            LexerErrorKind::UnclosedQuote { start_delim, .. } => start_delim.location(),
        }
    }

    fn parts(&self) -> (String, String, Location) {
        match self {
            LexerErrorKind::UnexpectedCharacter {
                location,
                expected,
                found,
            } => {
                let found: String = found.map(Into::into).unwrap_or_else(|| "<eof>".into());

                (
                    "An unexpected character was found".to_string(),
                    format!("Expected {expected}, but found {found}"),
                    *location,
                )
            },
            LexerErrorKind::NotADoubleChar { location, found } => (
                format!("Tried to parse {found} as double char"),
                format!(
                    " {found:?} is not a double char, this is an internal error"
                ),
                *location,
            ),
            LexerErrorKind::InvalidIntegerLiteral { location, found } => (
                "Invalid integer literal".to_string(),
                format!(" {found} is not an integer"),
                *location,
            ),
            LexerErrorKind::IntegerLiteralTooLarge { location, limit } => (
                "Integer literal is too large".to_string(),
                format!("value exceeds limit of {limit}"),
                *location,
            ),
            LexerErrorKind::MalformedFuncAttribute { location, found } => (
                "Malformed function attribute".to_string(),
                format!(" {found} is not a valid attribute"),
                *location,
            ),
            LexerErrorKind::MalformedTestAttribute { location } => (
                "Malformed test attribute".to_string(),
                "The test attribute can be written in one of these forms: `#[test]`, `#[test(should_fail)]` or `#[test(should_fail_with = \"message\")]`".to_string(),
                *location,
            ),
            LexerErrorKind::MalformedFuzzAttribute { location } => (
                "Malformed fuzz attribute".to_string(),
                "The fuzz attribute can be written in one of these forms: `#[fuzz]`, `#[fuzz(should_fail)]`, `#[fuzz(should_fail_with = \"message\")]` or `#[fuzz(only_fail_with = \"message\")]`".to_string(),
                *location,
            ),
            LexerErrorKind::InvalidInnerAttribute { location, found } => (
                "Invalid inner attribute".to_string(),
                format!(" {found} is not a valid inner attribute"),
                *location,
            ),
            LexerErrorKind::UnterminatedBlockComment { location } => ("Unterminated block comment".to_string(), "Unterminated block comment".to_string(), *location),
            LexerErrorKind::UnterminatedStringLiteral { location } =>
                ("Unterminated string literal".to_string(), "Unterminated string literal".to_string(), *location),
            LexerErrorKind::InvalidFormatString { found, location } => {
                if found == &'}' {
                    (
                        "Invalid format string: unmatched '}}' found".to_string(),
                        "If you intended to print '}', you can escape it using '}}'".to_string(),
                        *location,
                    )
                } else {
                    (
                        format!("Invalid format string: expected '}}', found {found:?}"),
                        if found == &'.' {
                            "Field access isn't supported in format strings".to_string()
                        } else {
                            "If you intended to print '{', you can escape it using '{{'".to_string()
                        },
                        *location,
                    )
                }
            }
            LexerErrorKind::EmptyFormatStringInterpolation { location } => {
                (
                    "Invalid format string: expected letter or underscore, found '}}'".to_string(),
                    "If you intended to print '{' or '}', you can escape them using '{{' and '}}' respectively".to_string(),
                    *location,
                )
            }
            LexerErrorKind::InvalidEscape { escaped, location } =>
                (format!("'\\{escaped}' is not a valid escape sequence. Use '\\' for a literal backslash character."), "Invalid escape sequence".to_string(), *location),
            LexerErrorKind::InvalidQuoteDelimiter { delimiter } => {
                (format!("Invalid quote delimiter `{delimiter}`"), "Valid delimiters are `{`, `[`, and `(`".to_string(), delimiter.location())
            },
            LexerErrorKind::NonAsciiComment { location } => {
                ("Non-ASCII character in comment".to_string(), "Invalid comment character: only ASCII is currently supported.".to_string(), *location)
            }
            LexerErrorKind::UnclosedQuote { start_delim, end_delim } => {
                ("Unclosed `quote` expression".to_string(), format!("Expected a `{end_delim}` to close this `{start_delim}`"), start_delim.location())
            }
            LexerErrorKind::UnicodeCharacterLooksLikeSpaceButIsItNot { char, location } => {
                // List taken from https://en.wikipedia.org/wiki/Whitespace_character
                let char_name = match char {
                    '\u{0085}' => Some("Next Line"),
                    '\u{00A0}' => Some("No-Break Space"),
                    '\u{1680}' => Some("Ogham Space Mark"),
                    '\u{2000}' => Some("En Quad"),
                    '\u{2001}' => Some("Em Quad"),
                    '\u{2002}' => Some("En Space"),
                    '\u{2003}' => Some("Em Space"),
                    '\u{2004}' => Some("Three-Per-Em Space"),
                    '\u{2005}' => Some("Four-Per-Em Space"),
                    '\u{2006}' => Some("Six-Per-Em Space"),
                    '\u{2007}' => Some("Figure Space"),
                    '\u{2008}' => Some("Punctuation Space"),
                    '\u{2009}' => Some("Thin Space"),
                    '\u{200A}' => Some("Hair Space"),
                    '\u{2028}' => Some("Line Separator"),
                    '\u{2029}' => Some("Paragraph Separator"),
                    '\u{202F}' => Some("Narrow No-Break Space"),
                    '\u{205F}' => Some("Medium Mathematical Space"),
                    '\u{3000}' => Some("Ideographic Space"),
                    '\u{180E}' => Some("Mongolian Vowel Separator"),
                    '\u{200B}' => Some("Zero Width Space"),
                    '\u{200C}' => Some("Zero Width Non-Joiner"),
                    '\u{200D}' => Some("Zero Width Joiner"),
                    '\u{2060}' => Some("Word Joiner"),
                    '\u{FEFF}' => Some("Zero Width No-Break Space"), // cSpell:disable-line
                    _ => None,
                };

                let primary = format!("Unknown start of token: \\u{{{:x}}}", (*char as u32));
                let secondary = match char_name {
                    Some(name) => format!("Unicode character '{char}' ({name}) looks like ' ' (Space), but is it not"),
                    None => {
                        format!("Unicode character '{char}' looks like ' ' (Space), but is it not")
                    }
                };
                (primary, secondary, *location)
            }
            LexerErrorKind::MalformedMustUseAttribute { location } => {
                ("Invalid syntax for `must_use` attribute".to_string(), "Valid syntaxes are: `#[must_use]` and `#[must_use = \"message\"]`".to_string(), *location)
            },
        }
    }
}

impl<'a> From<&'a LexerErrorKind> for Diagnostic {
    fn from(error: &'a LexerErrorKind) -> Diagnostic {
        let (primary, secondary, span) = error.parts();
        Diagnostic::simple_error(primary, secondary, span)
    }
}
