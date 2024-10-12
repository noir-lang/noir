use super::Formatter;
use noirc_frontend::{ast::ItemVisibility, token::Keyword};

impl<'a> Formatter<'a> {
    pub(super) fn format_item_visibility(&mut self, visibility: ItemVisibility) {
        self.skip_comments_and_whitespace();

        match visibility {
            ItemVisibility::Private => (),
            ItemVisibility::PublicCrate => {
                self.write_keyword(Keyword::Pub);
                self.write_left_paren();
                self.write_keyword(Keyword::Crate);
                self.write_right_paren();
                self.write_space();
            }
            ItemVisibility::Public => {
                self.write_keyword(Keyword::Pub);
                self.write_space();
            }
        }
    }
}
