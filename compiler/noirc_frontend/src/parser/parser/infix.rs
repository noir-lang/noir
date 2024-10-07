use noirc_errors::{Span, Spanned};

use crate::{
    ast::{BinaryOpKind, Expression, ExpressionKind, InfixExpression},
    token::Token,
};

use super::Parser;

impl<'a> Parser<'a> {
    /// EqualOrNotEqualExpression
    ///     = OrExpression ( ( '==' | '!=' ) OrExpression )*
    pub(super) fn parse_equal_or_not_equal(
        &mut self,
        allow_constructors: bool,
    ) -> Option<Expression> {
        self.parse_infix(allow_constructors, Parser::parse_or, |parser| {
            if parser.eat(Token::Equal) {
                Some(BinaryOpKind::Equal)
            } else if parser.eat(Token::NotEqual) {
                Some(BinaryOpKind::NotEqual)
            } else {
                None
            }
        })
    }

    /// OrExpression
    ///     = AndExpression ( '|' AndExpression )*
    pub(super) fn parse_or(&mut self, allow_constructors: bool) -> Option<Expression> {
        self.parse_infix(allow_constructors, Parser::parse_and, |parser| {
            // Don't parse `x |= ...`, etc.
            if parser.next_is(Token::Assign) {
                None
            } else if parser.eat(Token::Pipe) {
                Some(BinaryOpKind::Or)
            } else {
                None
            }
        })
    }

    /// AndExpression
    ///     = XorExpression ( '&' XorExpression )*
    pub(super) fn parse_and(&mut self, allow_constructors: bool) -> Option<Expression> {
        self.parse_infix(allow_constructors, Parser::parse_xor, |parser| {
            // Don't parse `x |= ...`, etc.
            if parser.next_is(Token::Assign) {
                None
            } else if parser.eat(Token::Ampersand) {
                Some(BinaryOpKind::And)
            } else {
                None
            }
        })
    }

    /// XorExpression
    ///     = LessOrGreaterExpression ( '^' LessOrGreaterExpression )*
    pub(super) fn parse_xor(&mut self, allow_constructors: bool) -> Option<Expression> {
        self.parse_infix(allow_constructors, Parser::parse_less_or_greater, |parser| {
            // Don't parse `x |= ...`, etc.
            if parser.next_is(Token::Assign) {
                None
            } else if parser.eat(Token::Caret) {
                Some(BinaryOpKind::Xor)
            } else {
                None
            }
        })
    }

    /// LessOrGreaterExpression
    ///     = ShiftExpression ( ( '<' | '<=' | '>' | '>=' ) ShiftExpression )*
    pub(super) fn parse_less_or_greater(&mut self, allow_constructors: bool) -> Option<Expression> {
        self.parse_infix(allow_constructors, Parser::parse_shift, |parser| {
            if parser.eat(Token::Less) {
                Some(BinaryOpKind::Less)
            } else if parser.eat(Token::LessEqual) {
                Some(BinaryOpKind::LessEqual)
            } else if parser.next_token.token() != &Token::GreaterEqual
                && parser.eat(Token::Greater)
            {
                // Make sure to skip the `>>=` case, as `>>=` is lexed as `> >=`.
                Some(BinaryOpKind::Greater)
            } else if parser.eat(Token::GreaterEqual) {
                Some(BinaryOpKind::GreaterEqual)
            } else {
                None
            }
        })
    }

    /// ShiftExpression
    ///     = AddOrSubtractExpression ( ( '<<' | '>' '>' ) AddOrSubtractExpression )*
    pub(super) fn parse_shift(&mut self, allow_constructors: bool) -> Option<Expression> {
        self.parse_infix(allow_constructors, Parser::parse_add_or_subtract, |parser| {
            if !parser.next_is(Token::Assign) && parser.eat(Token::ShiftLeft) {
                Some(BinaryOpKind::ShiftLeft)
            } else if parser.at(Token::Greater) && parser.next_is(Token::Greater) {
                // Right-shift (>>) is issued as two separate > tokens by the lexer as this makes it easier
                // to parse nested generic types. For normal expressions however, it means we have to manually
                // parse two greater-than tokens as a single right-shift here.
                parser.bump();
                parser.bump();
                Some(BinaryOpKind::ShiftRight)
            } else {
                None
            }
        })
    }

    /// AddOrSubtractExpression
    ///     = MultiplyOrDivideOrModuloExpression ( ( '+' | '-' ) MultiplyOrDivideOrModuloExpression )*
    pub(super) fn parse_add_or_subtract(&mut self, allow_constructors: bool) -> Option<Expression> {
        self.parse_infix(allow_constructors, Parser::parse_multiply_or_divide_or_modulo, |parser| {
            if parser.next_is(Token::Assign) {
                None
            } else if parser.eat(Token::Plus) {
                Some(BinaryOpKind::Add)
            } else if parser.eat(Token::Minus) {
                Some(BinaryOpKind::Subtract)
            } else {
                None
            }
        })
    }

    /// MultiplyOrDivideOrModuloExpression
    ///     = Term ( ( '*' | '/' | '%' ) Term )*
    pub(super) fn parse_multiply_or_divide_or_modulo(
        &mut self,
        allow_constructors: bool,
    ) -> Option<Expression> {
        self.parse_infix(allow_constructors, Parser::parse_term, |parser| {
            if parser.next_is(Token::Assign) {
                None
            } else if parser.eat(Token::Star) {
                Some(BinaryOpKind::Multiply)
            } else if parser.eat(Token::Slash) {
                Some(BinaryOpKind::Divide)
            } else if parser.eat(Token::Percent) {
                Some(BinaryOpKind::Modulo)
            } else {
                None
            }
        })
    }

    fn parse_infix<Next, Op>(
        &mut self,
        allow_constructors: bool,
        mut next: Next,
        mut op: Op,
    ) -> Option<Expression>
    where
        Next: FnMut(&mut Parser<'a>, bool) -> Option<Expression>,
        Op: FnMut(&mut Parser<'a>) -> Option<BinaryOpKind>,
    {
        let start_span = self.current_token_span;
        let mut lhs = next(self, allow_constructors)?;

        loop {
            let operator_start_span = self.current_token_span;
            let Some(operator) = op(self) else {
                break;
            };
            let operator = Spanned::from(operator_start_span, operator);

            let Some(rhs) = next(self, allow_constructors) else {
                self.push_expected_expression();
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
