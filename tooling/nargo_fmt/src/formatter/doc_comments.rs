use noirc_frontend::token::{DocStyle, Token};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_inner_doc_comment(&mut self) {
        self.skip_comments_and_whitespace();
        let Token::InnerAttribute(..) = self.token else {
            panic!("Expected inner doc comment, got {:?}", self.token);
        };
        self.write_indentation();
        self.write_current_token();
        self.bump();
        self.write_line();
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
