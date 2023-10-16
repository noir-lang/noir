use crate::parser::ParserError;
use crate::parser::ParserErrorReason;
use crate::token::SpannedToken;

use super::token::Token;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::Span;
use thiserror::Error;

#[derive(Error, Clone, Debug, PartialEq, Eq)]
pub enum LexerErrorKind {
    #[error("An unexpected character {:?} was found.", found)]
    UnexpectedCharacter { span: Span, expected: String, found: Option<char> },
    #[error("NotADoubleChar : {:?} is not a double char token", found)]
    NotADoubleChar { span: Span, found: Token },
    #[error("InvalidIntegerLiteral : {:?} is not a integer", found)]
    InvalidIntegerLiteral { span: Span, found: String },
    #[error("MalformedFuncAttribute : {:?} is not a valid attribute", found)]
    MalformedFuncAttribute { span: Span, found: String },
    #[error("TooManyBits")]
    TooManyBits { span: Span, max: u32, got: u32 },
    #[error("LogicalAnd used instead of bitwise and")]
    LogicalAnd { span: Span },
    #[error("Unterminated block comment")]
    UnterminatedBlockComment { span: Span },
    #[error("Unterminated string literal")]
    UnterminatedStringLiteral { span: Span },
    #[error(
        "'\\{escaped}' is not a valid escape sequence. Use '\\' for a literal backslash character."
    )]
    InvalidEscape { escaped: char, span: Span },
}

impl From<LexerErrorKind> for ParserError {
    fn from(value: LexerErrorKind) -> Self {
        let span = value.span();
        ParserError::with_reason(ParserErrorReason::Lexer(value), span)
    }
}

impl LexerErrorKind {
    pub fn span(&self) -> Span {
        match self {
            LexerErrorKind::UnexpectedCharacter { span, .. } => *span,
            LexerErrorKind::NotADoubleChar { span, .. } => *span,
            LexerErrorKind::InvalidIntegerLiteral { span, .. } => *span,
            LexerErrorKind::MalformedFuncAttribute { span, .. } => *span,
            LexerErrorKind::TooManyBits { span, .. } => *span,
            LexerErrorKind::LogicalAnd { span } => *span,
            LexerErrorKind::UnterminatedBlockComment { span } => *span,
            LexerErrorKind::UnterminatedStringLiteral { span } => *span,
            LexerErrorKind::InvalidEscape { span, .. } => *span,
        }
    }

    fn parts(&self) -> (String, String, Span) {
        match self {
            LexerErrorKind::UnexpectedCharacter {
                span,
                expected,
                found,
            } => {
                let found: String = found.map(Into::into).unwrap_or_else(|| "<eof>".into());

                (
                    "An unexpected character was found".to_string(),
                    format!("Expected {expected}, but found {found}"),
                    *span,
                )
            },
            LexerErrorKind::NotADoubleChar { span, found } => (
                format!("Tried to parse {found} as double char"),
                format!(
                    " {found:?} is not a double char, this is an internal error"
                ),
                *span,
            ),
            LexerErrorKind::InvalidIntegerLiteral { span, found } => (
                "Invalid integer literal".to_string(),
                format!(" {found} is not an integer"),
                *span,
            ),
            LexerErrorKind::MalformedFuncAttribute { span, found } => (
                "Malformed function attribute".to_string(),
                format!(" {found} is not a valid attribute"),
                *span,
            ),
            LexerErrorKind::TooManyBits { span, max, got } => (
                "Integer literal too large".to_string(),
                format!(
                    "The maximum number of bits needed to represent a field is {max}, This integer type needs {got} bits"
                ),
                *span,
            ),
            LexerErrorKind::LogicalAnd { span } => (
                "Noir has no logical-and (&&) operator since short-circuiting is much less efficient when compiling to circuits".to_string(),
                "Try `&` instead, or use `if` only if you require short-circuiting".to_string(),
                *span,
            ),
            LexerErrorKind::UnterminatedBlockComment { span } => ("Unterminated block comment".to_string(), "Unterminated block comment".to_string(), *span),
            LexerErrorKind::UnterminatedStringLiteral { span } =>
                ("Unterminated string literal".to_string(), "Unterminated string literal".to_string(), *span),
            LexerErrorKind::InvalidEscape { escaped, span } =>
                (format!("'\\{escaped}' is not a valid escape sequence. Use '\\' for a literal backslash character."), "Invalid escape sequence".to_string(), *span),
        }
    }
}

impl From<LexerErrorKind> for Diagnostic {
    fn from(error: LexerErrorKind) -> Diagnostic {
        let (primary, secondary, span) = error.parts();
        Diagnostic::simple_error(primary, secondary, span)
    }
}

impl From<LexerErrorKind> for chumsky::error::Simple<SpannedToken, Span> {
    fn from(error: LexerErrorKind) -> Self {
        let (_, message, span) = error.parts();
        chumsky::error::Simple::custom(span, message)
    }
}
