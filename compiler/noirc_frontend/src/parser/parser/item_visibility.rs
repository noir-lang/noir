use crate::{
    ast::ItemVisibility,
    token::{Keyword, Token},
};

use super::Parser;

impl Parser<'_> {
    /// ItemVisibility
    ///     = 'pub'                 // ItemVisibility::Public
    ///     | 'pub' '(' 'crate' ')' // ItemVisibility::PublicCrate
    ///     | nothing               // ItemVisibility::Private
    pub fn parse_item_visibility(&mut self) -> ItemVisibility {
        if !self.eat_keyword(Keyword::Pub) {
            return ItemVisibility::Private;
        }

        if !self.eat_left_paren() {
            // `pub`
            return ItemVisibility::Public;
        }

        if !self.eat_keyword(Keyword::Crate) {
            // `pub(` or `pub()`
            self.expected_token(Token::Keyword(Keyword::Crate));
            self.eat_right_paren();
            return ItemVisibility::Public;
        }

        self.eat_or_error(Token::RightParen);

        // `pub(crate)``
        ItemVisibility::PublicCrate
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::ItemVisibility,
        parser::{
            Parser,
            parser::tests::{check_errors, expect_no_errors},
        },
    };

    #[test]
    fn parses_private_visibility() {
        let src = "(";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let visibility = parser.parse_item_visibility();
        expect_no_errors(&parser.errors);
        assert_eq!(visibility, ItemVisibility::Private);
    }

    #[test]
    fn parses_public_visibility() {
        let src = "pub";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let visibility = parser.parse_item_visibility();
        expect_no_errors(&parser.errors);
        assert_eq!(visibility, ItemVisibility::Public);
    }

    #[test]
    fn parses_public_visibility_unclosed_parentheses() {
        let src = "
        pub(
           ^ Expected a 'crate' but found end of input
        ";
        let visibility = check_errors(src, |parser| parser.parse_item_visibility());
        assert_eq!(visibility, ItemVisibility::Public);
    }

    #[test]
    fn parses_public_visibility_no_crate_after_pub() {
        let src = "
        pub(hello
            ^^^^^ Expected a 'crate' but found 'hello'
        ";
        let visibility = check_errors(src, |parser| parser.parse_item_visibility());
        assert_eq!(visibility, ItemVisibility::Public);
    }
    #[test]
    fn parses_public_visibility_missing_paren_after_pub_crate() {
        let src = "
        pub(crate
                ^ Expected a ')' but found end of input
        ";
        let visibility = check_errors(src, |parser| parser.parse_item_visibility());
        assert_eq!(visibility, ItemVisibility::PublicCrate);
    }

    #[test]
    fn parses_public_crate_visibility() {
        let src = "pub(crate)";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let visibility = parser.parse_item_visibility();
        expect_no_errors(&parser.errors);
        assert_eq!(visibility, ItemVisibility::PublicCrate);
    }
}
