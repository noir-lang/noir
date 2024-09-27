use crate::ast::UnresolvedTypeExpression;

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_type_expression(&mut self) -> Option<UnresolvedTypeExpression> {
        let expr = self.parse_expression();
        let span = expr.span;
        let type_expr = UnresolvedTypeExpression::from_expr(expr, span);
        match type_expr {
            Ok(type_expr) => Some(type_expr),
            Err(parser_error) => {
                self.errors.push(parser_error);
                None
            }
        }
    }
}
