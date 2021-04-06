use super::*;
use crate::ast::BinaryOpKind;
use crate::ast::ConstrainStatement;

pub struct ConstrainParser;

/// For now we disallow statement of the form `constrain x / y` as their meaning is a bit ambiguous
/// In the future, if the user inputs a disallowed operator, we may modify the AST so that it's RHS is `== 0`
///
/// We could also do it for the other expressions:
///
/// Example : `constrain x` becomes `constrain x == 0`
///
/// Example :  `constrain x / y` becomes `constrain x/y == 0`
///
///
/// XXX: Although explicitness is preferred, in this case it seems that since we will always be adding a `== 0`
/// it may be fine. Since this change is backwards compatible, it is not necessary to make an eager decision without
/// more case study.
fn disallowed_operators() -> Vec<BinaryOpKind> {
    vec![
        BinaryOpKind::And,
        BinaryOpKind::Or,
        BinaryOpKind::Divide,
        BinaryOpKind::Multiply,
    ]
}

/// Parses statements of the form
/// - constrain <EXPR> OP <EXPR>
///
/// Cursor Start : `constrain`
///
/// Cursor End : `;`
impl ConstrainParser {
    // Since == is an infix operator
    // The pratt parser will do most of the job, we just need to check that everything was correct
    pub(crate) fn parse_statement(
        parser: &mut Parser,
    ) -> Result<ConstrainStatement, ParserErrorKind> {
        parser.advance_tokens();

        let expr = parser.parse_expression(Precedence::Lowest)?;

        // Expressions are eagerly parsed as PredicateStatements before they are parsed as infix
        // We want to convert the outer predicate expression, into an infix expression.
        // It is possible to make this work for predicate statements also,
        // but Predicate statements imply extra constraints, so this should be explicit.
        let infix = match expr.kind.into_infix() {
            Some(infix) => infix,
            None => {
                let message = "Expected an infix expression since this is a constrain statement. You cannot assign values".to_string();
                return Err(ParserErrorKind::UnstructuredError {
                    message,
                    span: expr.span,
                });
            }
        };

        if infix.operator.contents == BinaryOpKind::Assign {
            let message = "Cannot use '=' with a constrain statement".to_string();
            return Err(ParserErrorKind::UnstructuredError {
                message,
                span: infix.operator.span(),
            });
        }

        if disallowed_operators().contains(&infix.operator.contents) {
            let message = format!(
                "Cannot use the {} operator in a constraint statement.",
                infix.operator.contents.as_string()
            );
            return Err(ParserErrorKind::UnstructuredError {
                message,
                span: expr.span,
            });
        }

        // XXX: Add a `help` note to tell the user to add a semi colon here
        parser.peek_check_variant_advance(&Token::Semicolon)?;

        Ok(ConstrainStatement(infix))
    }
}

#[cfg(test)]
mod test {
    use crate::{parser::test_parse, token::Token};

    use super::{disallowed_operators, ConstrainParser};

    /// This is the standard way to declare a constrain statement
    #[test]
    fn valid_syntax() {
        const SRC: &str = r#"
            constrain x == y;
        "#;

        let mut parser = test_parse(SRC);

        let start = parser.curr_token.clone();

        let _stmt = ConstrainParser::parse_statement(&mut parser).unwrap();

        let end = parser.curr_token;

        // First check that the cursor was in the right position at
        // the start and at the end
        assert_eq!(
            start.token(),
            &Token::Keyword(crate::token::Keyword::Constrain)
        );
        assert_eq!(end.token(), &Token::Semicolon);
    }
    #[test]
    fn disallowed_syntax() {
        // Currently we disallow constrain statements where the outer infix operator
        // produces a value. This would require an implicit `==` which
        // may not be intuitive to the user.
        //
        // If this is deemed useful, one would either apply a transformation
        // or interpret it with an `==` in the evaluator
        for operator in disallowed_operators() {
            let src = format!("constrain x {} y;", operator.as_string());

            let mut parser = test_parse(&src);
            let _stmt = ConstrainParser::parse_statement(&mut parser).unwrap_err();
        }
    }
    #[test]
    fn valid_general_syntax() {
        /// These are general cases which should always work.
        ///
        /// The first case is the most noteworthy. It contains two `==`
        /// The first (inner) `==` is a predicate which returns 0/1
        /// The outer layer is an infix `==` which is
        /// associated with the Constrain statement
        const VALID_1: &str = r#"
            constrain ((x + y) == k) + z == y;
        "#;
        const VALID_2: &str = r#"
            constrain (x + !y) == y;
        "#;
        const VALID_3: &str = r#"
            constrain (x ^ y) == y;
        "#;
        const VALID_4: &str = r#"
            constrain (x ^ y) == (y + m);
        "#;
        const VALID_5: &str = r#"
            constrain x + x ^ x == y | m;
        "#;

        let mut parser = test_parse(VALID_1);
        let _stmt = ConstrainParser::parse_statement(&mut parser).unwrap();
        let mut parser = test_parse(VALID_2);
        let _stmt = ConstrainParser::parse_statement(&mut parser).unwrap();
        let mut parser = test_parse(VALID_3);
        let _stmt = ConstrainParser::parse_statement(&mut parser).unwrap();
        let mut parser = test_parse(VALID_4);
        let _stmt = ConstrainParser::parse_statement(&mut parser).unwrap();
        let mut parser = test_parse(VALID_5);
        let _stmt = ConstrainParser::parse_statement(&mut parser).unwrap();
    }

    #[test]
    fn regression_skip_semi_colon() {
        /// This test previously failed, as we were allowing a statement to
        /// optionally contain a trailing semi-colon. This however is
        /// not intuitive and the trailing semi-colon's use-case will be to
        /// eventually signify that a expression returns nothing.
        ///
        /// This is so that, you can have an expression as the last item in
        /// a block.
        const SRC: &str = r#"
                constrain x == y
            }
        "#;

        let mut parser = test_parse(SRC);
        let _err = ConstrainParser::parse_statement(&mut parser).unwrap_err();
    }

    #[test]
    fn invalid_assign() {
        /// The Assignment operator is different to the equals sign
        /// and cannot be used in a constrain statement.
        const SRC: &str = r#"
                constrain x = y;
            }
        "#;

        let mut parser = test_parse(SRC);
        let _err = ConstrainParser::parse_statement(&mut parser).unwrap_err();
    }
}
