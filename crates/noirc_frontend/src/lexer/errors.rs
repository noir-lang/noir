use crate::token::SpannedToken;

use super::token::Token;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::{DiagnosableError, Span};
use thiserror::Error;

#[derive(Error, Clone, Debug, PartialEq, Eq)]
pub enum LexerErrorKind {
    #[error("An unexpected character {:?} was found.", found)]
    UnexpectedCharacter { span: Span, expected: String, found: char },
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
        }
    }

    fn parts(&self) -> (String, String, Span) {
        match self {
            LexerErrorKind::UnexpectedCharacter {
                span,
                expected,
                found,
            } => (
                "an unexpected character was found".to_string(),
                format!(" expected {} , but got {}", expected, found),
                *span,
            ),
            LexerErrorKind::NotADoubleChar { span, found } => (
                format!("tried to parse {} as double char", found),
                format!(
                    " {:?} is not a double char, this is an internal error",
                    found
                ),
                *span,
            ),
            LexerErrorKind::InvalidIntegerLiteral { span, found } => (
                "invalid integer literal".to_string(),
                format!(" {} is not an integer", found),
                *span,
            ),
            LexerErrorKind::MalformedFuncAttribute { span, found } => (
                "malformed function attribute".to_string(),
                format!(" {} is not a valid attribute", found),
                *span,
            ),
            LexerErrorKind::TooManyBits { span, max, got } => (
                "integer literal too large".to_string(),
                format!(
                    "The maximum number of bits needed to represent a field is {}, This integer type needs {} bits",
                    max, got
                ),
                *span,
            ),
            LexerErrorKind::LogicalAnd { span } => (
                "Noir has no logical-and (&&) operator since short-circuiting is much less efficient when compiling to circuits".to_string(),
                "Try `&` instead, or use `if` only if you require short-circuiting".to_string(),
                *span,
            ),
        }
    }
}

impl DiagnosableError for LexerErrorKind {
    fn to_diagnostic(&self) -> Diagnostic {
        let (primary, secondary, span) = self.parts();
        Diagnostic::simple_error(primary, secondary, span)
    }
}

impl From<LexerErrorKind> for chumsky::error::Simple<SpannedToken, Span> {
    fn from(error: LexerErrorKind) -> Self {
        let (_, message, span) = error.parts();
        chumsky::error::Simple::custom(span, message)
    }
}
