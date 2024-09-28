use noirc_errors::Span;

use crate::{
    ast::{Expression, ExpressionKind, UnresolvedTypeExpression},
    parser::ParserError,
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_type_expression(
        &mut self,
    ) -> Result<UnresolvedTypeExpression, ParserError> {
        let expr = self.parse_expression().unwrap_or(Expression {
            kind: ExpressionKind::Error,
            span: Span::from(self.previous_token_span.end()..self.previous_token_span.end()),
        });
        let span = expr.span;
        UnresolvedTypeExpression::from_expr(expr, span)
    }
}
