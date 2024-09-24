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
                    self.next_token();
                }
                _ => unreachable!(),
            }
        }

        attributes
    }
}
