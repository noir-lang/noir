use crate::{ast::Expression, token::Token};

use super::Parser;

pub(crate) struct CallArguments {
    pub(crate) arguments: Vec<Expression>,
    pub(crate) is_macro_call: bool,
}

impl<'a> Parser<'a> {
    /// Arguments = '(' ArgumentsList? ')'
    ///
    /// ArgumentsList = Expression ( ',' Expression )? ','?
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

    /// CallArguments = '!'? Arguments
    pub(super) fn parse_call_arguments(&mut self) -> Option<CallArguments> {
        let is_macro_call = self.tokens_follow(Token::Bang, Token::LeftParen);

        if is_macro_call {
            // Given that we expected '!' '(', it's safe to skip the '!' because the next
            // `self.parse_arguments()` will always return `Some`.
            self.next_token();
        }

        self.parse_arguments().map(|arguments| CallArguments { arguments, is_macro_call })
    }
}
