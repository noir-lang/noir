use crate::{
    ast::DocComment,
    parser::ParserErrorReason,
    token::{DocStyle, Token, TokenKind},
};

use super::{Parser, parse_many::without_separator};

impl Parser<'_> {
    /// InnerDocComments = inner_doc_comment*
    pub(super) fn parse_inner_doc_comments(&mut self) -> Vec<DocComment> {
        self.parse_many("inner doc comments", without_separator(), Self::parse_inner_doc_comment)
    }

    fn parse_inner_doc_comment(&mut self) -> Option<DocComment> {
        self.eat_kind(TokenKind::InnerDocComment).map(|token| {
            let location = token.location();
            match token.into_token() {
                Token::LineComment(comment, Some(DocStyle::Inner)) => {
                    let comment = fix_line_comment(comment);
                    DocComment::from(location, comment)
                }
                Token::BlockComment(comment, Some(DocStyle::Inner)) => {
                    let comment = fix_block_comment(comment);
                    DocComment::from(location, comment)
                }
                _ => unreachable!(),
            }
        })
    }

    /// OuterDocComments = OuterDocComment*
    pub(super) fn parse_outer_doc_comments(&mut self) -> Vec<DocComment> {
        self.parse_many("outer doc comments", without_separator(), Self::parse_outer_doc_comment)
    }

    /// OuterDocComment = outer_doc_comment
    pub(super) fn parse_outer_doc_comment(&mut self) -> Option<DocComment> {
        self.eat_kind(TokenKind::OuterDocComment).map(|token| {
            let location = token.location();
            match token.into_token() {
                Token::LineComment(comment, Some(DocStyle::Outer)) => {
                    let comment = fix_line_comment(comment);
                    DocComment::from(location, comment)
                }
                Token::BlockComment(comment, Some(DocStyle::Outer)) => {
                    let comment = fix_block_comment(comment);
                    DocComment::from(location, comment)
                }
                _ => unreachable!(),
            }
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

/// Strips leading ' ' from a line comment.
fn fix_line_comment(comment: String) -> String {
    if let Some(comment) = comment.strip_prefix(' ') { comment.to_string() } else { comment }
}

/// Strips leading '*' from a block comment if all non-empty lines have it.
fn fix_block_comment(comment: String) -> String {
    let all_stars = comment.lines().enumerate().all(|(index, line)| {
        if index == 0 || line.trim().is_empty() {
            // The first line never has a star. Then we ignore empty lines.
            true
        } else {
            line.trim_start().starts_with('*')
        }
    });

    let mut fixed_comment = String::new();
    for (index, line) in comment.lines().enumerate() {
        if index > 0 {
            fixed_comment.push('\n');
        }

        if all_stars {
            if let Some(line) = line.trim_start().strip_prefix("*") {
                fixed_comment.push_str(line.strip_prefix(' ').unwrap_or(line));
                continue;
            }
        }

        if let Some(line) = line.strip_prefix(' ') {
            fixed_comment.push_str(line);
            continue;
        }

        fixed_comment.push_str(line);
    }
    fixed_comment.trim().to_string()
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
        assert_eq!(comments[0].contents, "Hello");
        assert_eq!(comments[1].contents, "World");
    }

    #[test]
    fn parses_inner_block_doc_comments() {
        let src = "/*! Hello\n * World\n *\n * !\n*/";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let comments = parser.parse_inner_doc_comments();
        expect_no_errors(&parser.errors);
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].contents, "Hello\nWorld\n\n!");
    }

    #[test]
    fn parses_inner_block_doc_comments_with_indentation() {
        let src = "    /*! Hello\n     * World\n     *\n     * !\n    */";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let comments = parser.parse_inner_doc_comments();
        expect_no_errors(&parser.errors);
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].contents, "Hello\nWorld\n\n!");
    }

    #[test]
    fn parses_outer_doc_comments() {
        let src = "/// Hello\n/// World";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let comments = parser.parse_outer_doc_comments();
        expect_no_errors(&parser.errors);
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].contents, "Hello");
        assert_eq!(comments[1].contents, "World");
    }

    #[test]
    fn parses_outer_block_doc_comments() {
        let src = "/** Hello\n * World\n *\n * !\n*/";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let comments = parser.parse_outer_doc_comments();
        expect_no_errors(&parser.errors);
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].contents, "Hello\nWorld\n\n!");
    }

    #[test]
    fn parses_outer_block_doc_comments_not_every_line_has_stars() {
        let src = "/** Hello\n * World\n Oops\n * !\n*/";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let comments = parser.parse_outer_doc_comments();
        expect_no_errors(&parser.errors);
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].contents, "Hello\n* World\nOops\n* !");
    }
}
