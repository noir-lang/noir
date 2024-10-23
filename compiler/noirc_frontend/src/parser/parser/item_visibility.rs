use crate::{
    ast::ItemVisibility,
    token::{Keyword, Token},
};

use super::Parser;

impl<'a> Parser<'a> {
    /// ItemVisibility
    ///     = 'pub'                 // ItemVisibility::Public
    ///     | 'pub' '(' 'crate' ')' // ItemVisibility::PublicCrate
    ///     | nothing               // ItemVisibility::Private
    pub(super) fn parse_item_visibility(&mut self) -> ItemVisibility {
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
            parser::tests::{expect_no_errors, get_single_error, get_source_with_error_span},
            Parser,
        },
    };

    #[test]
    fn parses_private_visibility() {
        let src = "(";
        let mut parser = Parser::for_str(src);
        let visibility = parser.parse_item_visibility();
        expect_no_errors(&parser.errors);
        assert_eq!(visibility, ItemVisibility::Private);
    }

    #[test]
    fn parses_public_visibility() {
        let src = "pub";
        let mut parser = Parser::for_str(src);
        let visibility = parser.parse_item_visibility();
        expect_no_errors(&parser.errors);
        assert_eq!(visibility, ItemVisibility::Public);
    }

    #[test]
    fn parses_public_visibility_unclosed_parentheses() {
        let src = "
        pub( 
            ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        let visibility = parser.parse_item_visibility();
        assert_eq!(visibility, ItemVisibility::Public);
        let error = get_single_error(&parser.errors, span);
        assert_eq!(error.to_string(), "Expected a 'crate' but found end of input");
    }

    #[test]
    fn parses_public_visibility_no_crate_after_pub() {
        let src = "
        pub(hello
            ^^^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        let visibility = parser.parse_item_visibility();
        assert_eq!(visibility, ItemVisibility::Public);
        let error = get_single_error(&parser.errors, span);
        assert_eq!(error.to_string(), "Expected a 'crate' but found 'hello'");
    }
    #[test]
    fn parses_public_visibility_missing_paren_after_pub_crate() {
        let src = "
        pub(crate 
                 ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        let visibility = parser.parse_item_visibility();
        assert_eq!(visibility, ItemVisibility::PublicCrate);
        let error = get_single_error(&parser.errors, span);
        assert_eq!(error.to_string(), "Expected a ')' but found end of input");
    }

    #[test]
    fn parses_public_crate_visibility() {
        let src = "pub(crate)";
        let mut parser = Parser::for_str(src);
        let visibility = parser.parse_item_visibility();
        expect_no_errors(&parser.errors);
        assert_eq!(visibility, ItemVisibility::PublicCrate);
    }
}
