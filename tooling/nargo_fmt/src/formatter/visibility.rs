use super::Formatter;
use noirc_frontend::{
    ast::{ItemVisibility, Visibility},
    token::Keyword,
};

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

    pub(super) fn format_visibility(&mut self, visibility: Visibility) {
        self.skip_comments_and_whitespace();

        match visibility {
            Visibility::Private => (),
            Visibility::Public => {
                self.write_keyword(Keyword::Pub);
                self.write_space();
            }
            Visibility::CallData(..) => {
                self.write_keyword(Keyword::CallData);
                self.write_left_paren();
                self.skip_comments_and_whitespace();
                self.write_current_token_and_bump();
                self.skip_comments_and_whitespace();
                self.write_right_paren();
                self.write_space();
            }
            Visibility::ReturnData => {
                self.write_keyword(Keyword::ReturnData);
                self.write_space();
            }
        }
    }
}
