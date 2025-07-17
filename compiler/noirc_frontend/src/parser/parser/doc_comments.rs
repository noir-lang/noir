use crate::{
    parser::ParserErrorReason,
    token::{DocStyle, Token, TokenKind},
};

use super::{Parser, parse_many::without_separator};

impl Parser<'_> {
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

    /// OuterDocComments = OuterDocComment*
    pub(super) fn parse_outer_doc_comments(&mut self) -> Vec<String> {
        self.parse_many("outer doc comments", without_separator(), Self::parse_outer_doc_comment)
    }

    /// OuterDocComment = outer_doc_comment
    pub(super) fn parse_outer_doc_comment(&mut self) -> Option<String> {
        self.eat_kind(TokenKind::OuterDocComment).map(|token| match token.into_token() {
            Token::LineComment(comment, Some(DocStyle::Outer))
            | Token::BlockComment(comment, Some(DocStyle::Outer)) => comment,
            _ => unreachable!(),
        })
    }

    /// Skips any outer doc comments but produces a warning saying that they don't document anything.
    pub(super) fn warn_on_outer_doc_comments(&mut self) {
        self.skip_doc_comments_with_reason(ParserErrorReason::DocCommentDoesNotDocumentAnything);
    }

    /// Skips any outer doc comments but produces an error saying that they can't be applied to parameters
    pub(super) fn error_on_outer_doc_comments_on_parameter(&mut self) {
        let reason = ParserErrorReason::DocCommentCannotBeAppliedToFunctionParameters;
        self.skip_doc_comments_with_reason(reason);
    }

    fn skip_doc_comments_with_reason(&mut self, reason: ParserErrorReason) {
        let location_before_doc_comments = self.current_token_location;
        let doc_comments = self.parse_outer_doc_comments();
        if !doc_comments.is_empty() {
            self.push_error(reason, self.location_since(location_before_doc_comments));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{Parser, parser::tests::expect_no_errors};

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
