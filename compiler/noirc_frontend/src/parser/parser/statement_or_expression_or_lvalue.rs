use crate::{
    ast::{AssignStatement, CfgAttributed, Expression, LValue, Statement, StatementKind},
    token::{Token, TokenKind},
};

use super::Parser;

#[derive(Debug)]
pub enum StatementOrExpressionOrLValue {
    Statement(Statement),
    Expression(Expression),
    LValue(LValue),
}

impl Parser<'_> {
    /// Parses either a statement, an expression or an LValue. Returns `StatementKind::Error`
    /// if none can be parsed, recording an error if so.
    ///
    /// This method is only used in `Quoted::as_expr`.
    pub(crate) fn parse_statement_or_expression_or_lvalue(
        &mut self,
    ) -> CfgAttributed<StatementOrExpressionOrLValue> {
        let start_location = self.current_token_location;

        // First check if it's an interned LValue
        if let Some(token) = self.eat_kind(TokenKind::InternedLValue) {
            match token.into_token() {
                Token::InternedLValue(lvalue) => {
                    let lvalue = LValue::Interned(lvalue, self.location_since(start_location));

                    // If it is, it could be something like `lvalue = expr`: check that.
                    if self.eat(Token::Assign) {
                        let expression = self.parse_expression_or_error();
                        let kind = StatementKind::Assign(AssignStatement { lvalue, expression });
                        return StatementOrExpressionOrLValue::Statement(Statement {
                            kind,
                            location: self.location_since(start_location),
                        })
                        .into();
                    } else {
                        return StatementOrExpressionOrLValue::LValue(lvalue).into();
                    }
                }
                _ => unreachable!(),
            }
        }

        // Otherwise, check if it's a statement (which in turn checks if it's an expression)
        self.parse_statement_or_error().map(|statement| {
            if let StatementKind::Expression(expr) = statement.kind {
                StatementOrExpressionOrLValue::Expression(expr)
            } else {
                StatementOrExpressionOrLValue::Statement(statement)
            }
        })
    }
}
