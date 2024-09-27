use crate::{ast::UnresolvedTypeExpression, parser::ParserError};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_type_expression(
        &mut self,
    ) -> Result<UnresolvedTypeExpression, ParserError> {
        let expr = self.parse_expression();
        let span = expr.span;
        UnresolvedTypeExpression::from_expr(expr, span)
    }
}
