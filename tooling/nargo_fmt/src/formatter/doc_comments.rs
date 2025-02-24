use noirc_frontend::token::{DocStyle, Token};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_inner_doc_comments(&mut self) {
        loop {
            self.skip_comments_and_whitespace();

            match self.token {
                Token::LineComment(_, Some(DocStyle::Inner))
                | Token::BlockComment(_, Some(DocStyle::Inner)) => {
                    self.write_indentation();
                    self.write_current_token_trimming_end();
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

            match self.token {
                Token::LineComment(_, Some(DocStyle::Outer))
                | Token::BlockComment(_, Some(DocStyle::Outer)) => {
                    self.write_indentation();
                    self.write_current_token_trimming_end();
                    self.bump();
                    self.write_line();
                }
                _ => break,
            }
        }
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
    use crate::assert_format;

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
}
