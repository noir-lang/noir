use crate::{Path, ast::PathKind, token::SpannedToken};

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

        let path_kind = path_kind(&parser.curr_token);

        let initial_segment = tok_to_ident(parser.curr_token.clone(), file_id)?;
        let segments = UseParser::parser_path(parser, initial_segment)?;
  
        // We do not allow `use dep` as it does not unambiguously mean anything semantically 
        if path_kind ==  PathKind::Dep && segments.len() == 1 {
            return Err(ParserErrorKind::UnstructuredError{message : format!("please append the dependency you want to import after `use dep`"), span : segments[0].0.span() }.into_err(file_id))

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

        Ok(ImportStatement {path: Path{segments, kind: path_kind},alias})
    }

    pub fn parser_path(parser: &mut Parser, initial_segment : Ident) -> Result<Vec<Ident>, ParserError> {
        
            let mut path = vec![initial_segment];
            
            // Current token should now be an identifier.
            while parser.peek_token == Token::DoubleColon {
                parser.advance_tokens(); // Now we are on the `::`

                parser.advance_tokens(); // Now we are on the next Token which should be an identifier because after every `::` is an identifier

                // Check that the token is an identifier
                path.push(tok_to_ident(parser.curr_token.clone(), parser.file_id)?);
            }

            Ok(path)
    }

}

fn tok_to_ident(spanned_tok : SpannedToken, file_id : usize) -> Result<Ident, ParserError> {
    if spanned_tok.kind() != TokenKind::Ident {
        return Err(ParserErrorKind::UnstructuredError{message : format!("names in path must be identifiers"), span : spanned_tok.into_span() }.into_err(file_id))
    }
    Ok(spanned_tok.into())
}

fn path_kind(spanned_tok : &SpannedToken) -> PathKind {
    let contents = spanned_tok.token().to_string();
    // XXX: modify the lexer and parser to have multiple conditions
    // for peek_check_kind_advance so we cna check for keyword and Ident
    // then make "crate" a keyword
    // Still undecided about "dep"
    // We could just use double colon like rust and have absolute paths
    if contents == "crate" {
        PathKind::Crate
    } else if contents == "dep" {
        PathKind::Dep
    } else {
        PathKind::Plain
    }
}