use crate::ast::{Statement, StatementKind};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_statement(&mut self) -> Statement {
        let expr = self.parse_expression();
        let span = expr.span;
        Statement { kind: StatementKind::Expression(expr), span }
    }
}
