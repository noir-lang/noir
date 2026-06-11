use crate::{
    ast::{Expression, ExpressionKind, LValue, Statement, StatementKind},
    token::{Token, TokenKind},
};

use super::Parser;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // Tested shrinking in https://github.com/noir-lang/noir/pull/8746 with minimal memory impact
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
    ) -> StatementOrExpressionOrLValue {
        let start_location = self.current_token_location;

        // First check if it's an interned LValue. An interned lvalue is really an interned
        // expression (see `NodeInterner::push_lvalue`), so we treat it as the leading atom of
        // an expression and let the normal postfix and assignment machinery consume whatever
        // follows: `$lv`, `$lv.field`, `$lv.method(args)`, `$lv[i]`, `$lv = rhs`, `$lv += rhs`.
        if let Some(token) = self.eat_kind(TokenKind::InternedLValue) {
            let Token::InternedLValue(interned) = token.into_token() else {
                unreachable!("eat_kind(InternedLValue) only matches InternedLValue tokens")
            };

            let atom = Expression {
                kind: ExpressionKind::Interned(interned),
                location: self.location_since(start_location),
            };
            let atom = self.parse_atom_rhs_after_expression(atom, start_location);
            let kind = self.parse_assignment_or_expression_statement(atom);

            return match kind {
                // We started from a spliced lvalue, so prefer an `LValue` result whenever the
                // (postfix-extended) expression still denotes a place (`$lv`, `$lv.field`,
                // `$lv[i]`). A non-place tail such as a method call stays an `Expression`
                // rather than being silently dropped back to the bare lvalue.
                StatementKind::Expression(expr) => match LValue::from_expression(expr.clone()) {
                    Some(lvalue) => StatementOrExpressionOrLValue::LValue(lvalue),
                    None => StatementOrExpressionOrLValue::Expression(expr),
                },
                kind => StatementOrExpressionOrLValue::Statement(Statement {
                    kind,
                    location: self.location_since(start_location),
                }),
            };
        }

        // Otherwise, check if it's a statement (which in turn checks if it's an expression)
        let statement = self.parse_statement_or_error();
        if let StatementKind::Expression(expr) = statement.kind {
            StatementOrExpressionOrLValue::Expression(expr)
        } else {
            StatementOrExpressionOrLValue::Statement(statement)
        }
    }
}
