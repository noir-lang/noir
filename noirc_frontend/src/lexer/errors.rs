use super::token::Token;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::{DiagnosableError, Span};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexerErrorKind {
    #[error("An unexpected character {:?} was found.", found)]
    UnexpectedCharacter { span: Span, found: char },
    #[error("The character {:?} is not in the language.", found)]
    CharacterNotInLanguage { span: Span, found: char },
    #[error("Internal Error : {:?} is not a double char token", found)]
    NotADoubleChar { span: Span, found: Token },
}

impl LexerErrorKind {
    pub fn into_err(self, file_id: usize) -> LexerError {
        LexerError {
            kind: self,
            file_id,
        }
    }
}

#[derive(Debug)]
pub struct LexerError {
    kind: LexerErrorKind,
    file_id: usize,
}

impl DiagnosableError for LexerError {
    fn to_diagnostic(&self) -> Diagnostic {
        match &self.kind {
            LexerErrorKind::UnexpectedCharacter { span, found } => Diagnostic::simple_error(
                format!("an unexpected character was found"),
                format!(" {:?} is unexpected", found),
                *span,
            ),
            LexerErrorKind::CharacterNotInLanguage { span, found } => Diagnostic::simple_error(
                format!("char is not in language"),
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
        }
    }
}
