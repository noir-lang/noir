use thiserror::Error;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::{Span, DiagnosableError};
use super::token::Token;

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("An unexpected character {:?} was found.", found)]
    UnexpectedCharacter { span: Span, found: char },
    #[error("The character {:?} is not in the language.", found)]
    CharacterNotInLanguage { span: Span, found: char },
    #[error("Internal Error : {:?} is not a double char token", found)]
    NotADoubleChar { span: Span, found: Token },
    #[error("Internal Compiler Error, unrecoverable")] // Actually lets separate these two types of errors
    InternalError,
}

impl DiagnosableError for LexerError {
    fn to_diagnostic(&self) -> Diagnostic{
        match self {
            LexerError::UnexpectedCharacter{span, found} => {
                Diagnostic::simple_error(format!("an unexpected character was found"), format!(" {:?} is unexpected", found), *span)
            },
            LexerError::InternalError => panic!("Internal Error. This is a bug in the compiler"),
            _=> todo!()
        }
    }
}