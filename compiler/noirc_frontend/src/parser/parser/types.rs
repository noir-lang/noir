use super::{
    expression_with_precedence, keyword, nothing, parenthesized, NoirParser, ParserError,
    ParserErrorReason, Precedence,
};
use crate::ast::{UnresolvedType, UnresolvedTypeData};

use crate::parser::labels::ParsingRuleLabel;
use crate::token::{Keyword, Token};
use crate::{Recoverable, UnresolvedTypeExpression};

use chumsky::prelude::*;
use noirc_errors::Span;

fn maybe_comp_time() -> impl NoirParser<()> {
    keyword(Keyword::Comptime).or_not().validate(|opt, span, emit| {
        if opt.is_some() {
            emit(ParserError::with_reason(ParserErrorReason::ComptimeDeprecated, span));
        }
    })
}

pub(super) fn parenthesized_type(
    recursive_type_parser: impl NoirParser<UnresolvedType>,
) -> impl NoirParser<UnresolvedType> {
    recursive_type_parser
        .delimited_by(just(Token::LeftParen), just(Token::RightParen))
        .map_with_span(|typ, span| UnresolvedType {
            typ: UnresolvedTypeData::Parenthesized(Box::new(typ)),
            span: span.into(),
        })
}

pub(super) fn field_type() -> impl NoirParser<UnresolvedType> {
    maybe_comp_time()
        .then_ignore(keyword(Keyword::Field))
        .map_with_span(|_, span| UnresolvedTypeData::FieldElement.with_span(span))
}

pub(super) fn bool_type() -> impl NoirParser<UnresolvedType> {
    maybe_comp_time()
        .then_ignore(keyword(Keyword::Bool))
        .map_with_span(|_, span| UnresolvedTypeData::Bool.with_span(span))
}

pub(super) fn string_type() -> impl NoirParser<UnresolvedType> {
    keyword(Keyword::String)
        .ignore_then(
            type_expression().delimited_by(just(Token::Less), just(Token::Greater)).or_not(),
        )
        .map_with_span(|expr, span| UnresolvedTypeData::String(expr).with_span(span))
}

pub(super) fn format_string_type(
    type_parser: impl NoirParser<UnresolvedType>,
) -> impl NoirParser<UnresolvedType> {
    keyword(Keyword::FormatString)
        .ignore_then(
            type_expression()
                .then_ignore(just(Token::Comma))
                .then(type_parser)
                .delimited_by(just(Token::Less), just(Token::Greater)),
        )
        .map_with_span(|(size, fields), span| {
            UnresolvedTypeData::FormatString(size, Box::new(fields)).with_span(span)
        })
}

pub(super) fn int_type() -> impl NoirParser<UnresolvedType> {
    maybe_comp_time()
        .then(filter_map(|span, token: Token| match token {
            Token::IntType(int_type) => Ok(int_type),
            unexpected => {
                Err(ParserError::expected_label(ParsingRuleLabel::IntegerType, unexpected, span))
            }
        }))
        .validate(|(_, token), span, emit| {
            UnresolvedTypeData::from_int_token(token)
                .map(|data| data.with_span(span))
                .unwrap_or_else(|err| {
                    emit(ParserError::with_reason(ParserErrorReason::InvalidBitSize(err.0), span));
                    UnresolvedType::error(span)
                })
        })
}

pub(super) fn array_type(
    type_parser: impl NoirParser<UnresolvedType>,
) -> impl NoirParser<UnresolvedType> {
    just(Token::LeftBracket)
        .ignore_then(type_parser)
        .then(just(Token::Semicolon).ignore_then(type_expression()))
        .then_ignore(just(Token::RightBracket))
        .map_with_span(|(element_type, size), span| {
            UnresolvedTypeData::Array(size, Box::new(element_type)).with_span(span)
        })
}

pub(super) fn slice_type(
    type_parser: impl NoirParser<UnresolvedType>,
) -> impl NoirParser<UnresolvedType> {
    just(Token::LeftBracket)
        .ignore_then(type_parser)
        .then_ignore(just(Token::RightBracket))
        .map_with_span(|element_type, span| {
            UnresolvedTypeData::Slice(Box::new(element_type)).with_span(span)
        })
}

pub(super) fn type_expression() -> impl NoirParser<UnresolvedTypeExpression> {
    recursive(|expr| {
        expression_with_precedence(
            Precedence::lowest_type_precedence(),
            expr,
            nothing(),
            nothing(),
            true,
            false,
        )
    })
    .labelled(ParsingRuleLabel::TypeExpression)
    .try_map(UnresolvedTypeExpression::from_expr)
}

pub(super) fn tuple_type<T>(type_parser: T) -> impl NoirParser<UnresolvedType>
where
    T: NoirParser<UnresolvedType>,
{
    let fields = type_parser.separated_by(just(Token::Comma)).allow_trailing();
    parenthesized(fields).map_with_span(|fields, span| {
        if fields.is_empty() {
            UnresolvedTypeData::Unit.with_span(span)
        } else {
            UnresolvedTypeData::Tuple(fields).with_span(span)
        }
    })
}

pub(super) fn function_type<T>(type_parser: T) -> impl NoirParser<UnresolvedType>
where
    T: NoirParser<UnresolvedType>,
{
    let args = parenthesized(type_parser.clone().separated_by(just(Token::Comma)).allow_trailing());

    let env = just(Token::LeftBracket)
        .ignore_then(type_parser.clone())
        .then_ignore(just(Token::RightBracket))
        .or_not()
        .map_with_span(|t, span| {
            t.unwrap_or_else(|| UnresolvedTypeData::Unit.with_span(Span::empty(span.end())))
        });

    keyword(Keyword::Fn)
        .ignore_then(env)
        .then(args)
        .then_ignore(just(Token::Arrow))
        .then(type_parser)
        .map_with_span(|((env, args), ret), span| {
            UnresolvedTypeData::Function(args, Box::new(ret), Box::new(env)).with_span(span)
        })
}

pub(super) fn mutable_reference_type<T>(type_parser: T) -> impl NoirParser<UnresolvedType>
where
    T: NoirParser<UnresolvedType>,
{
    just(Token::Ampersand)
        .ignore_then(keyword(Keyword::Mut))
        .ignore_then(type_parser)
        .map_with_span(|element, span| {
            UnresolvedTypeData::MutableReference(Box::new(element)).with_span(span)
        })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::parser::test_helpers::*;

    #[test]
    fn parse_type_expression() {
        parse_all(type_expression(), vec!["(123)", "123", "(1 + 1)", "(1 + (1))"]);
    }
}
