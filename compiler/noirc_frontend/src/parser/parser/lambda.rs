use crate::{
    ast::{ExpressionKind, Lambda, Pattern, UnresolvedType},
    parser::labels::ParsingRuleLabel,
    token::{Keyword, Token},
};

use super::{Parser, parse_many::separated_by_comma};

impl Parser<'_> {
    /// Lambda = ( 'unconstrained' )? '|' LambdaParameters? '|' ( '->' Type )? Expression
    ///
    /// LambdaParameters = LambdaParameter ( ',' LambdaParameter )? ','?
    ///
    /// LambdaParameter
    ///     = Pattern OptionalTypeAnnotation
    pub(super) fn parse_lambda(&mut self) -> Option<ExpressionKind> {
        let unconstrained = self.next_is(Token::Pipe) && self.eat_keyword(Keyword::Unconstrained);

        if !self.eat_pipe() {
            return None;
        }

        let parameters = self.parse_lambda_parameters();
        let return_type =
            if self.eat(Token::Arrow) { Some(self.parse_type_or_error()) } else { None };
        let body = self.parse_expression_or_error();

        Some(ExpressionKind::Lambda(Box::new(Lambda {
            parameters,
            return_type,
            body,
            unconstrained,
        })))
    }

    fn parse_lambda_parameters(&mut self) -> Vec<(Pattern, Option<UnresolvedType>)> {
        self.parse_many(
            "parameters",
            separated_by_comma().until(Token::Pipe),
            Self::parse_lambda_parameter,
        )
    }

    fn parse_lambda_parameter(&mut self) -> Option<(Pattern, Option<UnresolvedType>)> {
        loop {
            let Some(pattern) = self.parse_pattern() else {
                self.expected_label(ParsingRuleLabel::Pattern);

                // Let's try with the next token.
                self.bump();
                if self.at_eof() {
                    return None;
                } else {
                    continue;
                }
            };

            let typ = self.parse_optional_type_annotation();
            return Some((pattern, typ));
        }
    }
}
