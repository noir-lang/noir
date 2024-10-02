use crate::ast::Expression;

use super::Parser;

impl<'a> Parser<'a> {
    /// Arguments = '(' (Expression ','?)* ')'
    pub(crate) fn parse_arguments(&mut self) -> Option<Vec<Expression>> {
        if !self.eat_left_paren() {
            return None;
        }

        if self.eat_right_paren() {
            return Some(Vec::new());
        }

        let mut arguments = Vec::new();
        let mut trailing_comma = false;
        loop {
            let start_span = self.current_token_span;
            let Some(expr) = self.parse_expression() else {
                self.eat_right_paren();
                break;
            };

            if !trailing_comma && !arguments.is_empty() {
                self.expected_token_separating_items(",", "arguments", start_span);
            }

            arguments.push(expr);

            trailing_comma = self.eat_comma();
        }

        Some(arguments)
    }
}
