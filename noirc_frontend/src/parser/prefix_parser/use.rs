use crate::ast::PathKind;

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

        let path = PathParser::parse(parser)?.into_path().expect("ice : path parser did not produce a path");
        
        // We do not allow `use dep` by itself as it does not unambiguously mean anything semantically 
        if path.kind == PathKind::Dep && path.segments.len() == 1 {
            return Err(ParserErrorKind::UnstructuredError{message : format!("please append the dependency you want to import after `use dep`"), span : path.segments[0].0.span() }.into_err(file_id))
        }

        // Check if next token is `as` for path aliasing
        let mut alias : Option<Ident> = None;
        if parser.peek_token == Token::Keyword(Keyword::As) {
            parser.advance_tokens(); // Advance to the `as`
            parser.advance_tokens(); // Advance to the `identifier` // XXX: maybe add an expect_ident here?
            
            if parser.curr_token.kind() != TokenKind::Ident {
                return Err(ParserErrorKind::UnstructuredError{message : format!("path alias must be identifiers"), span : parser.curr_token.into_span() }.into_err(file_id))
            }

            alias = Some(parser.curr_token.clone().into());
        }

        // Imports are not included in the statement branch for the parser, which means we must clean up our own semicolons
        parser.peek_check_variant_advance(&Token::Semicolon)?;

        Ok(ImportStatement {path,alias})
    }
}

