use crate::{
    parser::ParserErrorReason,
    token::{DocStyle, Token, TokenKind},
};

use super::{parse_many::without_separator, Parser};

impl<'a> Parser<'a> {
    /// InnerDocComments = inner_doc_comment*
    pub(super) fn parse_inner_doc_comments(&mut self) -> Vec<String> {
        self.parse_many("inner doc comments", without_separator(), Self::parse_inner_doc_comment)
    }

    fn parse_inner_doc_comment(&mut self) -> Option<String> {
        self.eat_kind(TokenKind::InnerDocComment).map(|token| match token.into_token() {
            Token::LineComment(comment, Some(DocStyle::Inner))
            | Token::BlockComment(comment, Some(DocStyle::Inner)) => comment,
            _ => unreachable!(),
        })
    }

    /// OuterDocComments = outer_doc_comments*
    pub(super) fn parse_outer_doc_comments(&mut self) -> Vec<String> {
        self.parse_many("outer doc comments", without_separator(), Self::parse_outer_doc_comment)
    }

    fn parse_outer_doc_comment(&mut self) -> Option<String> {
        self.eat_kind(TokenKind::OuterDocComment).map(|token| match token.into_token() {
            Token::LineComment(comment, Some(DocStyle::Outer))
            | Token::BlockComment(comment, Some(DocStyle::Outer)) => comment,
            _ => unreachable!(),
        })
    }

    /// Skips any outer doc comments but produces a warning saying that they don't document anything.
    pub(super) fn warn_on_outer_doc_comments(&mut self) {
        let location_before_doc_comments = self.current_token_location;
        let doc_comments = self.parse_outer_doc_comments();
        if !doc_comments.is_empty() {
            self.push_error(
                ParserErrorReason::DocCommentDoesNotDocumentAnything,
                location_before_doc_comments,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{parser::tests::expect_no_errors, Parser};

    #[test]
    fn parses_inner_doc_comments() {
        let src = "//! Hello\n//! World";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let comments = parser.parse_inner_doc_comments();
        expect_no_errors(&parser.errors);
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0], " Hello");
        assert_eq!(comments[1], " World");
    }

    #[test]
    fn parses_outer_doc_comments() {
        let src = "/// Hello\n/// World";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let comments = parser.parse_outer_doc_comments();
        expect_no_errors(&parser.errors);
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0], " Hello");
        assert_eq!(comments[1], " World");
    }
}
