use super::*;

pub struct UseParser;

impl UseParser {
    // Import statements of the form use std::hash::sha256;
    pub fn parse(parser: &mut Parser) -> Result<ImportStatement, ParserErrorKind> {
        // Current token is 'use'
        //
        // Next token should be the first segment for the Path
        // The PathParser expects the cursor to be on the first segment
        // so we advance the parser.
        parser.advance_tokens();

        // Current token is the first path segment
        // Which is the correct condition to call the Path parser
        // as a sub procedure.
        let path = PathParser::parse(parser)?
            .into_path()
            .expect("ice : path parser did not produce a path");

        // Current token is the last identifier in the path
        //
        // Check if next token is `as` for path aliasing
        let mut alias: Option<Ident> = None;
        if parser.peek_token == Token::Keyword(Keyword::As) {
            // Current token is last path segment
            //
            // Bump cursor. Next token is `as`
            parser.advance_tokens();

            // Current token is `as`
            //
            // Bump cursor. Next token is the alias
            parser.peek_check_kind_advance(TokenKind::Ident)?;

            alias = Some(parser.curr_token.clone().into());
        }

        // Current token is the alias or the last path segment
        //
        // Peek ahead and check if the next token is `;`
        parser.peek_check_variant_advance(&Token::Semicolon)?;

        Ok(ImportStatement { path, alias })
    }
}

#[cfg(test)]
mod test {
    use crate::parser::test_parse;

    use super::UseParser;

    #[test]
    fn valid_syntax() {
        let vectors = vec![
            "use std::hash;",
            "use std;",
            "use foo::bar as hello;",
            "use bar as bar;",
        ];

        for src in vectors {
            let mut parser = test_parse(src);
            UseParser::parse(&mut parser).unwrap();
        }
    }

    #[test]
    fn invalid_syntax() {
        let vectors = vec![
            // Missing semi-colon
            "use std::hash",
            "use bar as foo",
            //
            // Missing alias
            "use std as ;",
            //
            // alias is replaced with `as`
            "use foobar as as;",
            //
            // Path ends with `::`
            "use hello:: as foo;",
        ];

        for src in vectors {
            let mut parser = test_parse(src);
            UseParser::parse(&mut parser).unwrap_err();
        }
    }
}
