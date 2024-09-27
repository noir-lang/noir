use crate::{ast::Expression, parser::ParserErrorReason};

use super::Parser;

impl<'a> Parser<'a> {
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
            let expr = self.parse_expression();
            if start_span == self.current_token_span {
                self.eat_right_paren();
                break;
            }

            if !trailing_comma && !arguments.is_empty() {
                self.push_error(ParserErrorReason::MissingCommaSeparatingArguments, start_span);
            }

            arguments.push(expr);

            trailing_comma = self.eat_comma();
        }

        Some(arguments)
    }
}
