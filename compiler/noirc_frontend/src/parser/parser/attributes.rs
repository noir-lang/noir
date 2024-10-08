use noirc_errors::Span;

use crate::parser::ParserErrorReason;
use crate::token::SecondaryAttribute;
use crate::token::{Attribute, Token, TokenKind};

use super::parse_many::without_separator;
use super::Parser;

impl<'a> Parser<'a> {
    /// InnerAttribute = inner_attribute
    pub(super) fn parse_inner_attribute(&mut self) -> Option<SecondaryAttribute> {
        let token = self.eat_kind(TokenKind::InnerAttribute)?;
        match token.into_token() {
            Token::InnerAttribute(attribute) => Some(attribute),
            _ => unreachable!(),
        }
    }

    /// Attributes = attribute*
    pub(super) fn parse_attributes(&mut self) -> Vec<(Attribute, Span)> {
        self.parse_many("attributes", without_separator(), Self::parse_attribute)
    }

    fn parse_attribute(&mut self) -> Option<(Attribute, Span)> {
        self.eat_kind(TokenKind::Attribute).map(|token| match token.into_token() {
            Token::Attribute(attribute) => (attribute, self.previous_token_span),
            _ => unreachable!(),
        })
    }

    pub(super) fn validate_secondary_attributes(
        &mut self,
        attributes: Vec<(Attribute, Span)>,
    ) -> Vec<SecondaryAttribute> {
        attributes
            .into_iter()
            .filter_map(|(attribute, span)| match attribute {
                Attribute::Function(..) => {
                    self.push_error(ParserErrorReason::NoFunctionAttributesAllowedOnStruct, span);
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
        parser::{parser::tests::expect_no_errors, Parser},
        token::{Attribute, FunctionAttribute, SecondaryAttribute, TestScope},
    };

    #[test]
    fn parses_inner_attribute() {
        let src = "#!['hello]";
        let mut parser = Parser::for_str(src);
        let Some(SecondaryAttribute::Tag(custom)) = parser.parse_inner_attribute() else {
            panic!("Expected inner tag attribute");
        };
        expect_no_errors(&parser.errors);
        assert_eq!(custom.contents, "hello");
    }

    #[test]
    fn parses_attributes() {
        let src = "#[test] #[deprecated]";
        let mut parser = Parser::for_str(src);
        let mut attributes = parser.parse_attributes();
        expect_no_errors(&parser.errors);
        assert_eq!(attributes.len(), 2);

        let (attr, _) = attributes.remove(0);
        assert!(matches!(attr, Attribute::Function(FunctionAttribute::Test(TestScope::None))));

        let (attr, _) = attributes.remove(0);
        assert!(matches!(attr, Attribute::Secondary(SecondaryAttribute::Deprecated(None))));
    }
}
