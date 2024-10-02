use crate::{
    ast::{ExpressionKind, Lambda, Pattern, UnresolvedType},
    parser::labels::ParsingRuleLabel,
    token::Token,
};

use super::Parser;

impl<'a> Parser<'a> {
    /// Lambda = '|' (LambdaParameter ','?)* '|' ('->' Type)? Expression
    ///
    /// LambdaParameter
    ///     = Pattern OptionalTypeAnnotation
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
            let Some(pattern) = self.parse_pattern() else {
                self.expected_label(ParsingRuleLabel::Pattern);

                // Let's try with the next token.
                self.next_token();
                if self.at_eof() {
                    break;
                } else {
                    continue;
                }
            };

            if !trailing_comma && !parameters.is_empty() {
                self.expected_token_separating_items(",", "parameters", start_span);
            }

            let typ = self.parse_optional_type_annotation();
            parameters.push((pattern, typ));

            trailing_comma = self.eat_commas();
        }

        parameters
    }
}
