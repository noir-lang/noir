use crate::ast::StatementKind;
use crate::parser::{ignore_then_commit, then_commit, ParserError, ParserErrorReason};
use crate::parser::{labels::ParsingRuleLabel, parenthesized, ExprParser, NoirParser};

use crate::ast::{ConstrainKind, ConstrainStatement};
use crate::token::{Keyword, Token};

use chumsky::prelude::*;

use super::keyword;

pub(super) fn constrain<'a, P>(expr_parser: P) -> impl NoirParser<StatementKind> + 'a
where
    P: ExprParser + 'a,
{
    ignore_then_commit(
        keyword(Keyword::Constrain).labelled(ParsingRuleLabel::Statement),
        expr_parser,
    )
    .map_with_span(|expr, span| {
        StatementKind::Constrain(ConstrainStatement {
            kind: ConstrainKind::Constrain,
            arguments: vec![expr],
            span,
        })
    })
    .validate(|expr, span, emit| {
        emit(ParserError::with_reason(ParserErrorReason::ConstrainDeprecated, span));
        expr
    })
}

pub(super) fn assertion<'a, P>(expr_parser: P) -> impl NoirParser<StatementKind> + 'a
where
    P: ExprParser + 'a,
{
    let keyword = choice((
        keyword(Keyword::Assert).map(|_| ConstrainKind::Assert),
        keyword(Keyword::AssertEq).map(|_| ConstrainKind::AssertEq),
    ));

    let argument_parser = expr_parser.separated_by(just(Token::Comma)).allow_trailing();

    then_commit(keyword, parenthesized(argument_parser))
        .labelled(ParsingRuleLabel::Statement)
        .map_with_span(|(kind, arguments), span| {
            StatementKind::Constrain(ConstrainStatement { arguments, kind, span })
        })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        ast::{BinaryOpKind, ExpressionKind, Literal},
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
            StatementKind::Constrain(ConstrainStatement { arguments, .. }) => {
                let message = arguments.last().unwrap();
                match &message.kind {
                    ExpressionKind::Literal(Literal::Str(message_string)) => {
                        assert_eq!(message_string, "assertion message");
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
            assertion(expression()),
            vec![
                "assert_eq(x, y)",
                "assert_eq(((x + y) == k) + z, y)",
                "assert_eq(x + !y, y)",
                "assert_eq(x ^ y, y)",
                "assert_eq(x ^ y, y + m)",
                "assert_eq(x + x ^ x, y | m)",
            ],
        );
        match parse_with(assertion(expression()), "assert_eq(x, y, \"assertion message\")").unwrap()
        {
            StatementKind::Constrain(ConstrainStatement { arguments, .. }) => {
                let message = arguments.last().unwrap();
                match &message.kind {
                    ExpressionKind::Literal(Literal::Str(message_string)) => {
                        assert_eq!(message_string, "assertion message");
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}
