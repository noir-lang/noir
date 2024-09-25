use noirc_errors::Span;

use crate::ast::{Expression, ExpressionKind};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_expression(&mut self) -> Expression {
        // TODO: parse other expressions

        let start_span = self.current_token_span;

        let kind = if let Some(int) = self.eat_int() {
            ExpressionKind::integer(int)
        } else {
            return Expression { kind: ExpressionKind::Error, span: Span::default() };
        };

        Expression { kind, span: self.span_since(start_span) }
    }
}
