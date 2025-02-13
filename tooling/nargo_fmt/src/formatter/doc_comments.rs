use noirc_frontend::token::{DocStyle, Token};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_inner_doc_comments(&mut self) {
        loop {
            self.skip_comments_and_whitespace();

            match &self.token {
                Token::LineComment(comment, Some(DocStyle::Inner)) => {
                    let comment = comment.clone();
                    self.write_indentation();
                    self.write_line_comment(&comment, "//!");
                    self.bump();
                    self.write_line();
                }
                Token::BlockComment(comment, Some(DocStyle::Inner)) => {
                    let comment = comment.clone();
                    self.write_indentation();
                    self.write_block_comment(&comment, "/*!");
                    self.bump();
                    self.write_line();
                }
                _ => break,
            }
        }
    }

    pub(super) fn format_outer_doc_comments(&mut self) {
        loop {
            self.skip_comments_and_whitespace();

            match &self.token {
                Token::LineComment(comment, Some(DocStyle::Outer)) => {
                    let comment = comment.clone();
                    self.write_indentation();
                    self.write_line_comment(&comment, "///");
                    self.bump();
                    self.write_line();
                }
                Token::BlockComment(comment, Some(DocStyle::Outer)) => {
                    let comment = comment.clone();
                    self.write_indentation();
                    self.write_block_comment(&comment, "/**");
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
    use crate::{assert_format, assert_format_with_config, Config};

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
that's going to be wrapped.
*/
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
that's going to be wrapped.
*/
global x: Field = 1;
";
        assert_format_wrapping_comments(src, expected, 29);
    }
}
