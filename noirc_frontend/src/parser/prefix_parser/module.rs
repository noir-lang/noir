use super::*;

pub struct ModuleParser;

impl ModuleParser {
    pub(crate) fn parse_module_definition(parser: &mut Parser) -> (String, Program) {
        if !parser.peek_check_kind_advance(TokenKind::Ident){
            panic!("Expected an Identifier after the Mod keyword")
        }

        let module_identifier = match parser.curr_token.token() {
            Token::Ident(x) => x.to_string(),
            _=> unreachable!()
        };

        if !parser.peek_check_variant_advance(&Token::LeftBrace) {            
            panic!("Expected a Left curly brace after the module identifier")
        }
        // Advance past the Left brace
        parser.advance_tokens();

        let module = parser.parse_module();

        (module_identifier, module)
    }
}