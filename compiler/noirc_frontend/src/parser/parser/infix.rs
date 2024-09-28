use noirc_errors::{Span, Spanned};

use crate::{
    ast::{BinaryOpKind, Expression, ExpressionKind, InfixExpression},
    token::Token,
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_equal_or_not_equal(
        &mut self,
        allow_constructors: bool,
    ) -> Option<Expression> {
        let start_span = self.current_token_span;
        let mut lhs = self.parse_or(allow_constructors)?;

        loop {
            let operator = if self.eat(Token::Equal) {
                BinaryOpKind::Equal
            } else if self.eat(Token::NotEqual) {
                BinaryOpKind::NotEqual
            } else {
                break;
            };
            let operator = Spanned::from(self.previous_token_span, operator);

            let Some(rhs) = self.parse_or(allow_constructors) else {
                self.push_expected_expression_after_this_error();
                break;
            };

            lhs = self.new_infix_expression(lhs, operator, rhs, start_span);
        }

        Some(lhs)
    }

    pub(super) fn parse_or(&mut self, allow_constructors: bool) -> Option<Expression> {
        let start_span = self.current_token_span;
        let mut lhs = self.parse_and(allow_constructors)?;

        // Don't parse `x |= ...`, etc.
        if self.next_token.token() == &Token::Assign {
            return Some(lhs);
        }

        loop {
            let operator = if self.eat(Token::Pipe) {
                BinaryOpKind::Or
            } else {
                break;
            };
            let operator = Spanned::from(self.previous_token_span, operator);

            let Some(rhs) = self.parse_and(allow_constructors) else {
                self.push_expected_expression_after_this_error();
                break;
            };

            lhs = self.new_infix_expression(lhs, operator, rhs, start_span);
        }

        Some(lhs)
    }

    pub(super) fn parse_and(&mut self, allow_constructors: bool) -> Option<Expression> {
        let start_span = self.current_token_span;
        let mut lhs = self.parse_xor(allow_constructors)?;

        // Don't parse `x &= ...`, etc.
        if self.next_token.token() == &Token::Assign {
            return Some(lhs);
        }

        loop {
            let operator = if self.eat(Token::Ampersand) {
                BinaryOpKind::And
            } else {
                break;
            };
            let operator = Spanned::from(self.previous_token_span, operator);

            let Some(rhs) = self.parse_xor(allow_constructors) else {
                self.push_expected_expression_after_this_error();
                break;
            };

            lhs = self.new_infix_expression(lhs, operator, rhs, start_span);
        }

        Some(lhs)
    }

    pub(super) fn parse_xor(&mut self, allow_constructors: bool) -> Option<Expression> {
        let start_span = self.current_token_span;
        let mut lhs = self.parse_less_or_greater(allow_constructors)?;

        // Don't parse `x ^= ...`, etc.
        if self.next_token.token() == &Token::Assign {
            return Some(lhs);
        }

        loop {
            let operator = if self.eat(Token::Caret) {
                BinaryOpKind::Xor
            } else {
                break;
            };
            let operator = Spanned::from(self.previous_token_span, operator);

            let Some(rhs) = self.parse_less_or_greater(allow_constructors) else {
                self.push_expected_expression_after_this_error();
                break;
            };

            lhs = self.new_infix_expression(lhs, operator, rhs, start_span);
        }

        Some(lhs)
    }

    pub(super) fn parse_less_or_greater(&mut self, allow_constructors: bool) -> Option<Expression> {
        let start_span = self.current_token_span;
        let mut lhs = self.parse_shift(allow_constructors)?;

        loop {
            let operator = if self.eat(Token::Less) {
                BinaryOpKind::Less
            } else if self.eat(Token::LessEqual) {
                BinaryOpKind::LessEqual
            } else if self.eat(Token::Greater) {
                BinaryOpKind::Greater
            } else if self.eat(Token::GreaterEqual) {
                BinaryOpKind::GreaterEqual
            } else {
                break;
            };
            let operator = Spanned::from(self.previous_token_span, operator);

            let Some(rhs) = self.parse_shift(allow_constructors) else {
                self.push_expected_expression_after_this_error();
                break;
            };

            lhs = self.new_infix_expression(lhs, operator, rhs, start_span);
        }

        Some(lhs)
    }

    pub(super) fn parse_shift(&mut self, allow_constructors: bool) -> Option<Expression> {
        let start_span = self.current_token_span;
        let mut lhs = self.parse_add_or_subtract(allow_constructors)?;

        loop {
            let operator =
                if self.next_token.token() != &Token::Assign && self.eat(Token::ShiftLeft) {
                    BinaryOpKind::ShiftLeft
                } else
                // Right-shift (>>) is issued as two separate > tokens by the lexer as this makes it easier
                // to parse nested generic types. For normal expressions however, it means we have to manually
                // parse two greater-than tokens as a single right-shift here.
                if self.token.token() == &Token::Greater
                    && self.next_token.token() == &Token::Greater
                    && self.next_next_token.token() != &Token::Assign
                {
                    self.next_token();
                    self.next_token();
                    BinaryOpKind::ShiftRight
                } else {
                    break;
                };
            let operator = Spanned::from(self.previous_token_span, operator);

            let Some(rhs) = self.parse_add_or_subtract(allow_constructors) else {
                self.push_expected_expression_after_this_error();
                break;
            };

            lhs = self.new_infix_expression(lhs, operator, rhs, start_span);
        }

        Some(lhs)
    }

    pub(super) fn parse_add_or_subtract(&mut self, allow_constructors: bool) -> Option<Expression> {
        let start_span = self.current_token_span;
        let mut lhs = self.parse_multiply_or_divide_or_modulo(allow_constructors)?;

        // Don't parse `x += ...`, etc.
        if self.next_token.token() == &Token::Assign {
            return Some(lhs);
        }

        loop {
            let operator = if self.eat(Token::Plus) {
                BinaryOpKind::Add
            } else if self.eat(Token::Minus) {
                BinaryOpKind::Subtract
            } else {
                break;
            };
            let operator = Spanned::from(self.previous_token_span, operator);

            let Some(rhs) = self.parse_multiply_or_divide_or_modulo(allow_constructors) else {
                self.push_expected_expression_after_this_error();
                break;
            };

            lhs = self.new_infix_expression(lhs, operator, rhs, start_span);
        }

        Some(lhs)
    }

    pub(super) fn parse_multiply_or_divide_or_modulo(
        &mut self,
        allow_constructors: bool,
    ) -> Option<Expression> {
        let start_span = self.current_token_span;
        let mut lhs = self.parse_term(allow_constructors)?;

        // Don't parse `x *= ...`, etc.
        if self.next_token.token() == &Token::Assign {
            return Some(lhs);
        }

        loop {
            let operator = if self.eat(Token::Star) {
                BinaryOpKind::Multiply
            } else if self.eat(Token::Slash) {
                BinaryOpKind::Divide
            } else if self.eat(Token::Percent) {
                BinaryOpKind::Modulo
            } else {
                break;
            };
            let operator = Spanned::from(self.previous_token_span, operator);

            let Some(rhs) = self.parse_term(allow_constructors) else {
                self.push_expected_expression_after_this_error();
                break;
            };

            lhs = self.new_infix_expression(lhs, operator, rhs, start_span);
        }

        Some(lhs)
    }

    fn new_infix_expression(
        &self,
        lhs: Expression,
        operator: Spanned<BinaryOpKind>,
        rhs: Expression,
        start_span: Span,
    ) -> Expression {
        let infix_expr = InfixExpression { lhs, operator, rhs };
        let kind = ExpressionKind::Infix(Box::new(infix_expr));
        let span = self.span_since(start_span);
        Expression { kind, span }
    }
}
