use crate::lexer::token::{SpannedToken, Token, TokenKind};
use crate::{lexer::errors::LexerErrorKind, PathKind};

use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::DiagnosableError;
use noirc_errors::Span;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserErrorKind {
    #[error("Lexer error found")]
    LexerError(LexerErrorKind),
    #[error(" expected expression, found `{}`", lexeme)]
    ExpectedExpression { span: Span, lexeme: String },
    #[error("Unexpected token found")]
    UnexpectedToken {
        span: Span,
        expected: Token,
        found: Token,
    },
    #[error("Unexpected token kind found")]
    UnexpectedTokenKind {
        span: Span,
        expected: TokenKind,
        found: TokenKind,
    },
    #[error("Paths with a single segment, cannot have the single segment be a keyword")]
    SingleKeywordSegmentNotAllowed { span: Span, path_kind: PathKind },
    #[error("Unstructured Error")]
    UnstructuredError { span: Span, message: String },
    #[error("Token is not a unary operation")]
    TokenNotUnaryOp { spanned_token: SpannedToken },
    #[error("Token is not a binary operation")]
    TokenNotBinaryOp { spanned_token: SpannedToken },
}

impl DiagnosableError for ParserErrorKind {
    fn to_diagnostic(&self) -> Diagnostic {
        match self {
            ParserErrorKind::LexerError(lex_err) => lex_err.to_diagnostic(),
            ParserErrorKind::ExpectedExpression { span, lexeme } => {
                let mut diag = Diagnostic::simple_error(
                    format!("Unexpected start of an expression {}", lexeme),
                    format!("did not expect this token"),
                    *span,
                );
                diag.add_note(format!("This error is commonly caused by either a previous error cascading or an unclosed delimiter."));
                diag
            }
            ParserErrorKind::TokenNotUnaryOp { spanned_token } => Diagnostic::simple_error(
                format!("Unsupported unary operation {}", spanned_token.token()),
                format!("cannot use as a unary operation."),
                spanned_token.into_span(),
            ),
            ParserErrorKind::TokenNotBinaryOp { spanned_token } => Diagnostic::simple_error(
                format!("Unsupported binary operation {}", spanned_token.token()),
                format!("cannot use as a binary operation."),
                spanned_token.into_span(),
            ),
            ParserErrorKind::UnexpectedToken {
                span,
                expected,
                found,
            } => Diagnostic::simple_error(
                format!("Expected a {} but found {}", expected, found),
                format!("Expected {}", expected),
                *span,
            ),
            ParserErrorKind::UnexpectedTokenKind {
                span,
                expected,
                found,
            } => Diagnostic::simple_error(
                format!("Expected a {} but found {}", expected, found),
                format!("Expected {}", expected),
                *span,
            ),
            ParserErrorKind::UnstructuredError { span, message } => {
                Diagnostic::simple_error("".to_owned(), message.to_string(), *span)
            }
            ParserErrorKind::SingleKeywordSegmentNotAllowed { span, path_kind } => {
                let note = match path_kind{
                    PathKind::Dep => "You have specified `dep`. However, it is not possible to determine which dependency you want to import.\n Try `use dep::{name of dependency}`",
                    PathKind::Crate => "You have specified `crate`. However, it is not possible to determine which module you want to import.\n Try `use crate::{name of module}`",
                    _=> unreachable!("ice: this error is caused by single segment paths which contain a keyword")
                };

                let mut diag = Diagnostic::simple_error(
                    format!("path is ambiguous"),
                    format!("path contains a single keyword"),
                    *span,
                );
                diag.add_note(note.to_owned());
                diag
            }
        }
    }
}
