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
                }
                _ => unreachable!(),
            }
        }

        comments
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    #[test]
    fn parses_inner_doc_comments() {
        let src = "//! Hello\n//! World";
        let mut parser = Parser::for_str(src);
        let comments = parser.parse_inner_doc_comments();
        assert!(parser.errors.is_empty());
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0], " Hello");
        assert_eq!(comments[1], " World");
    }

    #[test]
    fn parses_outer_doc_comments() {
        let src = "/// Hello\n/// World";
        let mut parser = Parser::for_str(src);
        let comments = parser.parse_outer_doc_comments();
        assert!(parser.errors.is_empty());
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0], " Hello");
        assert_eq!(comments[1], " World");
    }
}
