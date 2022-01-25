use std::collections::BTreeSet;

use crate::lexer::errors::LexerErrorKind;
use crate::lexer::token::Token;
use crate::BinaryOp;

use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::DiagnosableError;
use noirc_errors::Span;

#[derive(Debug)]
pub struct ParserError {
    expected_tokens: BTreeSet<Token>,
    expected_labels: BTreeSet<String>,
    found: Token,
    lexer_errors: Vec<LexerErrorKind>,
    reason: Option<String>,
    span: Span,
}

impl ParserError {
    pub fn empty(found: Token, span: Span) -> ParserError {
        ParserError {
            expected_tokens: BTreeSet::new(),
            expected_labels: BTreeSet::new(),
            found,
            lexer_errors: vec![],
            reason: None,
            span,
        }
    }

    pub fn expected(token: Token, found: Token, span: Span) -> ParserError {
        let mut error = ParserError::empty(found, span);
        error.expected_tokens.insert(token);
        error
    }

    pub fn expected_label(label: String, found: Token, span: Span) -> ParserError {
        let mut error = ParserError::empty(found, span);
        error.expected_labels.insert(label);
        error
    }

    pub fn with_reason(reason: String, span: Span) -> ParserError {
        let mut error = ParserError::empty(Token::EOF, span);
        error.reason = Some(reason);
        error
    }

    pub fn invalid_constrain_operator(operator: BinaryOp) -> ParserError {
        let message = format!(
            "Cannot use the {} operator in a constraint statement.",
            operator.contents.as_string()
        );
        let mut error = ParserError::empty(operator.contents.as_token(), operator.span());
        error.reason = Some(message);
        error
    }
}

impl DiagnosableError for ParserError {
    fn to_diagnostic(&self) -> Diagnostic {
        match &self.reason {
            Some(reason) => Diagnostic::simple_error(reason.clone(), String::new(), self.span),
            None => {
                let mut expected = self
                    .expected_tokens
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                expected.append(&mut self.expected_labels.iter().cloned().collect());

                let primary = if expected.is_empty() {
                    format!("Unexpected {} in input", self.found)
                } else if expected.len() == 1 {
                    format!(
                        "Expected a {} but found {}",
                        expected.first().unwrap(),
                        self.found
                    )
                } else {
                    let expected = expected
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ");

                    format!("Unexpected {}, expected one of {}", self.found, expected)
                };

                Diagnostic::simple_error(primary, String::new(), self.span)
            }
        }
    }
}

impl From<LexerErrorKind> for ParserError {
    fn from(error: LexerErrorKind) -> Self {
        let span = error.span();
        ParserError {
            expected_tokens: BTreeSet::new(),
            expected_labels: BTreeSet::new(),
            found: Token::EOF,
            lexer_errors: vec![error],
            reason: None,
            span,
        }
    }
}

impl chumsky::Error<Token> for ParserError {
    type Span = Span;
    type Label = String;

    fn expected_input_found<Iter>(span: Self::Span, expected: Iter, found: Option<Token>) -> Self
    where
        Iter: IntoIterator<Item = Option<Token>>,
    {
        ParserError {
            expected_tokens: expected
                .into_iter()
                .map(|opt| opt.unwrap_or(Token::EOF))
                .collect(),
            expected_labels: BTreeSet::new(),
            found: found.unwrap_or(Token::EOF),
            lexer_errors: vec![],
            reason: None,
            span,
        }
    }

    fn with_label(mut self, label: Self::Label) -> Self {
        self.expected_labels.insert(label);
        self
    }

    // Merge two errors into a new one that should encompass both.
    // If one error has a more specific reason with it then keep
    // that reason and discard the other if present.
    // The spans of both errors must match, otherwise the error
    // messages and error spans may not line up.
    fn merge(mut self, mut other: Self) -> Self {
        self.expected_tokens.append(&mut other.expected_tokens);
        self.expected_labels.append(&mut other.expected_labels);
        self.lexer_errors.append(&mut other.lexer_errors);

        if self.reason.is_none() {
            self.reason = other.reason;
        }

        assert_eq!(self.span, other.span);
        self
    }
}
