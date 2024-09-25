use crate::token::SecondaryAttribute;
use crate::token::{Attribute, Token, TokenKind};

use super::ItemKind;

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_inner_attribute(&mut self) -> Option<ItemKind> {
        let token = self.eat_kind(TokenKind::InnerAttribute)?;
        match token.into_token() {
            Token::InnerAttribute(attribute) => Some(ItemKind::InnerAttribute(attribute)),
            _ => unreachable!(),
        }
    }

    pub(super) fn parse_attributes(&mut self) -> Vec<Attribute> {
        let mut attributes: Vec<Attribute> = Vec::new();

        while let Some(token) = self.eat_kind(TokenKind::Attribute) {
            match token.into_token() {
                Token::Attribute(attribute) => {
                    attributes.push(attribute.clone());
                }
                _ => unreachable!(),
            }
        }

        attributes
    }

    pub(super) fn validate_secondary_attributes(
        &mut self,
        attributes: Vec<Attribute>,
    ) -> Vec<SecondaryAttribute> {
        attributes
            .into_iter()
            .filter_map(|attribute| {
                match attribute {
                    Attribute::Function(..) => {
                        // TODO: error
                        None
                    }
                    Attribute::Secondary(attr) => Some(attr),
                }
            })
            .collect()
    }
}
