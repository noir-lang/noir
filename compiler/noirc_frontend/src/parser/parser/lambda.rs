use crate::{
    ast::{ExpressionKind, Lambda, Pattern, UnresolvedType},
    parser::ParserErrorReason,
    token::Token,
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_lambda(&mut self) -> Option<ExpressionKind> {
        if !self.eat_pipe() {
            return None;
        }

        let parameters = self.parse_lambda_parameters();
        let return_type = if self.eat(Token::Arrow) {
            self.parse_type_or_error()
        } else {
            self.unspecified_type_at_previous_token_end()
        };
        let body = self.parse_expression_or_error();

        Some(ExpressionKind::Lambda(Box::new(Lambda { parameters, return_type, body })))
    }

    fn parse_lambda_parameters(&mut self) -> Vec<(Pattern, UnresolvedType)> {
        let mut parameters = Vec::new();
        let mut trailing_comma = false;

        loop {
            if self.eat_pipe() {
                break;
            }

            let start_span = self.current_token_span;
            let pattern = self.parse_pattern();
            if self.current_token_span == start_span {
                // An error was already produced by parse_pattern().
                // Let's try with the next token.
                self.next_token();
                if self.is_eof() {
                    break;
                }
                continue;
            }

            if !trailing_comma && !parameters.is_empty() {
                self.push_error(ParserErrorReason::MissingCommaSeparatingParameters, start_span);
            }

            let typ = self.parse_optional_type_annotation();
            parameters.push((pattern, typ));

            trailing_comma = self.eat_commas();
        }

        parameters
    }
}
