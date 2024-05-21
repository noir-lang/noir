use chumsky::prelude::*;

use crate::ast::{ExpressionKind, Ident, UnaryOp};
use crate::{
    parser::{labels::ParsingRuleLabel, ExprParser, NoirParser, ParserError},
    token::{Keyword, Token, TokenKind},
};

use super::path;

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

pub(super) fn variable() -> impl NoirParser<ExpressionKind> {
    path().map(ExpressionKind::Variable)
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
