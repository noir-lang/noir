use chumsky::prelude::*;

use crate::ast::{ExpressionKind, GenericTypeArgs, Ident, PathSegment, UnaryOp};
use crate::macros_api::UnresolvedType;
use crate::parser::ParserErrorReason;
use crate::{
    parser::{labels::ParsingRuleLabel, ExprParser, NoirParser, ParserError},
    token::{Keyword, Token, TokenKind},
};

use super::path::{path, path_no_turbofish};
use super::types::required_generic_type_args;

/// This parser always parses no input and fails
pub(super) fn nothing<T>() -> impl NoirParser<T> {
    one_of([]).map(|_| unreachable!("parser should always error"))
}

pub(super) fn keyword(keyword: Keyword) -> impl NoirParser<Token> {
    just(Token::Keyword(keyword))
}

pub(super) fn token_kind(token_kind: TokenKind) -> impl NoirParser<Token> {
    filter_map(move |span, found: Token| {
        if found.kind() == token_kind {
            Ok(found)
        } else {
            Err(ParserError::expected_label(
                ParsingRuleLabel::TokenKind(token_kind.clone()),
                found,
                span,
            ))
        }
    })
}

pub(super) fn path_segment<'a>(
    type_parser: impl NoirParser<UnresolvedType> + 'a,
) -> impl NoirParser<PathSegment> + 'a {
    ident().then(turbofish(type_parser)).validate(|(ident, generics), span, emit| {
        if generics.as_ref().map_or(false, |generics| !generics.named_args.is_empty()) {
            let reason = ParserErrorReason::AssociatedTypesNotAllowedInPaths;
            emit(ParserError::with_reason(reason, span));
        }

        let generics = generics.map(|generics| generics.ordered_args);
        PathSegment { ident, generics, span }
    })
}

pub(super) fn path_segment_no_turbofish() -> impl NoirParser<PathSegment> {
    ident().map(PathSegment::from)
}

pub(super) fn ident() -> impl NoirParser<Ident> {
    token_kind(TokenKind::Ident).map_with_span(Ident::from_token)
}

// Right-shift (>>) is issued as two separate > tokens by the lexer as this makes it easier
// to parse nested generic types. For normal expressions however, it means we have to manually
// parse two greater-than tokens as a single right-shift here.
pub(super) fn right_shift_operator() -> impl NoirParser<Token> {
    just(Token::Greater).then(just(Token::Greater)).to(Token::ShiftRight)
}

pub(super) fn not<P>(term_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    just(Token::Bang).ignore_then(term_parser).map(|rhs| ExpressionKind::prefix(UnaryOp::Not, rhs))
}

pub(super) fn negation<P>(term_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    just(Token::Minus)
        .ignore_then(term_parser)
        .map(|rhs| ExpressionKind::prefix(UnaryOp::Minus, rhs))
}

pub(super) fn mutable_reference<P>(term_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    just(Token::Ampersand)
        .ignore_then(keyword(Keyword::Mut))
        .ignore_then(term_parser)
        .map(|rhs| ExpressionKind::prefix(UnaryOp::MutableReference, rhs))
}

pub(super) fn dereference<P>(term_parser: P) -> impl NoirParser<ExpressionKind>
where
    P: ExprParser,
{
    just(Token::Star)
        .ignore_then(term_parser)
        .map(|rhs| ExpressionKind::prefix(UnaryOp::Dereference { implicitly_added: false }, rhs))
}

pub(super) fn turbofish<'a>(
    type_parser: impl NoirParser<UnresolvedType> + 'a,
) -> impl NoirParser<Option<GenericTypeArgs>> + 'a {
    just(Token::DoubleColon).ignore_then(required_generic_type_args(type_parser)).or_not()
}

pub(super) fn variable() -> impl NoirParser<ExpressionKind> {
    path(super::parse_type()).map(ExpressionKind::Variable)
}

pub(super) fn variable_no_turbofish() -> impl NoirParser<ExpressionKind> {
    path_no_turbofish().map(ExpressionKind::Variable)
}

pub(super) fn macro_quote_marker() -> impl NoirParser<ExpressionKind> {
    token_kind(TokenKind::UnquoteMarker).map(|token| match token {
        Token::UnquoteMarker(expr_id) => ExpressionKind::Resolved(expr_id),
        other => unreachable!("Non-unquote-marker parsed as an unquote marker: {other:?}"),
    })
}

#[cfg(test)]
mod test {
    use crate::parser::parser::{
        expression, expression_no_constructors, fresh_statement, term, test_helpers::*,
    };

    #[test]
    fn parse_unary() {
        parse_all(
            term(expression(), expression_no_constructors(expression()), fresh_statement(), true),
            vec!["!hello", "-hello", "--hello", "-!hello", "!-hello"],
        );
        parse_all_failing(
            term(expression(), expression_no_constructors(expression()), fresh_statement(), true),
            vec!["+hello", "/hello"],
        );
    }
}
