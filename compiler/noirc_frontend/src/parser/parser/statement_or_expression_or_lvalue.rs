use crate::{
    ast::{
        AssignOpStatement, AssignStatement, Expression, ExpressionKind, LValue, Statement,
        StatementKind,
    },
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

        // First check if it's an interned LValue
        if let Some(token) = self.eat_kind(TokenKind::InternedLValue) {
            match token.into_token() {
                Token::InternedLValue(interned) => {
                    let location = self.location_since(start_location);
                    let interned_lvalue = || LValue::Interned(interned, location);

                    // If it is, it could be something like `lvalue = expr`: check that.
                    if self.eat(Token::Assign) {
                        let expression = self.parse_expression_or_error();
                        let kind = StatementKind::Assign(AssignStatement {
                            lvalue: interned_lvalue(),
                            expression,
                        });
                        return StatementOrExpressionOrLValue::Statement(Statement {
                            kind,
                            location: self.location_since(start_location),
                        });
                    } else if let Some(op) = self.next_is_op_assign() {
                        let expression = self.parse_expression_or_error();
                        let kind = StatementKind::AssignOp(AssignOpStatement {
                            lvalue: interned_lvalue(),
                            op,
                            expression,
                        });
                        return StatementOrExpressionOrLValue::Statement(Statement {
                            kind,
                            location: self.location_since(start_location),
                        });
                    } else if self.current_is(Token::Dot) {
                        let object = Expression {
                            kind: ExpressionKind::Interned(interned),
                            location: self.location_since(start_location),
                        };
                        let access = self.parse_member_accesses_or_method_calls_after_expression(
                            object,
                            start_location,
                        );
                        if let Some(lvalue) = LValue::from_expression(access) {
                            return StatementOrExpressionOrLValue::LValue(lvalue);
                        }
                    }
                    return StatementOrExpressionOrLValue::LValue(interned_lvalue());
                }
                _ => unreachable!(),
            }
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

#[cfg(test)]
mod tests {
    use acvm::FieldElement;
    use noirc_errors::Location;

    use crate::{
        ast::{AssignOpKind, Ident, LValue, Path, StatementKind},
        node_interner::{InternedExpressionKind, NodeInterner},
        parser::{Parser, StatementOrExpressionOrLValue, parser::tests::expect_no_errors},
        token::{LocatedToken, Token, Tokens},
    };

    fn interned_lvalue() -> InternedExpressionKind {
        let location = Location::dummy();
        let mut interner = NodeInterner::default();
        let ident = Ident::new("x".to_string(), location);
        let lvalue = LValue::Path(Path::from_ident(ident));
        interner.push_lvalue(lvalue)
    }

    #[test]
    fn parses_op_assignment_for_interned_lvalue() {
        let location = Location::dummy();
        let interned = interned_lvalue();
        let quoted = Tokens(vec![
            LocatedToken::new(Token::InternedLValue(interned), location),
            LocatedToken::new(Token::Plus, location),
            LocatedToken::new(Token::Assign, location),
            LocatedToken::new(Token::Int(FieldElement::from(1u128), None), location),
        ]);

        let parser = Parser::for_tokens(quoted);
        let (parsed, warnings) = parser
            .parse_result(Parser::parse_statement_or_expression_or_lvalue)
            .expect("Expected successful parse");
        expect_no_errors(&warnings);

        let StatementOrExpressionOrLValue::Statement(statement) = parsed else {
            panic!("Expected statement");
        };
        let StatementKind::AssignOp(assign_op) = statement.kind else {
            panic!("Expected op-assignment statement");
        };
        let LValue::Interned(id, _) = assign_op.lvalue else {
            panic!("Expected interned lvalue");
        };
        assert_eq!(id, interned);
        assert!(matches!(assign_op.op.contents, AssignOpKind::Add));
        assert_eq!(assign_op.expression.to_string(), "1");
    }
}
