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

#[cfg(test)]
mod tests {
    use crate::{ast::ItemVisibility, parser::Parser};

    #[test]
    fn parses_private_visibility() {
        let src = "(";
        let mut parser = Parser::for_str(src);
        let visibility = parser.parse_item_visibility();
        assert!(parser.errors.is_empty());
        assert_eq!(visibility, ItemVisibility::Private);
    }

    #[test]
    fn parses_public_visibility() {
        let src = "pub";
        let mut parser = Parser::for_str(src);
        let visibility = parser.parse_item_visibility();
        assert!(parser.errors.is_empty());
        assert_eq!(visibility, ItemVisibility::Public);
    }

    #[test]
    fn parses_public_crate_visibility() {
        let src = "pub(crate)";
        let mut parser = Parser::for_str(src);
        let visibility = parser.parse_item_visibility();
        assert!(parser.errors.is_empty());
        assert_eq!(visibility, ItemVisibility::PublicCrate);
    }
}
