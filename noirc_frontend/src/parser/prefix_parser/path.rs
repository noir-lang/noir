use crate::{Path, PathKind};

use super::*;
pub struct PathParser;

impl PathParser {
    pub fn parse(parser: &mut Parser) -> ParserExprKindResult {
        
        let mut parsed_path = Vec::new();
        
        loop {
            let segment = PrefixParser::Name.parse(parser)?.into_ident().unwrap();
            parsed_path.push(segment);

            // Since NameParser does not advance past it's Identifier
            // Current token should now be an identifier. Lets peek at the next token to check if the path is finished
            if parser.peek_token == Token::DoubleColon {
                parser.advance_tokens(); // Advanced past the Identifier which is the current token
                parser.advance_tokens(); // Advanced past the :: which we peeked and know is there
            } else {
                break
            }
        }
        let path_kind = path_kind(parsed_path.first().expect("ice: this function triggers when there is at least one ident"));

        Ok(ExpressionKind::Path(Path{
            segments : parsed_path,
            kind: path_kind,
        }))
    }
}

fn path_kind(ident : &Ident) -> PathKind {
    let contents = &ident.0.contents;
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