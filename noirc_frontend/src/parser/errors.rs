use crate::lexer::errors::LexerError;

use thiserror::Error;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::DiagnosableError;
use noirc_errors::Span;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Lexer error found")]
    LexerError(LexerError),
    #[error(" `{:?}` cannot be used as a prefix operator.", lexeme)]
    NoPrefixFunction { span: Span, lexeme: String },
    #[error(" `{:?}` cannot be used as a binary operator.", lexeme)]
    NoInfixFunction { span: Span, lexeme: String },
    #[error("Internal Compiler Error, unrecoverable")] // Actually lets separate these two types of errors
    InternalError,
}

impl DiagnosableError for ParserError {
    fn to_diagnostic(&self) -> Diagnostic{
        match self {
            ParserError::LexerError(lex_err) => lex_err.to_diagnostic(),
            ParserError::InternalError => panic!("Internal Error. This is a bug in the compiler"),
            _ => todo!(),
        }
    }
}