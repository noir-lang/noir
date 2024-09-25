use crate::{ast::ItemVisibility, token::Keyword};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_item_visibility(&mut self) -> ItemVisibility {
        if !self.eat_keyword(Keyword::Pub) {
            return ItemVisibility::Private;
        }

        if !self.eat_left_paren() {
            // `pub`
            return ItemVisibility::Public;
        }

        if !self.eat_keyword(Keyword::Crate) {
            // TODO: error
            // `pub(` or `pub()`
            self.eat_right_paren();
            return ItemVisibility::Public;
        }

        if !self.eat_right_paren() {
            // `pub(crate`
            // TODO: error
        }

        // `pub(crate)``
        ItemVisibility::PublicCrate
    }
}
