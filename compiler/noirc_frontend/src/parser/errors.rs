use crate::lexer::errors::LexerErrorKind;
use crate::lexer::token::Token;
use crate::Expression;
use small_ord_set::SmallOrdSet;
use thiserror::Error;

use iter_extended::vecmap;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::Span;

use super::labels::ParsingRuleLabel;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParserErrorReason {
    #[error("Unexpected '{0}', expected a field name")]
    ExpectedFieldName(Token),
    #[error("expected a pattern but found a type - {0}")]
    ExpectedPatternButFoundType(Token),
    #[error("Expected a ; separating these two statements")]
    MissingSeparatingSemi,
    #[error("constrain keyword is deprecated")]
    ConstrainDeprecated,
    #[error("Expression is invalid in an array-length type: '{0}'. Only unsigned integer constants, globals, generics, +, -, *, /, and % may be used in this context.")]
    InvalidArrayLengthExpression(Expression),
    #[error("Early 'return' is unsupported")]
    EarlyReturn,
    #[error("Patterns aren't allowed in a trait's function declarations")]
    PatternInTraitFunctionParameter,
    #[error("comptime keyword is deprecated")]
    ComptimeDeprecated,
    #[error("{0} are experimental and aren't fully supported yet")]
    ExperimentalFeature(&'static str),
    #[error("Where clauses are allowed only on functions with generic parameters")]
    WhereClauseOnNonGenericFunction,
    #[error(
        "Multiple primary attributes found. Only one function attribute is allowed per function"
    )]
    MultipleFunctionAttributesFound,
    #[error("A function attribute cannot be placed on a struct")]
    NoFunctionAttributesAllowedOnStruct,
    #[error("Assert statements can only accept string literals")]
    AssertMessageNotString,
    #[error("{0}")]
    Lexer(LexerErrorKind),
}

/// Represents a parsing error, or a parsing error in the making.
///
/// `ParserError` is used extensively by the parser, as it not only used to report badly formed
/// token streams, but also as a general intermediate that accumulates information as various
/// parsing rules are tried. This struct is constructed and destructed with a very high frequency
/// and as such, the time taken to do so significantly impacts parsing performance. For this
/// reason we use `SmallOrdSet` to avoid heap allocations for as long as possible - this greatly
/// inflates the size of the error, but this is justified by a resulting increase in parsing
/// speeds of approximately 40% in release mode.
///
/// Both `expected_tokens` and `expected_labels` use `SmallOrdSet` sized 1. In the of labels this
/// is optimal. In the of tokens we stop here due to fast diminishing returns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserError {
    expected_tokens: SmallOrdSet<[Token; 1]>,
    expected_labels: SmallOrdSet<[ParsingRuleLabel; 1]>,
    found: Token,
    reason: Option<ParserErrorReason>,
    span: Span,
}

impl ParserError {
    pub fn empty(found: Token, span: Span) -> ParserError {
        ParserError {
            expected_tokens: SmallOrdSet::new(),
            expected_labels: SmallOrdSet::new(),
            found,
            reason: None,
            span,
        }
    }

    pub fn expected_label(label: ParsingRuleLabel, found: Token, span: Span) -> ParserError {
        let mut error = ParserError::empty(found, span);
        error.expected_labels.insert(label);
        error
    }

    pub fn with_reason(reason: ParserErrorReason, span: Span) -> ParserError {
        let mut error = ParserError::empty(Token::EOF, span);
        error.reason = Some(reason);
        error
    }

    pub fn found(&self) -> &Token {
        &self.found
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn reason(&self) -> Option<&ParserErrorReason> {
        self.reason.as_ref()
    }
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut expected = vecmap(&self.expected_tokens, ToString::to_string);
        expected.append(&mut vecmap(&self.expected_labels, |label| format!("{label}")));

        if expected.is_empty() {
            write!(f, "Unexpected {} in input", self.found)
        } else if expected.len() == 1 {
            let first = expected.first().unwrap();
            let vowel = "aeiou".contains(first.chars().next().unwrap());
            write!(
                f,
                "Expected a{} {} but found {}",
                if vowel { "n" } else { "" },
                first,
                self.found
            )
        } else {
            let expected = expected.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");

            write!(f, "Unexpected {}, expected one of {}", self.found, expected)
        }
    }
}

impl From<ParserError> for Diagnostic {
    fn from(error: ParserError) -> Diagnostic {
        match &error.reason {
            Some(reason) => {
                match reason {
                    ParserErrorReason::ConstrainDeprecated => Diagnostic::simple_error(
                        "Use of deprecated keyword 'constrain'".into(),
                        "The 'constrain' keyword is deprecated. Please use the 'assert' function instead.".into(),
                        error.span,
                    ),
                    ParserErrorReason::ComptimeDeprecated => Diagnostic::simple_warning(
                        "Use of deprecated keyword 'comptime'".into(),
                        "The 'comptime' keyword has been deprecated. It can be removed without affecting your program".into(),
                        error.span,
                    ),
                    ParserErrorReason::ExperimentalFeature(_) => Diagnostic::simple_warning(
                        reason.to_string(),
                        "".into(),
                        error.span,
                    ),
                    reason @ ParserErrorReason::ExpectedPatternButFoundType(ty) => {
                        Diagnostic::simple_error(reason.to_string(), format!("{ty} is a type and cannot be used as a variable name"), error.span)
                    }
                    other => {

                        Diagnostic::simple_error(format!("{other}"), String::new(), error.span)
                    }
                }
            }
            None => {
                let primary = error.to_string();
                Diagnostic::simple_error(primary, String::new(), error.span)
            }
        }
    }
}

impl chumsky::Error<Token> for ParserError {
    type Span = Span;
    type Label = ParsingRuleLabel;

    fn expected_input_found<Iter>(span: Self::Span, expected: Iter, found: Option<Token>) -> Self
    where
        Iter: IntoIterator<Item = Option<Token>>,
    {
        ParserError {
            expected_tokens: expected.into_iter().map(|opt| opt.unwrap_or(Token::EOF)).collect(),
            expected_labels: SmallOrdSet::new(),
            found: found.unwrap_or(Token::EOF),
            reason: None,
            span,
        }
    }

    fn with_label(mut self, label: Self::Label) -> Self {
        self.expected_tokens.clear();
        self.expected_labels.clear();
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

        if self.reason.is_none() {
            self.reason = other.reason;
        }

        self.span = self.span.merge(other.span);
        self
    }
}
