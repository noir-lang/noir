use crate::{
    ast::{
        Expression, ExpressionKind, GenericTypeArgs, Literal, UnresolvedType, UnresolvedTypeData,
        UnresolvedTypeExpression,
    },
    parser::{ParserError, ParserErrorReason},
    token::Token,
    BinaryTypeOperator,
};

use acvm::acir::AcirField;
use noirc_errors::Span;

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
        let lhs = self.parse_multiply_or_divide_or_modulo_type_expression()?;
        Some(self.parse_add_or_subtract_type_expression_after_lhs(lhs, start_span))
    }

    fn parse_add_or_subtract_type_expression_after_lhs(
        &mut self,
        mut lhs: UnresolvedTypeExpression,
        start_span: Span,
    ) -> UnresolvedTypeExpression {
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

        lhs
    }

    fn parse_multiply_or_divide_or_modulo_type_expression(
        &mut self,
    ) -> Option<UnresolvedTypeExpression> {
        let start_span = self.current_token_span;
        let lhs = self.parse_term_type_expression()?;
        Some(self.parse_multiply_or_divide_or_modulo_type_expression_after_lhs(lhs, start_span))
    }

    fn parse_multiply_or_divide_or_modulo_type_expression_after_lhs(
        &mut self,
        mut lhs: UnresolvedTypeExpression,
        start_span: Span,
    ) -> UnresolvedTypeExpression {
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

        lhs
    }

    fn parse_term_type_expression(&mut self) -> Option<UnresolvedTypeExpression> {
        let start_span = self.current_token_span;
        if self.eat(Token::Minus) {
            return match self.parse_term_type_expression() {
                Some(rhs) => {
                    let lhs = UnresolvedTypeExpression::Constant(0, start_span);
                    let op = BinaryTypeOperator::Subtraction;
                    let span = self.span_since(start_span);
                    Some(UnresolvedTypeExpression::BinaryOperation(
                        Box::new(lhs),
                        op,
                        Box::new(rhs),
                        span,
                    ))
                }
                None => {
                    self.push_expected_expression_after_this_error();
                    None
                }
            };
        }

        self.parse_atom_type_expression()
    }

    fn parse_atom_type_expression(&mut self) -> Option<UnresolvedTypeExpression> {
        if let Some(type_expr) = self.parse_constant_type_expression() {
            return Some(type_expr);
        }

        if let Some(type_expr) = self.parse_variable_type_expression() {
            return Some(type_expr);
        }

        if let Some(type_expr) = self.parse_parenthesized_type_expression() {
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

    fn parse_variable_type_expression(&mut self) -> Option<UnresolvedTypeExpression> {
        let path = self.parse_path();
        if path.is_empty() {
            None
        } else {
            Some(UnresolvedTypeExpression::Variable(path))
        }
    }

    fn parse_parenthesized_type_expression(&mut self) -> Option<UnresolvedTypeExpression> {
        // Make sure not to parse `()` as a parenthesized expression
        if self.token.token() == &Token::LeftParen && self.next_token.token() != &Token::RightParen
        {
            self.next_token();
            match self.parse_type_expression() {
                Ok(type_expr) => {
                    if !self.eat_right_paren() {
                        // TODO: error (expected `)`)
                    }
                    Some(type_expr)
                }
                Err(error) => {
                    self.errors.push(error);
                    self.eat_right_paren();
                    None
                }
            }
        } else {
            None
        }
    }

    pub(crate) fn parse_type_or_type_expression(&mut self) -> Option<UnresolvedType> {
        let typ = self.parse_add_or_subtract_type_or_type_expression()?;
        let span = typ.span;
        Some(
            if let UnresolvedTypeData::Expression(UnresolvedTypeExpression::Variable(path)) =
                typ.typ
            {
                UnresolvedType {
                    typ: UnresolvedTypeData::Named(path, GenericTypeArgs::default(), false),
                    span,
                }
            } else {
                typ
            },
        )
    }

    fn parse_add_or_subtract_type_or_type_expression(&mut self) -> Option<UnresolvedType> {
        let start_span = self.current_token_span;
        let lhs = self.parse_multiply_or_divide_or_modulo_type_or_type_expression()?;
        if !type_is_type_expr(&lhs) {
            return Some(lhs);
        }

        let lhs = type_to_type_expr(lhs).unwrap();
        let lhs = self.parse_add_or_subtract_type_expression_after_lhs(lhs, start_span);
        Some(type_expr_to_type(lhs, self.span_since(start_span)))
    }

    fn parse_multiply_or_divide_or_modulo_type_or_type_expression(
        &mut self,
    ) -> Option<UnresolvedType> {
        let start_span = self.current_token_span;
        let lhs = self.parse_term_type_or_type_expression()?;
        if !type_is_type_expr(&lhs) {
            return Some(lhs);
        }

        let lhs = type_to_type_expr(lhs).unwrap();
        let lhs =
            self.parse_multiply_or_divide_or_modulo_type_expression_after_lhs(lhs, start_span);
        Some(type_expr_to_type(lhs, self.span_since(start_span)))
    }

    fn parse_term_type_or_type_expression(&mut self) -> Option<UnresolvedType> {
        let start_span = self.current_token_span;
        if self.eat(Token::Minus) {
            return match self.parse_term_type_expression() {
                Some(rhs) => {
                    let lhs = UnresolvedTypeExpression::Constant(0, start_span);
                    let op = BinaryTypeOperator::Subtraction;
                    let span = self.span_since(start_span);
                    let type_expr = UnresolvedTypeExpression::BinaryOperation(
                        Box::new(lhs),
                        op,
                        Box::new(rhs),
                        span,
                    );
                    let typ = UnresolvedTypeData::Expression(type_expr);
                    Some(UnresolvedType { typ, span })
                }
                None => {
                    self.push_expected_expression_after_this_error();
                    None
                }
            };
        }

        self.parse_atom_type_or_type_expression()
    }

    fn parse_atom_type_or_type_expression(&mut self) -> Option<UnresolvedType> {
        let start_span = self.current_token_span;

        let path = self.parse_path();
        if !path.is_empty() {
            let generics = self.parse_generic_type_args();
            let typ = UnresolvedTypeData::Named(path, generics, false);
            let span = self.span_since(start_span);
            return Some(UnresolvedType { typ, span });
        }

        if let Some(type_expr) = self.parse_constant_type_expression() {
            let typ = UnresolvedTypeData::Expression(type_expr);
            let span = self.span_since(start_span);
            return Some(UnresolvedType { typ, span });
        }

        if let Some(typ) = self.parse_parenthesized_type_or_type_expression() {
            return Some(typ);
        }

        self.parse_type()
    }

    fn parse_parenthesized_type_or_type_expression(&mut self) -> Option<UnresolvedType> {
        let start_span = self.current_token_span;

        if !self.eat_left_paren() {
            return None;
        }

        if self.eat_right_paren() {
            return Some(UnresolvedType {
                typ: UnresolvedTypeData::Unit,
                span: self.span_since(start_span),
            });
        }

        let Some(typ) = self.parse_type_or_type_expression() else {
            // TODO: error
            return None;
        };

        let typ_span = typ.span;
        if let UnresolvedTypeData::Expression(type_expr) = typ.typ {
            if !self.eat_right_paren() {
                // TODO: error (expected `)`)
            }
            return Some(UnresolvedType {
                typ: UnresolvedTypeData::Expression(type_expr),
                span: typ_span,
            });
        }

        if self.eat_right_paren() {
            return Some(UnresolvedType {
                typ: UnresolvedTypeData::Parenthesized(Box::new(typ)),
                span: self.span_since(start_span),
            });
        }

        let mut types = vec![typ];
        loop {
            if !self.eat_commas() {
                // TODO: error (missing comma separating tuple types)
            }

            let typ = self.parse_type_or_error();
            types.push(typ);

            if self.eat_right_paren() {
                break;
            }
        }

        Some(UnresolvedType {
            typ: UnresolvedTypeData::Tuple(types),
            span: self.span_since(start_span),
        })
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

fn type_to_type_expr(typ: UnresolvedType) -> Option<UnresolvedTypeExpression> {
    match typ.typ {
        UnresolvedTypeData::Named(var, generics, _) => {
            if generics.is_empty() {
                Some(UnresolvedTypeExpression::Variable(var))
            } else {
                None
            }
        }
        UnresolvedTypeData::Expression(type_expr) => Some(type_expr),
        _ => None,
    }
}

fn type_is_type_expr(typ: &UnresolvedType) -> bool {
    match &typ.typ {
        UnresolvedTypeData::Named(_, generics, _) => generics.is_empty(),
        UnresolvedTypeData::Expression(..) => true,
        _ => false,
    }
}

fn type_expr_to_type(lhs: UnresolvedTypeExpression, span: Span) -> UnresolvedType {
    let lhs = UnresolvedTypeData::Expression(lhs);
    UnresolvedType { typ: lhs, span }
}

#[cfg(test)]
mod tests {
    use core::panic;

    use crate::{
        ast::{UnresolvedTypeData, UnresolvedTypeExpression},
        parser::Parser,
        BinaryTypeOperator,
    };

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

    #[test]
    fn parses_parenthesized_type_expression() {
        let src = "(N)";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_type_expression().unwrap();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeExpression::Variable(path) = expr else {
            panic!("Expected variable");
        };
        assert_eq!(path.to_string(), "N");
    }

    #[test]
    fn parses_minus_type_expression() {
        let src = "-N";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_type_expression().unwrap();
        assert!(parser.errors.is_empty());
        assert_eq!(expr.to_string(), "(0 - N)");
    }

    #[test]
    fn parse_type_or_type_expression_constant() {
        let src = "42";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type_or_type_expression().unwrap();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeData::Expression(expr) = typ.typ else {
            panic!("Expected expression");
        };
        let UnresolvedTypeExpression::Constant(n, _) = expr else {
            panic!("Expected constant");
        };
        assert_eq!(n, 42);
    }

    #[test]
    fn parse_type_or_type_expression_variable() {
        let src = "foo::Bar";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type_or_type_expression().unwrap();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeData::Named(path, generics, _) = typ.typ else {
            panic!("Expected named type");
        };
        assert_eq!(path.to_string(), "foo::Bar");
        assert!(generics.is_empty());
    }

    #[test]
    fn parses_type_or_type_expression_binary() {
        let src = "1 + 2 * 3 + 4";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type_or_type_expression().unwrap();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeData::Expression(expr) = typ.typ else {
            panic!("Expected expression");
        };
        let UnresolvedTypeExpression::BinaryOperation(lhs, operator, rhs, _) = expr else {
            panic!("Expected binary operation");
        };
        assert_eq!(lhs.to_string(), "(1 + (2 * 3))");
        assert_eq!(operator, BinaryTypeOperator::Addition);
        assert_eq!(rhs.to_string(), "4");
    }

    #[test]
    fn parses_type_or_type_expression_minus() {
        let src = "-N";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type_or_type_expression().unwrap();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeData::Expression(expr) = typ.typ else {
            panic!("Expected expression");
        };
        assert_eq!(expr.to_string(), "(0 - N)");
    }

    #[test]
    fn parses_type_or_type_expression_unit() {
        let src = "()";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type_or_type_expression().unwrap();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeData::Unit = typ.typ else {
            panic!("Expected unit type");
        };
    }

    #[test]
    fn parses_type_or_type_expression_parenthesized_type() {
        let src = "(Field)";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type_or_type_expression().unwrap();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeData::Parenthesized(typ) = typ.typ else {
            panic!("Expected parenthesized type");
        };
        let UnresolvedTypeData::FieldElement = typ.typ else {
            panic!("Expected field type");
        };
    }

    #[test]
    fn parses_type_or_type_expression_parenthesized_constant() {
        let src = "(1)";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type_or_type_expression().unwrap();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeData::Expression(expr) = typ.typ else {
            panic!("Expected expression type");
        };
        assert_eq!(expr.to_string(), "1");
    }

    #[test]
    fn parses_type_or_type_expression_tuple_type() {
        let src = "(Field, bool)";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type_or_type_expression().unwrap();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeData::Tuple(types) = typ.typ else {
            panic!("Expected tuple type");
        };
        let UnresolvedTypeData::FieldElement = types[0].typ else {
            panic!("Expected field type");
        };
        let UnresolvedTypeData::Bool = types[1].typ else {
            panic!("Expected bool type");
        };
    }

    #[test]
    fn parses_type_or_type_expression_var_minus_one() {
        let src = "N - 1";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type_or_type_expression().unwrap();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeData::Expression(expr) = typ.typ else {
            panic!("Expected expression type");
        };
        assert_eq!(expr.to_string(), "(N - 1)");
    }
}
