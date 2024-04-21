use chumsky::{primitive::just, Parser};

use super::{parse_type, pattern};
use crate::ast::{Expression, ExpressionKind, Lambda, Pattern, UnresolvedType};
use crate::{
    parser::{labels::ParsingRuleLabel, parameter_name_recovery, parameter_recovery, NoirParser},
    token::Token,
};

pub(super) fn lambda<'a>(
    expr_parser: impl NoirParser<Expression> + 'a,
) -> impl NoirParser<ExpressionKind> + 'a {
    lambda_parameters()
        .delimited_by(just(Token::Pipe), just(Token::Pipe))
        .then(lambda_return_type())
        .then(expr_parser)
        .map(|((parameters, return_type), body)| {
            ExpressionKind::Lambda(Box::new(Lambda { parameters, return_type, body }))
        })
}

fn lambda_parameters() -> impl NoirParser<Vec<(Pattern, UnresolvedType)>> {
    let typ = parse_type().recover_via(parameter_recovery());
    let typ = just(Token::Colon).ignore_then(typ);

    let parameter = pattern()
        .recover_via(parameter_name_recovery())
        .then(typ.or_not().map(|typ| typ.unwrap_or_else(UnresolvedType::unspecified)));

    parameter
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .labelled(ParsingRuleLabel::Parameter)
}

fn lambda_return_type() -> impl NoirParser<UnresolvedType> {
    just(Token::Arrow)
        .ignore_then(parse_type())
        .or_not()
        .map(|ret| ret.unwrap_or_else(UnresolvedType::unspecified))
}
