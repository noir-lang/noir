use crate::{ast::ItemVisibility, parser::ParserErrorReason, token::Keyword};

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
            // `pub(` or `pub()`
            self.push_error(ParserErrorReason::ExpectedCrateAfterPub, self.current_token_span);
            self.eat_right_paren();
            return ItemVisibility::Public;
        }

        if !self.eat_right_paren() {
            // `pub(crate`
            self.push_error(
                ParserErrorReason::ExpectedParenAfterPubCrate,
                self.previous_token_span,
            );
        }

        // `pub(crate)``
        ItemVisibility::PublicCrate
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::ItemVisibility,
        parser::{
            parser::tests::{expect_no_errors, get_single_error_reason, get_source_with_error_span},
            Parser, ParserErrorReason,
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
        let reason = get_single_error_reason(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::ExpectedCrateAfterPub));
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
        let reason = get_single_error_reason(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::ExpectedCrateAfterPub));
    }
    #[test]
    fn parses_public_visibility_missing_paren_after_pub_crate() {
        let src = "
        pub(crate
            ^^^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        let visibility = parser.parse_item_visibility();
        assert_eq!(visibility, ItemVisibility::PublicCrate);
        let reason = get_single_error_reason(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::ExpectedParenAfterPubCrate));
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
