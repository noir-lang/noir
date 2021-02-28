use super::*;

pub struct ModuleParser;

impl ModuleParser {
    /// Parses a module declaration
    ///
    /// mod IDENT;
    ///
    /// Cursor Start : `mod`
    ///
    /// Cursor End : `;`
    pub(crate) fn parse_decl(parser: &mut Parser) -> Result<String, ParserErrorKind> {
        // Currently on the mod keyword
        //
        // Peek ahead and check if the next token is an identifier
        parser.peek_check_kind_advance(TokenKind::Ident)?;

        // XXX: It may be helpful to have a token to Ident function
        let module_identifier = match parser.curr_token.token() {
            Token::Ident(x) => x.to_string(),
            _ => unreachable!("ice: next token was peeked to be an Ident"),
        };

        parser.peek_check_variant_advance(&Token::Semicolon)?;

        Ok(module_identifier)
    }
}

#[cfg(test)]
mod test {
    use crate::{parser::test_parse, token::Token};

    use super::ModuleParser;

    #[test]
    fn valid_syntax() {
        const SRC: &'static str = r#"
            mod foo;
        "#;

        let mut parser = test_parse(SRC);

        let start = parser.curr_token.clone();

        ModuleParser::parse_decl(&mut parser).unwrap();

        let end = parser.curr_token.clone();

        // First check that the cursor was in the right position at
        // the start and at the end
        assert_eq!(start.token(), &Token::Keyword(crate::token::Keyword::Mod));
        assert_eq!(end.token(), &Token::Semicolon);
    }
    #[test]
    fn invalid_syntax() {
        const SRC_MISSING_SEMI_COLON: &'static str = r#"
            mod foo
        "#;
        const SRC_INT: &'static str = r#"
            mod 1;
        "#;

        ModuleParser::parse_decl(&mut test_parse(SRC_MISSING_SEMI_COLON)).unwrap_err();
        ModuleParser::parse_decl(&mut test_parse(SRC_INT)).unwrap_err();
    }
}
