use crate::lexer::errors::LexerError;
use crate::lexer::token::{Token, TokenKind};

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
    #[error("Unexpected token found")]
    UnexpectedToken { span: Span, expected: Token, found : Token },
    #[error("Unexpected token kind found")]
    UnexpectedTokenKind { span: Span, expected: TokenKind, found : TokenKind },
    #[error("Unstructured Error")]
    UnstructuredError { span: Span, message : String},
    #[error("Internal Compiler Error, unrecoverable")] // Actually lets separate these two types of errors
    InternalError{message : String, span : Span},
}

impl DiagnosableError for ParserError {
    fn to_diagnostic(&self) -> Diagnostic{
        match self {
            ParserError::LexerError(lex_err) => lex_err.to_diagnostic(),
            ParserError::InternalError{message, span} => panic!("Internal Error. This is a bug in the compiler. Please report the following message :\n {} \n with the following span {:?}", message,span),
            ParserError::NoPrefixFunction{span, lexeme} => {
                Diagnostic{
                    message : format!("{} cannot be used as a prefix operator.", lexeme),
                    span : *span
                }
            },
            ParserError::NoInfixFunction{span, lexeme} => {
                Diagnostic{
                    message : format!("{} cannot be used as a infix operator.", lexeme),
                    span : *span
                }
            },
            ParserError::UnexpectedToken{span , expected, found} => {
                Diagnostic{
                    message : format!("Expected a {} but found {}", expected, found),
                    span : *span
                }
            }
            ParserError::UnexpectedTokenKind{span , expected, found} => {
                Diagnostic{
                    message : format!("Expected a {} but found {}", expected, found),
                    span : *span
                }
            },
            ParserError::UnstructuredError{span, message} => {
                Diagnostic{
                    message : message.to_string(),
                    span : *span
                }
            },
        }
    }
}