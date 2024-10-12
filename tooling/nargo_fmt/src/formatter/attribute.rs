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
}
