use noirc_errors::{Located, Location};

use crate::{
    ast::{BinaryOpKind, Expression, ExpressionKind, InfixExpression},
    parser::ParserErrorReason,
    token::Token,
};

use super::Parser;

macro_rules! parse_infix {
    ($self:expr, $next:expr, $operator:expr, $allow_constructors:expr) => {{
        let start_location = $self.current_token_location;
        let mut lhs = $next($self, $allow_constructors)?;

        loop {
            let operator_start_location = $self.current_token_location;
            let operator = $operator;
            let operator = Located::from(operator_start_location, operator);

            let Some(rhs) = $next($self, $allow_constructors) else {
                $self.push_expected_expression();
                break;
            };

            lhs = $self.new_infix_expression(lhs, operator, rhs, start_location);
        }

        Some(lhs)
    }};
}

impl Parser<'_> {
    /// EqualOrNotEqualExpression
    ///     = OrExpression ( ( '==' | '!=' ) OrExpression )*
    #[inline(always)]
    pub(super) fn parse_equal_or_not_equal(
        &mut self,
        allow_constructors: bool,
    ) -> Option<Expression> {
        parse_infix!(
            self,
            Parser::parse_or,
            if self.eat(Token::Equal) {
                BinaryOpKind::Equal
            } else if self.eat(Token::NotEqual) {
                BinaryOpKind::NotEqual
            } else {
                break;
            },
            allow_constructors
        )
    }

    /// OrExpression
    ///     = AndExpression ( '|' AndExpression )*
    #[inline(always)]
    pub(super) fn parse_or(&mut self, allow_constructors: bool) -> Option<Expression> {
        parse_infix!(
            self,
            Parser::parse_and,
            if self.next_is(Token::Assign) {
                break;
            } else if self.eat(Token::Pipe) {
                BinaryOpKind::Or
            } else {
                break;
            },
            allow_constructors
        )
    }

    /// AndExpression
    ///     = XorExpression ( '&' XorExpression )*
    #[inline(always)]
    pub(super) fn parse_and(&mut self, allow_constructors: bool) -> Option<Expression> {
        parse_infix!(
            self,
            Parser::parse_xor,
            // Don't parse `x |= ...`, etc.
            if self.next_is(Token::Assign) {
                break;
            } else if self.eat(Token::Ampersand) {
                BinaryOpKind::And
            } else if self.eat(Token::LogicalAnd) {
                self.push_error(ParserErrorReason::LogicalAnd, self.previous_token_location);
                BinaryOpKind::And
            } else {
                break;
            },
            allow_constructors
        )
    }

    /// XorExpression
    ///     = LessOrGreaterExpression ( '^' LessOrGreaterExpression )*
    #[inline(always)]
    pub(super) fn parse_xor(&mut self, allow_constructors: bool) -> Option<Expression> {
        parse_infix!(
            self,
            Parser::parse_less_or_greater,
            // Don't parse `x |= ...`, etc.
            if self.next_is(Token::Assign) {
                break;
            } else if self.eat(Token::Caret) {
                BinaryOpKind::Xor
            } else {
                break;
            },
            allow_constructors
        )
    }

    /// LessOrGreaterExpression
    ///     = ShiftExpression ( ( '<' | '<=' | '>' | '>=' ) ShiftExpression )*
    #[inline(always)]
    pub(super) fn parse_less_or_greater(&mut self, allow_constructors: bool) -> Option<Expression> {
        parse_infix!(
            self,
            Parser::parse_shift,
            if self.eat(Token::Less) {
                BinaryOpKind::Less
            } else if self.eat(Token::LessEqual) {
                BinaryOpKind::LessEqual
            } else if self.next_token.token() != &Token::GreaterEqual && self.eat(Token::Greater) {
                // Make sure to skip the `>>=` case, as `>>=` is lexed as `> >=`.
                BinaryOpKind::Greater
            } else if self.eat(Token::GreaterEqual) {
                BinaryOpKind::GreaterEqual
            } else {
                break;
            },
            allow_constructors
        )
    }

    /// ShiftExpression
    ///     = AddOrSubtractExpression ( ( '<<' | '>' '>' ) AddOrSubtractExpression )*
    #[inline(always)]
    pub(super) fn parse_shift(&mut self, allow_constructors: bool) -> Option<Expression> {
        parse_infix!(
            self,
            Parser::parse_add_or_subtract,
            if !self.next_is(Token::Assign) && self.eat(Token::ShiftLeft) {
                BinaryOpKind::ShiftLeft
            } else if self.at(Token::Greater) && self.next_is(Token::Greater) {
                // Right-shift (>>) is issued as two separate > tokens by the lexer as this makes it easier
                // to parse nested generic types. For normal expressions however, it means we have to manually
                // parse two greater-than tokens as a single right-shift here.
                self.bump();
                self.bump();
                BinaryOpKind::ShiftRight
            } else {
                break;
            },
            allow_constructors
        )
    }

    /// AddOrSubtractExpression
    ///     = MultiplyOrDivideOrModuloExpression ( ( '+' | '-' ) MultiplyOrDivideOrModuloExpression )*
    #[inline(always)]
    pub(super) fn parse_add_or_subtract(&mut self, allow_constructors: bool) -> Option<Expression> {
        parse_infix!(
            self,
            Parser::parse_multiply_or_divide_or_modulo,
            if self.next_is(Token::Assign) {
                break;
            } else if self.eat(Token::Plus) {
                BinaryOpKind::Add
            } else if self.eat(Token::Minus) {
                BinaryOpKind::Subtract
            } else {
                break;
            },
            allow_constructors
        )
    }

    /// MultiplyOrDivideOrModuloExpression
    ///     = Term ( ( '*' | '/' | '%' ) Term )*
    #[inline(always)]
    pub(super) fn parse_multiply_or_divide_or_modulo(
        &mut self,
        allow_constructors: bool,
    ) -> Option<Expression> {
        parse_infix!(
            self,
            Parser::parse_term,
            if self.next_is(Token::Assign) {
                break;
            } else if self.eat(Token::Star) {
                BinaryOpKind::Multiply
            } else if self.eat(Token::Slash) {
                BinaryOpKind::Divide
            } else if self.eat(Token::Percent) {
                BinaryOpKind::Modulo
            } else {
                break;
            },
            allow_constructors
        )
    }

    #[inline(always)]
    fn new_infix_expression(
        &self,
        lhs: Expression,
        operator: Located<BinaryOpKind>,
        rhs: Expression,
        start_location: Location,
    ) -> Expression {
        let infix_expr = InfixExpression { lhs, operator, rhs };
        let kind = ExpressionKind::Infix(Box::new(infix_expr));
        let location = self.location_since(start_location);
        Expression { kind, location }
    }
}
