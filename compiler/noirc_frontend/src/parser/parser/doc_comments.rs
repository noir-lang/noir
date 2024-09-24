use crate::token::{DocStyle, Token, TokenKind};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_inner_doc_comments(&mut self) -> Vec<String> {
        let mut comments: Vec<String> = Vec::new();

        while let Some(token) = self.eat_kind(TokenKind::InnerDocComment) {
            match token.into_token() {
                Token::LineComment(comment, Some(DocStyle::Inner))
                | Token::BlockComment(comment, Some(DocStyle::Inner)) => {
                    comments.push(comment);
                    self.next_token();
                }
                _ => unreachable!(),
            }
        }

        comments
    }

    pub(super) fn parse_outer_doc_comments(&mut self) -> Vec<String> {
        let mut comments: Vec<String> = Vec::new();

        while let Some(token) = self.eat_kind(TokenKind::OuterDocComment) {
            match token.into_token() {
                Token::LineComment(comment, Some(DocStyle::Outer))
                | Token::BlockComment(comment, Some(DocStyle::Outer)) => {
                    comments.push(comment);
                    self.next_token();
                }
                _ => unreachable!(),
            }
        }

        comments
    }
}
