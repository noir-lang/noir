use crate::{ast::Expression, token::Token};

use super::{parse_many::separated_by_comma_until_right_paren, Parser};

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

        let arguments = self.parse_many(
            "arguments",
            separated_by_comma_until_right_paren(),
            Self::parse_expression_in_list,
        );

        Some(arguments)
    }

    /// CallArguments = '!'? Arguments
    pub(super) fn parse_call_arguments(&mut self) -> Option<CallArguments> {
        let is_macro_call = self.at(Token::Bang) && self.next_is(Token::LeftParen);

        if is_macro_call {
            // Given that we expected '!' '(', it's safe to skip the '!' because the next
            // `self.parse_arguments()` will always return `Some`.
            self.bump();
        }

        self.parse_arguments().map(|arguments| CallArguments { arguments, is_macro_call })
    }
}
