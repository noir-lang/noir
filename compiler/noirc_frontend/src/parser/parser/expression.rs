use crate::{
    ast::{BlockExpression, Expression, ExpressionKind, Literal, Statement, StatementKind},
    parser::ParserErrorReason,
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_expression(&mut self) -> Expression {
        let start_span = self.current_token_span;
        let kind = self.parse_expression_kind();
        let span = self.span_since(start_span);

        Expression { kind, span }
    }

    fn parse_expression_kind(&mut self) -> ExpressionKind {
        if let Some(bool) = self.eat_bool() {
            return ExpressionKind::Literal(Literal::Bool(bool));
        }

        if let Some(int) = self.eat_int() {
            return ExpressionKind::integer(int);
        }

        if let Some(kind) = self.parse_parentheses_expression() {
            return kind;
        }

        if let Some(kind) = self.parse_block_expression() {
            return ExpressionKind::Block(kind);
        }

        self.push_error(ParserErrorReason::ExpectedExpression, self.current_token_span);

        ExpressionKind::Error
    }

    fn parse_parentheses_expression(&mut self) -> Option<ExpressionKind> {
        if !self.eat_left_paren() {
            return None;
        }

        if self.eat_right_paren() {
            return Some(ExpressionKind::Literal(Literal::Unit));
        }

        let mut exprs = Vec::new();
        let mut trailing_comma = false;
        loop {
            let start_span = self.current_token_span;
            let expr = self.parse_expression();
            if let ExpressionKind::Error = expr.kind {
                self.eat_right_paren();
                break;
            }
            if !trailing_comma && !exprs.is_empty() {
                self.push_error(ParserErrorReason::MissingCommaSeparatingExpressions, start_span);
            }

            exprs.push(expr);

            trailing_comma = self.eat_commas();

            if self.eat_right_paren() {
                break;
            }
        }

        Some(if exprs.len() == 1 && !trailing_comma {
            ExpressionKind::Parenthesized(Box::new(exprs.remove(0)))
        } else {
            ExpressionKind::Tuple(exprs)
        })
    }

    pub(super) fn parse_block_expression(&mut self) -> Option<BlockExpression> {
        if !self.eat_left_brace() {
            return None;
        }

        let mut statements = Vec::new();

        if self.eat_right_brace() {
            return Some(BlockExpression { statements });
        }

        let expr = self.parse_expression();
        let span = expr.span;
        statements.push(Statement { kind: StatementKind::Expression(expr), span });

        if !self.eat_right_brace() {
            // TODO: error
        }

        Some(BlockExpression { statements })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{ExpressionKind, Literal, StatementKind},
        parser::{Parser, ParserErrorReason},
    };

    #[test]
    fn parses_bool_literals() {
        let src = "true";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression();
        assert!(parser.errors.is_empty());
        assert!(matches!(expr.kind, ExpressionKind::Literal(Literal::Bool(true))));

        let src = "false";
        let expr = Parser::for_str(src).parse_expression();
        assert!(matches!(expr.kind, ExpressionKind::Literal(Literal::Bool(false))));
    }

    #[test]
    fn parses_integer_literal() {
        let src = "42";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Literal(Literal::Integer(field, negative)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(field, 42_u128.into());
        assert!(!negative);
    }

    #[test]
    fn parses_parenthesized_expression() {
        let src = "(42)";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Parenthesized(expr) = expr.kind else {
            panic!("Expected parenthesized expression");
        };
        let ExpressionKind::Literal(Literal::Integer(field, negative)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(field, 42_u128.into());
        assert!(!negative);
    }

    #[test]
    fn parses_unit() {
        let src = "()";
        let expr = Parser::for_str(src).parse_expression();
        assert!(matches!(expr.kind, ExpressionKind::Literal(Literal::Unit)));
    }

    #[test]
    fn parses_tuple_expression() {
        let src = "(1, 2)";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Tuple(mut exprs) = expr.kind else {
            panic!("Expected tuple expression");
        };
        assert_eq!(exprs.len(), 2);

        let expr = exprs.remove(0);
        let ExpressionKind::Literal(Literal::Integer(field, negative)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(field, 1_u128.into());
        assert!(!negative);

        let expr = exprs.remove(0);
        let ExpressionKind::Literal(Literal::Integer(field, negative)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(field, 2_u128.into());
        assert!(!negative);
    }

    #[test]
    fn parses_block_expression_with_a_single_expression() {
        let src = "{ 1 }";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Block(mut block) = expr.kind else {
            panic!("Expected block expression");
        };
        assert_eq!(block.statements.len(), 1);

        let statement = block.statements.remove(0);
        let StatementKind::Expression(expr) = statement.kind else {
            panic!("Expected expression statement");
        };

        let ExpressionKind::Literal(Literal::Integer(field, negative)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(field, 1_u128.into());
        assert!(!negative);
    }

    #[test]
    fn parses_unclosed_parentheses() {
        let src = "(";
        let mut parser = Parser::for_str(src);
        let _ = parser.parse_expression();
        assert_eq!(parser.errors.len(), 1);

        let error = parser.errors[0].reason().unwrap();
        assert!(matches!(error, ParserErrorReason::ExpectedExpression));
    }

    #[test]
    fn parses_missing_comma() {
        let src = "(1 2)";
        let mut parser = Parser::for_str(src);
        let _ = parser.parse_expression();
        assert_eq!(parser.errors.len(), 1);

        let error = parser.errors[0].reason().unwrap();
        assert!(matches!(error, ParserErrorReason::MissingCommaSeparatingExpressions));
    }
}
