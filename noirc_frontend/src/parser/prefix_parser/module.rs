use super::*;

pub struct ModuleParser;

impl ModuleParser {
    pub(crate) fn parse_module_decl(parser: &mut Parser) -> Result<String, ParserError> {
        // Currently on the mod keyword
        //
        parser.peek_check_kind_advance(TokenKind::Ident)?;

        let module_identifier = match parser.curr_token.token() {
            Token::Ident(x) => x.to_string(),
            _ => unreachable!(),
        };

        parser.peek_check_variant_advance(&Token::Semicolon)?;

        Ok(module_identifier)
    }
}
