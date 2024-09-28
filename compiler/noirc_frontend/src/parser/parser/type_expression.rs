use crate::{
    ast::{Expression, ExpressionKind, Literal, UnresolvedTypeExpression},
    parser::{ParserError, ParserErrorReason},
    token::Token,
    BinaryTypeOperator,
};

use acvm::acir::AcirField;

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_type_expression(
        &mut self,
    ) -> Result<UnresolvedTypeExpression, ParserError> {
        match self.parse_add_or_subtract_type_expression() {
            Some(type_expr) => Ok(type_expr),
            None => self.expected_type_expression_after_this(),
        }
    }

    fn parse_add_or_subtract_type_expression(&mut self) -> Option<UnresolvedTypeExpression> {
        let start_span = self.current_token_span;
        let mut lhs = self.parse_multiply_or_divide_or_modulo_type_expression()?;

        loop {
            let operator = if self.eat(Token::Plus) {
                BinaryTypeOperator::Addition
            } else if self.eat(Token::Minus) {
                BinaryTypeOperator::Subtraction
            } else {
                break;
            };

            match self.parse_multiply_or_divide_or_modulo_type_expression() {
                Some(rhs) => {
                    let span = self.span_since(start_span);
                    lhs = UnresolvedTypeExpression::BinaryOperation(
                        Box::new(lhs),
                        operator,
                        Box::new(rhs),
                        span,
                    );
                }
                None => {
                    self.push_expected_expression_after_this_error();
                }
            }
        }

        Some(lhs)
    }

    fn parse_multiply_or_divide_or_modulo_type_expression(
        &mut self,
    ) -> Option<UnresolvedTypeExpression> {
        let start_span = self.current_token_span;
        let mut lhs = self.parse_term_type_expression()?;

        loop {
            let operator = if self.eat(Token::Star) {
                BinaryTypeOperator::Multiplication
            } else if self.eat(Token::Slash) {
                BinaryTypeOperator::Division
            } else if self.eat(Token::Percent) {
                BinaryTypeOperator::Modulo
            } else {
                break;
            };

            match self.parse_term_type_expression() {
                Some(rhs) => {
                    let span = self.span_since(start_span);
                    lhs = UnresolvedTypeExpression::BinaryOperation(
                        Box::new(lhs),
                        operator,
                        Box::new(rhs),
                        span,
                    );
                }
                None => {
                    self.push_expected_expression_after_this_error();
                    break;
                }
            }
        }

        Some(lhs)
    }

    fn parse_term_type_expression(&mut self) -> Option<UnresolvedTypeExpression> {
        if let Some(type_expr) = self.parse_constant_type_expression() {
            return Some(type_expr);
        }

        if let Some(type_expr) = self.parses_variable_type_expression() {
            return Some(type_expr);
        }

        None
    }

    fn parse_constant_type_expression(&mut self) -> Option<UnresolvedTypeExpression> {
        let Some(int) = self.eat_int() else {
            return None;
        };

        let int = if let Some(int) = int.try_to_u32() {
            int
        } else {
            let err_expr = Expression {
                kind: ExpressionKind::Literal(Literal::Integer(int, false)),
                span: self.previous_token_span,
            };
            self.push_error(
                ParserErrorReason::InvalidTypeExpression(err_expr),
                self.previous_token_span,
            );
            0
        };

        Some(UnresolvedTypeExpression::Constant(int, self.previous_token_span))
    }

    fn parses_variable_type_expression(&mut self) -> Option<UnresolvedTypeExpression> {
        let path = self.parse_path();
        if path.is_empty() {
            None
        } else {
            Some(UnresolvedTypeExpression::Variable(path))
        }
    }

    fn expected_type_expression_after_this(
        &mut self,
    ) -> Result<UnresolvedTypeExpression, ParserError> {
        Err(ParserError::with_reason(
            ParserErrorReason::ExpectedTypeExpressionAfterThis,
            self.previous_token_span,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast::UnresolvedTypeExpression, parser::Parser, BinaryTypeOperator};

    #[test]
    fn parses_constant_type_expression() {
        let src = "42";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_type_expression().unwrap();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeExpression::Constant(n, _) = expr else {
            panic!("Expected constant");
        };
        assert_eq!(n, 42);
    }

    #[test]
    fn parses_variable_type_expression() {
        let src = "foo::bar";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_type_expression().unwrap();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeExpression::Variable(path) = expr else {
            panic!("Expected path");
        };
        assert_eq!(path.to_string(), "foo::bar");
    }

    #[test]
    fn parses_binary_type_expression() {
        let src = "1 + 2 * 3 + 4";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_type_expression().unwrap();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeExpression::BinaryOperation(lhs, operator, rhs, _) = expr else {
            panic!("Expected binary operation");
        };
        assert_eq!(lhs.to_string(), "(1 + (2 * 3))");
        assert_eq!(operator, BinaryTypeOperator::Addition);
        assert_eq!(rhs.to_string(), "4");
    }
}
