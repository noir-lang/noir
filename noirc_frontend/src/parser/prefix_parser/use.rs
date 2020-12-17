use crate::token::SpannedToken;

use super::*;

pub struct UseParser;

impl UseParser {
    // Import statements of the form use std::hash::sha256;
    pub fn parse(parser: &mut Parser) -> Result<ImportStatement, ParserError> {
        let file_id = parser.file_id;
        // Current token is 'use'
        //
        // Next token should be an Identifier
        parser.peek_check_kind_advance(TokenKind::Ident)?;

        let mut path = Vec::new();
        path.push(tok_to_ident(parser.curr_token.clone(), file_id)?);
        // Current token should now be an identifier.
        while parser.peek_token == Token::DoubleColon {
            parser.advance_tokens(); // Now we are on the `::`

            parser.advance_tokens(); // Now we are on the next Token which should be an identifier because after every `::` is an identifier

            // Check that the token is an identifier
            
            path.push(tok_to_ident(parser.curr_token.clone(), file_id)?);
        }

        // Check if next token is `as` for path aliasing
        let mut alias = None;
        if parser.peek_token == Token::Keyword(Keyword::As) {
            parser.advance_tokens(); // Advance to the `as`
            parser.advance_tokens(); // Advance to the `identifier` // XXX: maybe add an expect_ident here?
            alias = Some(tok_to_ident(parser.curr_token.clone(), file_id)?);
        }

        // Imports are not included in the statement branch for the parser, which means we must clean up our own semicolons
        parser.peek_check_variant_advance(&Token::Semicolon)?;

        Ok(ImportStatement {path: path,alias,})
    }
}

fn tok_to_ident(spanned_tok : SpannedToken, file_id : usize) -> Result<Ident, ParserError> {
    if spanned_tok.kind() != TokenKind::Ident {
        return Err(ParserErrorKind::UnstructuredError{message : format!("names in path must be identifiers"), span : spanned_tok.into_span() }.into_err(file_id))
    }
    Ok(spanned_tok.into())
}
