use super::*;

pub struct UseParser;

impl UseParser {
    // Import statements of the form use std::hash::sha256;
    pub fn parse(parser: &mut Parser) -> ImportStatement {
        // Current token is 'use'
        //
        // Next token should be an Identifier
        if !parser.peek_check_kind_advance(TokenKind::Ident) {
            panic!("Expected an identifier after the `use` keyword")
        }

        let mut path = Vec::new();
        path.push(parser.curr_token.clone());
        // Current token should now be an identifier.
        while parser.peek_token == Token::DoubleColon {
            parser.advance_tokens(); // Now we are on the `::`

            parser.advance_tokens(); // Now we are on the next Token which should be an identifier because after every `::` is an identifier

            path.push(parser.curr_token.clone());
        }

        // XXX: Check if next token is `as` for path aliasing

        // Check and convert all of the path variables to be identifiers
        let mut path_as_strings = Vec::new();
        for path_expr in path.into_iter() {
            let ident = match path_expr {
                Token::Ident(ident) => ident,
                _ => panic!("names in path must be identifiers"),
            };

            path_as_strings.push(ident);
        }

        ImportStatement {
            path: path_as_strings,
            alias: None,
        }
    }
}
