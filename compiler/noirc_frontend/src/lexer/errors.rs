use crate::hir::def_collector::dc_crate::CompilationError;
use crate::parser::ParserError;
use crate::parser::ParserErrorReason;

use super::token::LocatedToken;
use super::token::Token;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::Location;
use noirc_errors::Span;
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
    #[error("{:?} is not a valid inner attribute", found)]
    InvalidInnerAttribute { location: Location, found: String },
    #[error("Logical and used instead of bitwise and")]
    LogicalAnd { location: Location },
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
            LexerErrorKind::NotADoubleChar { location, .. } => *location,
            LexerErrorKind::InvalidIntegerLiteral { location, .. } => *location,
            LexerErrorKind::IntegerLiteralTooLarge { location, .. } => *location,
            LexerErrorKind::MalformedFuncAttribute { location, .. } => *location,
            LexerErrorKind::MalformedTestAttribute { location, .. } => *location,
            LexerErrorKind::InvalidInnerAttribute { location, .. } => *location,
            LexerErrorKind::LogicalAnd { location } => *location,
            LexerErrorKind::UnterminatedBlockComment { location } => *location,
            LexerErrorKind::UnterminatedStringLiteral { location } => *location,
            LexerErrorKind::InvalidFormatString { location, .. } => *location,
            LexerErrorKind::EmptyFormatStringInterpolation { location, .. } => *location,
            LexerErrorKind::InvalidEscape { location, .. } => *location,
            LexerErrorKind::InvalidQuoteDelimiter { delimiter } => delimiter.to_location(),
            LexerErrorKind::NonAsciiComment { location, .. } => *location,
            LexerErrorKind::UnclosedQuote { start_delim, .. } => start_delim.to_location(),
        }
    }

    fn parts(&self) -> (String, String, Span) {
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
                    location.span,
                )
            },
            LexerErrorKind::NotADoubleChar { location, found } => (
                format!("Tried to parse {found} as double char"),
                format!(
                    " {found:?} is not a double char, this is an internal error"
                ),
                location.span,
            ),
            LexerErrorKind::InvalidIntegerLiteral { location, found } => (
                "Invalid integer literal".to_string(),
                format!(" {found} is not an integer"),
                location.span,
            ),
            LexerErrorKind::IntegerLiteralTooLarge { location, limit } => (
                "Integer literal is too large".to_string(),
                format!("value exceeds limit of {limit}"),
                location.span,
            ),
            LexerErrorKind::MalformedFuncAttribute { location, found } => (
                "Malformed function attribute".to_string(),
                format!(" {found} is not a valid attribute"),
                location.span,
            ),
            LexerErrorKind::MalformedTestAttribute { location } => (
                "Malformed test attribute".to_string(),
                "The test attribute can be written in one of these forms: `#[test]`, `#[test(should_fail)]` or `#[test(should_fail_with = \"message\")]`".to_string(),
                location.span,
            ),
            LexerErrorKind::InvalidInnerAttribute { location, found } => (
                "Invalid inner attribute".to_string(),
                format!(" {found} is not a valid inner attribute"),
                location.span,
            ),
            LexerErrorKind::LogicalAnd { location } => (
                "Noir has no logical-and (&&) operator since short-circuiting is much less efficient when compiling to circuits".to_string(),
                "Try `&` instead, or use `if` only if you require short-circuiting".to_string(),
                location.span,
            ),
            LexerErrorKind::UnterminatedBlockComment { location } => ("Unterminated block comment".to_string(), "Unterminated block comment".to_string(), location.span),
            LexerErrorKind::UnterminatedStringLiteral { location } =>
                ("Unterminated string literal".to_string(), "Unterminated string literal".to_string(), location.span),
            LexerErrorKind::InvalidFormatString { found, location } => {
                if found == &'}' {
                    (
                        "Invalid format string: unmatched '}}' found".to_string(),
                        "If you intended to print '}', you can escape it using '}}'".to_string(),
                        location.span,
                    )
                } else {
                    (
                        format!("Invalid format string: expected '}}', found {found:?}"),
                        if found == &'.' {
                            "Field access isn't supported in format strings".to_string()
                        } else {
                            "If you intended to print '{', you can escape it using '{{'".to_string()
                        },
                        location.span,
                    )
                }
            }
            LexerErrorKind::EmptyFormatStringInterpolation { location } => {
                (
                    "Invalid format string: expected letter or underscore, found '}}'".to_string(),
                    "If you intended to print '{' or '}', you can escape them using '{{' and '}}' respectively".to_string(),
                    location.span,
                )
            }
            LexerErrorKind::InvalidEscape { escaped, location } =>
                (format!("'\\{escaped}' is not a valid escape sequence. Use '\\' for a literal backslash character."), "Invalid escape sequence".to_string(), location.span),
            LexerErrorKind::InvalidQuoteDelimiter { delimiter } => {
                (format!("Invalid quote delimiter `{delimiter}`"), "Valid delimiters are `{`, `[`, and `(`".to_string(), delimiter.to_span())
            },
            LexerErrorKind::NonAsciiComment { location } => {
                ("Non-ASCII character in comment".to_string(), "Invalid comment character: only ASCII is currently supported.".to_string(), location.span)
            }
            LexerErrorKind::UnclosedQuote { start_delim, end_delim } => {
                ("Unclosed `quote` expression".to_string(), format!("Expected a `{end_delim}` to close this `{start_delim}`"), start_delim.to_span())
            }
        }
    }
}

impl<'a> From<&'a LexerErrorKind> for Diagnostic {
    fn from(error: &'a LexerErrorKind) -> Diagnostic {
        let (primary, secondary, span) = error.parts();
        Diagnostic::simple_error(primary, secondary, span)
    }
}
