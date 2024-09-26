use noirc_errors::Span;

use crate::parser::{ParserError, ParserErrorReason};
use crate::token::SecondaryAttribute;
use crate::token::{Attribute, Token, TokenKind};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_inner_attribute(&mut self) -> Option<SecondaryAttribute> {
        let token = self.eat_kind(TokenKind::InnerAttribute)?;
        match token.into_token() {
            Token::InnerAttribute(attribute) => Some(attribute),
            _ => unreachable!(),
        }
    }

    pub(super) fn parse_attributes(&mut self) -> Vec<(Attribute, Span)> {
        let mut attributes = Vec::new();

        while let Some(token) = self.eat_kind(TokenKind::Attribute) {
            match token.into_token() {
                Token::Attribute(attribute) => {
                    attributes.push((attribute, self.previous_token_span));
                }
                _ => unreachable!(),
            }
        }

        attributes
    }

    pub(super) fn validate_secondary_attributes(
        &mut self,
        attributes: Vec<(Attribute, Span)>,
    ) -> Vec<SecondaryAttribute> {
        attributes
            .into_iter()
            .filter_map(|(attribute, span)| match attribute {
                Attribute::Function(..) => {
                    self.errors.push(ParserError::with_reason(
                        ParserErrorReason::NoFunctionAttributesAllowedOnStruct,
                        span,
                    ));
                    None
                }
                Attribute::Secondary(attr) => Some(attr),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        parser::Parser,
        token::{Attribute, FunctionAttribute, SecondaryAttribute, TestScope},
    };

    #[test]
    fn parses_inner_attribute() {
        let src = "#![hello]";
        let Some(SecondaryAttribute::Custom(custom)) = Parser::for_str(src).parse_inner_attribute()
        else {
            panic!("Expected inner custom attribute");
        };
        assert_eq!(custom.contents, "hello");
    }

    #[test]
    fn parses_attributes() {
        let src = "#[test] #[deprecated]";
        let mut attributes = Parser::for_str(src).parse_attributes();
        assert_eq!(attributes.len(), 2);

        let (attr, _) = attributes.remove(0);
        assert!(matches!(attr, Attribute::Function(FunctionAttribute::Test(TestScope::None))));

        let (attr, _) = attributes.remove(0);
        assert!(matches!(attr, Attribute::Secondary(SecondaryAttribute::Deprecated(None))));
    }
}
