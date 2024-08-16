use super::path::{as_trait_path, path_no_turbofish};
use super::primitives::token_kind;
use super::{
    expression_with_precedence, keyword, nothing, parenthesized, NoirParser, ParserError,
    ParserErrorReason, Precedence,
};
use crate::ast::{
    Expression, Recoverable, UnresolvedType, UnresolvedTypeData, UnresolvedTypeExpression,
};
use crate::QuotedType;

use crate::parser::labels::ParsingRuleLabel;
use crate::token::{Keyword, Token, TokenKind};

use chumsky::prelude::*;
use noirc_errors::Span;

pub fn parse_type<'a>() -> impl NoirParser<UnresolvedType> + 'a {
    recursive(parse_type_inner)
}

pub(super) fn parse_type_inner<'a>(
    recursive_type_parser: impl NoirParser<UnresolvedType> + 'a,
) -> impl NoirParser<UnresolvedType> + 'a {
    choice((
        field_type(),
        int_type(),
        bool_type(),
        string_type(),
        expr_type(),
        struct_definition_type(),
        trait_constraint_type(),
        trait_definition_type(),
        trait_impl_type(),
        function_definition_type(),
        module_type(),
        top_level_item_type(),
        type_of_quoted_types(),
        quoted_type(),
        resolved_type(),
        format_string_type(recursive_type_parser.clone()),
        named_type(recursive_type_parser.clone()),
        named_trait(recursive_type_parser.clone()),
        slice_type(recursive_type_parser.clone()),
        array_type(recursive_type_parser.clone()),
        parenthesized_type(recursive_type_parser.clone()),
        tuple_type(recursive_type_parser.clone()),
        function_type(recursive_type_parser.clone()),
        mutable_reference_type(recursive_type_parser.clone()),
        as_trait_path_type(recursive_type_parser),
    ))
}

fn as_trait_path_type<'a>(
    type_parser: impl NoirParser<UnresolvedType> + 'a,
) -> impl NoirParser<UnresolvedType> + 'a {
    as_trait_path(type_parser)
        .map_with_span(|path, span| UnresolvedTypeData::AsTraitPath(Box::new(path)).with_span(span))
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

pub(super) fn maybe_comp_time() -> impl NoirParser<bool> {
    keyword(Keyword::Comptime).or_not().map(|opt| opt.is_some())
}

pub(super) fn field_type() -> impl NoirParser<UnresolvedType> {
    keyword(Keyword::Field)
        .map_with_span(|_, span| UnresolvedTypeData::FieldElement.with_span(span))
}

pub(super) fn bool_type() -> impl NoirParser<UnresolvedType> {
    keyword(Keyword::Bool).map_with_span(|_, span| UnresolvedTypeData::Bool.with_span(span))
}

/// This is the type `Expr` - the type of a quoted, untyped expression object used for macros
pub(super) fn expr_type() -> impl NoirParser<UnresolvedType> {
    keyword(Keyword::Expr)
        .map_with_span(|_, span| UnresolvedTypeData::Quoted(QuotedType::Expr).with_span(span))
}

/// This is the type `StructDefinition` - the type of a quoted struct definition
pub(super) fn struct_definition_type() -> impl NoirParser<UnresolvedType> {
    keyword(Keyword::StructDefinition).map_with_span(|_, span| {
        UnresolvedTypeData::Quoted(QuotedType::StructDefinition).with_span(span)
    })
}

/// This is the type `TraitConstraint` - the type of a quoted trait constraint
pub(super) fn trait_constraint_type() -> impl NoirParser<UnresolvedType> {
    keyword(Keyword::TraitConstraint).map_with_span(|_, span| {
        UnresolvedTypeData::Quoted(QuotedType::TraitConstraint).with_span(span)
    })
}

pub(super) fn trait_definition_type() -> impl NoirParser<UnresolvedType> {
    keyword(Keyword::TraitDefinition).map_with_span(|_, span| {
        UnresolvedTypeData::Quoted(QuotedType::TraitDefinition).with_span(span)
    })
}

pub(super) fn trait_impl_type() -> impl NoirParser<UnresolvedType> {
    keyword(Keyword::TraitImpl)
        .map_with_span(|_, span| UnresolvedTypeData::Quoted(QuotedType::TraitImpl).with_span(span))
}

pub(super) fn function_definition_type() -> impl NoirParser<UnresolvedType> {
    keyword(Keyword::FunctionDefinition).map_with_span(|_, span| {
        UnresolvedTypeData::Quoted(QuotedType::FunctionDefinition).with_span(span)
    })
}

pub(super) fn module_type() -> impl NoirParser<UnresolvedType> {
    keyword(Keyword::Module)
        .map_with_span(|_, span| UnresolvedTypeData::Quoted(QuotedType::Module).with_span(span))
}

/// This is the type `TopLevelItem` - the type of a quoted statement in the top level.
/// E.g. a type definition, trait definition, trait impl, function, etc.
fn top_level_item_type() -> impl NoirParser<UnresolvedType> {
    keyword(Keyword::TopLevelItem).map_with_span(|_, span| {
        UnresolvedTypeData::Quoted(QuotedType::TopLevelItem).with_span(span)
    })
}

/// This is the type `Type` - the type of a quoted noir type.
fn type_of_quoted_types() -> impl NoirParser<UnresolvedType> {
    keyword(Keyword::TypeType)
        .map_with_span(|_, span| UnresolvedTypeData::Quoted(QuotedType::Type).with_span(span))
}

/// This is the type of a quoted, unparsed token stream.
fn quoted_type() -> impl NoirParser<UnresolvedType> {
    keyword(Keyword::Quoted)
        .map_with_span(|_, span| UnresolvedTypeData::Quoted(QuotedType::Quoted).with_span(span))
}

/// This is the type of an already resolved type.
/// The only way this can appear in the token input is if an already resolved `Type` object
/// was spliced into a macro's token stream via the `$` operator.
pub(super) fn resolved_type() -> impl NoirParser<UnresolvedType> {
    token_kind(TokenKind::QuotedType).map_with_span(|token, span| match token {
        Token::QuotedType(id) => UnresolvedTypeData::Resolved(id).with_span(span),
        _ => unreachable!("token_kind(QuotedType) guarantees we parse a quoted type"),
    })
}

pub(super) fn string_type() -> impl NoirParser<UnresolvedType> {
    keyword(Keyword::String)
        .ignore_then(type_expression().delimited_by(just(Token::Less), just(Token::Greater)))
        .map_with_span(|expr, span| UnresolvedTypeData::String(expr).with_span(span))
}

pub(super) fn format_string_type<'a>(
    type_parser: impl NoirParser<UnresolvedType> + 'a,
) -> impl NoirParser<UnresolvedType> + 'a {
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
    filter_map(|span, token: Token| match token {
        Token::IntType(int_type) => Ok(int_type),
        unexpected => {
            Err(ParserError::expected_label(ParsingRuleLabel::IntegerType, unexpected, span))
        }
    })
    .validate(|token, span, emit| {
        UnresolvedTypeData::from_int_token(token).map(|data| data.with_span(span)).unwrap_or_else(
            |err| {
                emit(ParserError::with_reason(ParserErrorReason::InvalidBitSize(err.0), span));
                UnresolvedType::error(span)
            },
        )
    })
}

pub(super) fn named_type<'a>(
    type_parser: impl NoirParser<UnresolvedType> + 'a,
) -> impl NoirParser<UnresolvedType> + 'a {
    path_no_turbofish().then(generic_type_args(type_parser)).map_with_span(|(path, args), span| {
        UnresolvedTypeData::Named(path, args, false).with_span(span)
    })
}

pub(super) fn named_trait<'a>(
    type_parser: impl NoirParser<UnresolvedType> + 'a,
) -> impl NoirParser<UnresolvedType> + 'a {
    keyword(Keyword::Impl)
        .ignore_then(path_no_turbofish())
        .then(generic_type_args(type_parser))
        .map_with_span(|(path, args), span| {
            UnresolvedTypeData::TraitAsType(path, args).with_span(span)
        })
}

pub(super) fn generic_type_args<'a>(
    type_parser: impl NoirParser<UnresolvedType> + 'a,
) -> impl NoirParser<Vec<UnresolvedType>> + 'a {
    required_generic_type_args(type_parser).or_not().map(Option::unwrap_or_default)
}

pub(super) fn required_generic_type_args<'a>(
    type_parser: impl NoirParser<UnresolvedType> + 'a,
) -> impl NoirParser<Vec<UnresolvedType>> + 'a {
    type_parser
        .clone()
        // Without checking for a terminating ',' or '>' here we may incorrectly
        // parse a generic `N * 2` as just the type `N` then fail when there is no
        // separator afterward. Failing early here ensures we try the `type_expression`
        // parser afterward.
        .then_ignore(one_of([Token::Comma, Token::Greater]).rewind())
        .or(type_expression_validated())
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .at_least(1)
        .delimited_by(just(Token::Less), just(Token::Greater))
}

pub(super) fn array_type<'a>(
    type_parser: impl NoirParser<UnresolvedType> + 'a,
) -> impl NoirParser<UnresolvedType> + 'a {
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

fn type_expression() -> impl NoirParser<UnresolvedTypeExpression> {
    type_expression_inner().try_map(UnresolvedTypeExpression::from_expr)
}

/// This parser is the same as `type_expression()`, however, it continues parsing and
/// emits a parser error in the case of an invalid type expression rather than halting the parser.
pub(super) fn type_expression_validated() -> impl NoirParser<UnresolvedType> {
    type_expression_inner().validate(|expr, span, emit| {
        let type_expr = UnresolvedTypeExpression::from_expr(expr, span);
        match type_expr {
            Ok(type_expression) => UnresolvedTypeData::Expression(type_expression).with_span(span),
            Err(parser_error) => {
                emit(parser_error);
                UnresolvedType::error(span)
            }
        }
    })
}

fn type_expression_inner() -> impl NoirParser<Expression> {
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

    keyword(Keyword::Unconstrained)
        .or_not()
        .then(keyword(Keyword::Fn))
        .map(|(unconstrained_token, _fn_token)| unconstrained_token.is_some())
        .then(env)
        .then(args)
        .then_ignore(just(Token::Arrow))
        .then(type_parser)
        .map_with_span(|(((unconstrained, env), args), ret), span| {
            UnresolvedTypeData::Function(args, Box::new(ret), Box::new(env), unconstrained)
                .with_span(span)
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
