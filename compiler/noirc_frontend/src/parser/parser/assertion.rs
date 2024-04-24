use crate::ast::{Expression, ExpressionKind, StatementKind};
use crate::parser::{
    ignore_then_commit, labels::ParsingRuleLabel, parenthesized, ExprParser, NoirParser,
    ParserError, ParserErrorReason,
};

use crate::ast::{BinaryOpKind, ConstrainKind, ConstrainStatement, InfixExpression, Recoverable};
use crate::token::{Keyword, Token};

use chumsky::prelude::*;
use noirc_errors::Spanned;

use super::keyword;

pub(super) fn constrain<'a, P>(expr_parser: P) -> impl NoirParser<StatementKind> + 'a
where
    P: ExprParser + 'a,
{
    ignore_then_commit(
        keyword(Keyword::Constrain).labelled(ParsingRuleLabel::Statement),
        expr_parser,
    )
    .map(|expr| StatementKind::Constrain(ConstrainStatement(expr, None, ConstrainKind::Constrain)))
    .validate(|expr, span, emit| {
        emit(ParserError::with_reason(ParserErrorReason::ConstrainDeprecated, span));
        expr
    })
}

pub(super) fn assertion<'a, P>(expr_parser: P) -> impl NoirParser<StatementKind> + 'a
where
    P: ExprParser + 'a,
{
    let argument_parser =
        expr_parser.separated_by(just(Token::Comma)).allow_trailing().at_least(1).at_most(2);

    ignore_then_commit(keyword(Keyword::Assert), parenthesized(argument_parser))
        .labelled(ParsingRuleLabel::Statement)
        .validate(|expressions, span, _| {
            let condition = expressions.first().unwrap_or(&Expression::error(span)).clone();
            let message = expressions.get(1).cloned();
            StatementKind::Constrain(ConstrainStatement(condition, message, ConstrainKind::Assert))
        })
}

pub(super) fn assertion_eq<'a, P>(expr_parser: P) -> impl NoirParser<StatementKind> + 'a
where
    P: ExprParser + 'a,
{
    let argument_parser =
        expr_parser.separated_by(just(Token::Comma)).allow_trailing().at_least(2).at_most(3);

    ignore_then_commit(keyword(Keyword::AssertEq), parenthesized(argument_parser))
        .labelled(ParsingRuleLabel::Statement)
        .validate(|exprs: Vec<Expression>, span, _| {
            let predicate = Expression::new(
                ExpressionKind::Infix(Box::new(InfixExpression {
                    lhs: exprs.first().unwrap_or(&Expression::error(span)).clone(),
                    rhs: exprs.get(1).unwrap_or(&Expression::error(span)).clone(),
                    operator: Spanned::from(span, BinaryOpKind::Equal),
                })),
                span,
            );
            let message = exprs.get(2).cloned();
            StatementKind::Constrain(ConstrainStatement(
                predicate,
                message,
                ConstrainKind::AssertEq,
            ))
        })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        ast::Literal,
        parser::parser::{
            expression,
            test_helpers::{parse_all, parse_all_failing, parse_with},
        },
    };

    /// Deprecated constrain usage test
    #[test]
    fn parse_constrain() {
        let errors = parse_with(constrain(expression()), "constrain x == y").unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(format!("{}", errors.first().unwrap()).contains("deprecated"));

        // Currently we disallow constrain statements where the outer infix operator
        // produces a value. This would require an implicit `==` which
        // may not be intuitive to the user.
        //
        // If this is deemed useful, one would either apply a transformation
        // or interpret it with an `==` in the evaluator
        let disallowed_operators = vec![
            BinaryOpKind::And,
            BinaryOpKind::Subtract,
            BinaryOpKind::Divide,
            BinaryOpKind::Multiply,
            BinaryOpKind::Or,
        ];

        for operator in disallowed_operators {
            let src = format!("constrain x {} y;", operator.as_string());
            let errors = parse_with(constrain(expression()), &src).unwrap_err();
            assert_eq!(errors.len(), 2);
            assert!(format!("{}", errors.first().unwrap()).contains("deprecated"));
        }

        // These are general cases which should always work.
        //
        // The first case is the most noteworthy. It contains two `==`
        // The first (inner) `==` is a predicate which returns 0/1
        // The outer layer is an infix `==` which is
        // associated with the Constrain statement
        let errors = parse_all_failing(
            constrain(expression()),
            vec![
                "constrain ((x + y) == k) + z == y",
                "constrain (x + !y) == y",
                "constrain (x ^ y) == y",
                "constrain (x ^ y) == (y + m)",
                "constrain x + x ^ x == y | m",
            ],
        );
        assert_eq!(errors.len(), 5);
        assert!(errors
            .iter()
            .all(|err| { err.is_error() && err.to_string().contains("deprecated") }));
    }

    /// This is the standard way to declare an assert statement
    #[test]
    fn parse_assert() {
        parse_with(assertion(expression()), "assert(x == y)").unwrap();

        // Currently we disallow constrain statements where the outer infix operator
        // produces a value. This would require an implicit `==` which
        // may not be intuitive to the user.
        //
        // If this is deemed useful, one would either apply a transformation
        // or interpret it with an `==` in the evaluator
        let disallowed_operators = vec![
            BinaryOpKind::And,
            BinaryOpKind::Subtract,
            BinaryOpKind::Divide,
            BinaryOpKind::Multiply,
            BinaryOpKind::Or,
        ];

        for operator in disallowed_operators {
            let src = format!("assert(x {} y);", operator.as_string());
            parse_with(assertion(expression()), &src).unwrap_err();
        }

        // These are general cases which should always work.
        //
        // The first case is the most noteworthy. It contains two `==`
        // The first (inner) `==` is a predicate which returns 0/1
        // The outer layer is an infix `==` which is
        // associated with the Constrain statement
        parse_all(
            assertion(expression()),
            vec![
                "assert(((x + y) == k) + z == y)",
                "assert((x + !y) == y)",
                "assert((x ^ y) == y)",
                "assert((x ^ y) == (y + m))",
                "assert(x + x ^ x == y | m)",
            ],
        );

        match parse_with(assertion(expression()), "assert(x == y, \"assertion message\")").unwrap()
        {
            StatementKind::Constrain(ConstrainStatement(_, message, _)) => {
                let message = message.unwrap();
                match message.kind {
                    ExpressionKind::Literal(Literal::Str(message_string)) => {
                        assert_eq!(message_string, "assertion message".to_owned());
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    /// This is the standard way to assert that two expressions are equivalent
    #[test]
    fn parse_assert_eq() {
        parse_all(
            assertion_eq(expression()),
            vec![
                "assert_eq(x, y)",
                "assert_eq(((x + y) == k) + z, y)",
                "assert_eq(x + !y, y)",
                "assert_eq(x ^ y, y)",
                "assert_eq(x ^ y, y + m)",
                "assert_eq(x + x ^ x, y | m)",
            ],
        );
        match parse_with(assertion_eq(expression()), "assert_eq(x, y, \"assertion message\")")
            .unwrap()
        {
            StatementKind::Constrain(ConstrainStatement(_, message, _)) => {
                let message = message.unwrap();
                match message.kind {
                    ExpressionKind::Literal(Literal::Str(message_string)) => {
                        assert_eq!(message_string, "assertion message".to_owned());
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}
