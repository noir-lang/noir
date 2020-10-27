use super::*;

pub struct UseParser;

impl UseParser {
    // Import statements of the form use std::hash::sha256;
    pub fn parse(parser: &mut Parser) -> Result<ImportStatement, ParserError> {
        // Current token is 'use'
        //
        // Next token should be an Identifier
        parser.peek_check_kind_advance(TokenKind::Ident)?;

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
            let token_span = path_expr.into_span();
            let ident = match path_expr.into() {
                Token::Ident(ident) => ident,
                _ => return Err(ParserError::UnstructuredError{message : format!("names in path must be identifiers"), span : token_span }),
            };

            path_as_strings.push(ident);
        }

        let stmt = ImportStatement {
            path: path_as_strings,
            alias: None,
        };
        Ok(stmt)
    }
}
