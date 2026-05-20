use noirc_frontend::token::{DocStyle, Token, TokenKind};

use super::Formatter;

impl Formatter<'_> {
    pub(super) fn format_inner_doc_comments(&mut self) {
        loop {
            self.skip_comments_and_whitespace();

            if self.token.kind() != TokenKind::InnerDocComment {
                break;
            }

            match self.bump() {
                Token::LineComment(comment, Some(DocStyle::Inner)) => {
                    let bodies = self.collect_doc_line_comment_group(comment, DocStyle::Inner);
                    self.write_indentation();
                    self.write_line_comment_group(&bodies, "//!");
                    self.write_line();
                }
                Token::BlockComment(comment, Some(DocStyle::Inner)) => {
                    self.write_indentation();
                    self.write_block_comment(&comment, "/*!");
                    self.write_line();
                }
                _ => unreachable!("Expected an inner doc comment"),
            }
        }
    }

    pub(super) fn format_outer_doc_comments(&mut self) {
        loop {
            self.skip_comments_and_whitespace();

            if self.token.kind() != TokenKind::OuterDocComment {
                break;
            }

            match self.bump() {
                Token::LineComment(comment, Some(DocStyle::Outer)) => {
                    let bodies = self.collect_doc_line_comment_group(comment, DocStyle::Outer);
                    self.write_indentation();
                    self.write_line_comment_group(&bodies, "///");
                    self.write_line();
                }
                Token::BlockComment(comment, Some(DocStyle::Outer)) => {
                    let comment = comment.clone();
                    self.write_indentation();
                    self.write_block_comment(&comment, "/**");
                    self.write_line();
                }
                _ => unreachable!("Expected an outer doc comment"),
            }
        }
    }

    /// Greedily collects the consecutive run of line-style doc comments at the same
    /// `style` (separated by a single newline) starting from a body that was already
    /// consumed via `bump`. The returned bodies are the lexer comment strings (with the
    /// leading `///` or `//!` already stripped); the caller passes them to
    /// `write_line_comment_group` so the reflow engine can recognize fenced code blocks
    /// and other paragraph-spanning markdown across multiple source lines.
    fn collect_doc_line_comment_group(&mut self, first: String, style: DocStyle) -> Vec<String> {
        let mut bodies = vec![first];
        if !self.config.wrap_comments || self.ignore_next {
            return bodies;
        }
        loop {
            let Token::Whitespace(ws) = &self.token else { break };
            let newlines = ws.chars().filter(|c| *c == '\n').count();
            if newlines != 1 {
                break;
            }
            self.bump();
            if let Token::LineComment(next_body, Some(next_style)) = &self.token
                && *next_style == style
            {
                bodies.push(next_body.clone());
                self.bump();
            } else {
                break;
            }
        }
        bodies
    }

    /// Formats outer doc comments, turning them into regular comments if they start with "Safety:"
    pub(super) fn format_outer_doc_comments_checking_safety(&mut self) {
        self.skip_comments_and_whitespace();

        let is_safety = match &self.token {
            Token::LineComment(comment, Some(DocStyle::Outer))
            | Token::BlockComment(comment, Some(DocStyle::Outer)) => {
                comment.trim().to_lowercase().starts_with("safety:")
            }
            _ => false,
        };
        if !is_safety {
            return self.format_outer_doc_comments();
        }

        loop {
            self.skip_comments_and_whitespace();

            match self.token {
                Token::LineComment(_, Some(DocStyle::Outer)) => {
                    self.write_indentation();
                    let string = self.token.to_string();
                    let string = string.trim_end();
                    let string = string.replacen("///", "//", 1);
                    self.write(&string);
                    self.bump();
                    self.write_line();
                }
                Token::BlockComment(_, Some(DocStyle::Outer)) => {
                    self.write_indentation();
                    let string = self.token.to_string();
                    let string = string.trim_end();
                    let string = string.replacen("/**", "/*", 1);
                    self.write(&string);
                    self.bump();
                    self.write_line();
                }
                _ => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Config, assert_format, assert_format_with_config};

    fn assert_format_wrapping_comments(src: &str, expected: &str, comment_width: usize) {
        let config = Config { wrap_comments: true, comment_width, ..Config::default() };
        assert_format_with_config(src, expected, config);
    }

    #[test]
    fn format_inner_doc_comments() {
        let src = " #![hello] #![world]";
        let expected = "#![hello]\n#![world]\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_inner_doc_comments_with_line_comments() {
        let src = " #![hello]  // foo   
   // bar   
    #![world]";
        let expected = "#![hello] // foo
// bar
#![world]
";
        assert_format(src, expected);
    }

    #[test]
    fn format_inner_doc_comments_with_block_comments() {
        let src = " #![hello]    /* foo */    #![world]";
        let expected = "#![hello] /* foo */
#![world]
";
        assert_format(src, expected);
    }

    #[test]
    fn wraps_line_outer_doc_comments() {
        let src = "
        /// This is a long comment that's going to be wrapped.
        global x: Field = 1;
        ";
        let expected = "/// This is a long comment
/// that's going to be
/// wrapped.
global x: Field = 1;
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn wraps_line_inner_doc_comments() {
        let src = "
        //! This is a long comment that's going to be wrapped.
        global x: Field = 1;
        ";
        let expected = "//! This is a long comment
//! that's going to be
//! wrapped.
global x: Field = 1;
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn wraps_block_outer_doc_comments() {
        let src = "
        /** This is a long comment that's going to be wrapped. */
        global x: Field = 1;
        ";
        let expected = "/** This is a long comment
 * that's going to be
 * wrapped. */
global x: Field = 1;
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn wraps_block_inner_doc_comments() {
        let src = "
        /*! This is a long comment that's going to be wrapped. */
        global x: Field = 1;
        ";
        let expected = "/*! This is a long comment
 * that's going to be
 * wrapped. */
global x: Field = 1;
";
        assert_format_wrapping_comments(src, expected, 29);
    }

    #[test]
    fn doc_comment_followed_by_comment() {
        let src = "
        /// Some doc comment
        // Some comment
        global x: Field = 1;
        ";
        let expected = "/// Some doc comment
// Some comment
global x: Field = 1;
";
        assert_format(src, expected);
    }

    #[test]
    fn does_not_wrap_fenced_code_block_in_outer_doc_comment() {
        let src = "/// ```
/// fn foo() { x + y + z + a + b + c + d + e + f }
/// ```
fn bar() {}
";
        assert_format_wrapping_comments(src, src, 30);
    }

    #[test]
    fn does_not_wrap_fenced_code_block_in_middle_of_outer_doc_comment() {
        let src = "/// Example below.
///
/// ```
/// fn foo() { x + y + z + a + b + c + d + e + f }
/// ```
///
/// After the fence.
fn bar() {}
";
        assert_format_wrapping_comments(src, src, 30);
    }

    #[test]
    fn does_not_wrap_fenced_code_block_in_top_level_block_comment() {
        let src = "/**
 * Example below.
 *
 * ```
 * fn foo() { x + y + z + a + b + c + d + e + f }
 * ```
 *
 * After the fence.
 */
fn bar() {}
";
        assert_format_wrapping_comments(src, src, 30);
    }

    #[test]
    fn does_not_wrap_fenced_code_block_in_inner_doc_comment() {
        let src = "//! ```
//! fn foo() { x + y + z + a + b + c + d + e + f }
//! ```
";
        assert_format_wrapping_comments(src, src, 30);
    }

    #[test]
    fn reflows_doc_comment_paragraph_outside_fence() {
        let src = "/// Hello world, I just realized that this
/// is a long comment
fn bar() {}
";
        let expected = "/// Hello world, I just realized
/// that this is a long comment
fn bar() {}
";
        assert_format_wrapping_comments(src, expected, 35);
    }

    #[test]
    fn format_struct_outer_doc_comment_after_attribute() {
        let src = "#[derive(Eq)]
/// A struct
struct Foo {}\n";
        assert_format(src, src);
    }
}
