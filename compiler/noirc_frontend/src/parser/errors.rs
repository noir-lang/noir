use crate::ast::{Expression, IntegerBitSize, ItemVisibility};
use crate::lexer::errors::LexerErrorKind;
use crate::lexer::token::Token;
use crate::token::TokenKind;
use small_ord_set::SmallOrdSet;
use thiserror::Error;

use iter_extended::vecmap;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::Span;

use super::labels::ParsingRuleLabel;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParserErrorReason {
    #[error("Unexpected `;`")]
    UnexpectedSemicolon,
    #[error("Unexpected `,`")]
    UnexpectedComma,
    #[error("Expected a `{token}` separating these two {items}")]
    ExpectedTokenSeparatingTwoItems { token: Token, items: &'static str },
    #[error("Expected `mut` after `&`, found `{found}`")]
    ExpectedMutAfterAmpersand { found: Token },
    #[error("Invalid left-hand side of assignment")]
    InvalidLeftHandSideOfAssignment,
    #[error("Expected trait, found {found}")]
    ExpectedTrait { found: String },
    #[error("Visibility `{visibility}` is not followed by an item")]
    VisibilityNotFollowedByAnItem { visibility: ItemVisibility },
    #[error("`unconstrained` is not followed by an item")]
    UnconstrainedNotFollowedByAnItem,
    #[error("`comptime` is not followed by an item")]
    ComptimeNotFollowedByAnItem,
    #[error("`mut` cannot be applied to this item")]
    MutableNotApplicable,
    #[error("`comptime` cannot be applied to this item")]
    ComptimeNotApplicable,
    #[error("`unconstrained` cannot be applied to this item")]
    UnconstrainedNotApplicable,
    #[error("Expected an identifier or `(expression) after `$` for unquoting")]
    ExpectedIdentifierOrLeftParenAfterDollar,
    #[error("`&mut` can only be used with `self")]
    RefMutCanOnlyBeUsedWithSelf,
    #[error("Invalid pattern")]
    InvalidPattern,
    #[error("Documentation comment does not document anything")]
    DocCommentDoesNotDocumentAnything,

    #[error("Missing type for function parameter")]
    MissingTypeForFunctionParameter,
    #[error("Missing type for numeric generic")]
    MissingTypeForNumericGeneric,
    #[error("Expected a function body (`{{ ... }}`), not `;`")]
    ExpectedFunctionBody,
    #[error("Expected the global to have a value")]
    GlobalWithoutValue,

    #[error("Unexpected '{0}', expected a field name or number")]
    ExpectedFieldName(Token),
    #[error("Expected a pattern but found a type - {0}")]
    ExpectedPatternButFoundType(Token),
    #[error("Expected a ; separating these two statements")]
    MissingSeparatingSemi,
    #[error("constrain keyword is deprecated")]
    ConstrainDeprecated,
    #[error("Invalid type expression: '{0}'. Only unsigned integer constants up to `u32`, globals, generics, +, -, *, /, and % may be used in this context.")]
    InvalidTypeExpression(Expression),
    #[error("Early 'return' is unsupported")]
    EarlyReturn,
    #[error("Patterns aren't allowed in a trait's function declarations")]
    PatternInTraitFunctionParameter,
    #[error("Patterns aren't allowed in a trait impl's associated constants")]
    PatternInAssociatedConstant,
    #[error("Visibility is ignored on a trait method")]
    TraitVisibilityIgnored,
    #[error("Visibility is ignored on a trait impl method")]
    TraitImplVisibilityIgnored,
    #[error("comptime keyword is deprecated")]
    ComptimeDeprecated,
    #[error("{0} are experimental and aren't fully supported yet")]
    ExperimentalFeature(&'static str),
    #[error(
        "Multiple primary attributes found. Only one function attribute is allowed per function"
    )]
    MultipleFunctionAttributesFound,
    #[error("A function attribute cannot be placed on a struct")]
    NoFunctionAttributesAllowedOnStruct,
    #[error("Assert statements can only accept string literals")]
    AssertMessageNotString,
    #[error("Integer bit size {0} isn't supported")]
    InvalidBitSize(u32),
    #[error("{0}")]
    Lexer(LexerErrorKind),
    #[error("The only supported numeric generic types are `u1`, `u8`, `u16`, and `u32`")]
    ForbiddenNumericGenericType,
    #[error("Invalid call data identifier, must be a number. E.g `call_data(0)`")]
    InvalidCallDataIdentifier,
    #[error("Associated types are not allowed in paths")]
    AssociatedTypesNotAllowedInPaths,
    #[error("Associated types are not allowed on a method call")]
    AssociatedTypesNotAllowedInMethodCalls,
    #[error("Empty trait alias")]
    EmptyTraitAlias,
    #[error(
        "Wrong number of arguments for attribute `{}`. Expected {}, found {}",
        name,
        if min == max { min.to_string() } else { format!("between {} and {}", min, max) },
        found
    )]
    WrongNumberOfAttributeArguments { name: String, min: usize, max: usize, found: usize },
    #[error("The `deprecated` attribute expects a string argument")]
    DeprecatedAttributeExpectsAStringArgument,
    #[error("Unsafe block must start with a safety comment")]
    MissingSafetyComment,
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

    pub fn expected_token(token: Token, found: Token, span: Span) -> ParserError {
        let mut error = ParserError::empty(found, span);
        error.expected_tokens.insert(token);
        error
    }

    pub fn expected_one_of_tokens(tokens: &[Token], found: Token, span: Span) -> ParserError {
        let mut error = ParserError::empty(found, span);
        for token in tokens {
            error.expected_tokens.insert(token.clone());
        }
        error
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

    pub fn is_warning(&self) -> bool {
        let diagnostic: Diagnostic = self.into();
        diagnostic.is_warning()
    }
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token_to_string = |token: &Token| match token {
            Token::EOF => token.to_string(),
            _ => format!("'{}'", token),
        };

        let reason_str: String = if self.reason.is_none() {
            "".to_string()
        } else {
            format!("\nreason: {}", Diagnostic::from(self))
        };
        let mut expected = vecmap(&self.expected_tokens, token_to_string);
        expected.append(&mut vecmap(&self.expected_labels, |label| format!("{label}")));

        if expected.is_empty() {
            write!(f, "Unexpected {} in input{}", self.found, reason_str)
        } else if expected.len() == 1 {
            let first = expected.first().unwrap();
            let vowel = "aeiou".contains(first.chars().next().unwrap());
            write!(
                f,
                "Expected a{} {} but found {}{}",
                if vowel { "n" } else { "" },
                first,
                token_to_string(&self.found),
                reason_str
            )
        } else {
            let expected = expected.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");

            write!(f, "Unexpected {:?}, expected one of {}{}", self.found, expected, reason_str)
        }
    }
}

impl<'a> From<&'a ParserError> for Diagnostic {
    fn from(error: &'a ParserError) -> Diagnostic {
        match &error.reason {
            Some(reason) => match reason {
                ParserErrorReason::ConstrainDeprecated => {
                    let mut diagnostic = Diagnostic::simple_error(
                        "Use of deprecated keyword 'constrain'".into(),
                        "The 'constrain' keyword is deprecated. Please use the 'assert' function instead.".into(),
                        error.span,
                    );
                    diagnostic.deprecated = true;
                    diagnostic
                }
                ParserErrorReason::ComptimeDeprecated => {
                    let mut diagnostic = Diagnostic::simple_warning(
                        "Use of deprecated keyword 'comptime'".into(),
                        "The 'comptime' keyword has been deprecated. It can be removed without affecting your program".into(),
                        error.span,
                    ) ;
                    diagnostic.deprecated = true;
                    diagnostic
                }
                ParserErrorReason::InvalidBitSize(bit_size) => Diagnostic::simple_error(
                    format!("Use of invalid bit size {}", bit_size),
                    format!(
                        "Allowed bit sizes for integers are {}",
                        IntegerBitSize::allowed_sizes()
                            .iter()
                            .map(|n| n.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    error.span,
                ),
                ParserErrorReason::ExperimentalFeature(_) => {
                    Diagnostic::simple_warning(reason.to_string(), "".into(), error.span)
                }
                ParserErrorReason::TraitVisibilityIgnored => {
                    Diagnostic::simple_warning(reason.to_string(), "".into(), error.span)
                }
                ParserErrorReason::TraitImplVisibilityIgnored => {
                    Diagnostic::simple_warning(reason.to_string(), "".into(), error.span)
                }
                ParserErrorReason::ExpectedPatternButFoundType(ty) => Diagnostic::simple_error(
                    format!("Expected a pattern but found a type - {ty}"),
                    format!("{ty} is a type and cannot be used as a variable name"),
                    error.span,
                ),
                ParserErrorReason::Lexer(error) => error.into(),
                ParserErrorReason::ExpectedMutAfterAmpersand { found } => Diagnostic::simple_error(
                    format!("Expected `mut` after `&`, found `{found}`"),
                    "Noir doesn't have immutable references, only mutable references".to_string(),
                    error.span,
                ),
                ParserErrorReason::MissingSafetyComment => Diagnostic::simple_warning(
                    "Missing Safety Comment".into(),
                    "Unsafe block must start with a safety comment: //@safety".into(),
                    error.span,
                ),
                other => Diagnostic::simple_error(format!("{other}"), String::new(), error.span),
            },
            None => {
                if matches!(
                    error.found.kind(),
                    TokenKind::InnerDocComment | TokenKind::OuterDocComment
                ) {
                    let primary = "This doc comment doesn't document anything".to_string();
                    let secondary = "Consider changing it to a regular `//` comment".to_string();
                    Diagnostic::simple_error(primary, secondary, error.span)
                } else {
                    let primary = error.to_string();
                    Diagnostic::simple_error(primary, String::new(), error.span)
                }
            }
        }
    }
}
