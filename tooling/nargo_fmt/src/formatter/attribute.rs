use noirc_frontend::token::Token;

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_attributes(&mut self) {
        loop {
            self.skip_comments_and_whitespace();

            if let Token::Attribute(_) = self.token {
                self.write_indentation();
                self.write_current_token();
                self.bump();
                self.write_line();
            } else {
                break;
            }
        }
    }

    pub(super) fn format_inner_attribute(&mut self) {
        self.skip_comments_and_whitespace();
        let Token::InnerAttribute(..) = self.token else {
            panic!("Expected inner attribute, got {:?}", self.token);
        };
        self.write_indentation();
        self.write_current_token();
        self.bump();
    }
}
