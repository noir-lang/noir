use super::token::Token;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::{DiagnosableError, Span};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexerErrorKind {
    #[error("An unexpected character {:?} was found.", found)]
    UnexpectedCharacter {
        span: Span,
        expected: String,
        found: char,
    },
    #[error("The character {:?} is not in the language.", found)]
    CharacterNotInLanguage { span: Span, found: char },
    #[error("NotADoubleChar : {:?} is not a double char token", found)]
    NotADoubleChar { span: Span, found: Token },
    #[error("InvalidIntegerLiteral : {:?} is not a integer", found)]
    InvalidIntegerLiteral { span: Span, found: String },
    #[error("MalformedFuncAttribute : {:?} is not a valid attribute", found)]
    MalformedFuncAttribute { span: Span, found: String },
    #[error("TooManyBits")]
    TooManyBits { span: Span, max: u32, got: u32 },
}

impl DiagnosableError for LexerErrorKind {
    fn to_diagnostic(&self) -> Diagnostic {
        match self {
            LexerErrorKind::UnexpectedCharacter {
                span,
                expected,
                found,
            } => Diagnostic::simple_error(
                "an unexpected character was found".to_string(),
                format!(" expected {} , but got {}", expected, found),
                *span,
            ),
            LexerErrorKind::CharacterNotInLanguage { span, found } => Diagnostic::simple_error(
                "char is not in language".to_string(),
                format!(" {:?} is not in language", found),
                *span,
            ),
            LexerErrorKind::NotADoubleChar { span, found } => Diagnostic::simple_error(
                format!("tried to parse {} as double char", found),
                format!(
                    " {:?} is not a double char, this is an internal error",
                    found
                ),
                *span,
            ),
            LexerErrorKind::InvalidIntegerLiteral { span, found } => Diagnostic::simple_error(
                "invalid integer literal".to_string(),
                format!(" {} is not an integer", found),
                *span,
            ),
            LexerErrorKind::MalformedFuncAttribute { span, found } => Diagnostic::simple_error(
                "malformed function attribute".to_string(),
                format!(" {} is not a valid attribute", found),
                *span,
            ),
            LexerErrorKind::TooManyBits { span, max, got } => Diagnostic::simple_error(
                "integer literal too large".to_string(),
                format!(
                    "The maximum number of bits needed to represent a field is {}, This integer type needs {} bits",
                    max, got
                ),
                *span,
            ),
        }
    }
}
