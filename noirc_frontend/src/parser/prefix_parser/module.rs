use super::*;

pub struct ModuleParser;

impl ModuleParser {
    pub(crate) fn parse_module_definition(parser: &mut Parser) -> Result<(String, Program), ParserError> {
        parser.peek_check_kind_advance(TokenKind::Ident)?;

        let module_identifier = match parser.curr_token.token() {
            Token::Ident(x) => x.to_string(),
            _=> unreachable!()
        };

        parser.peek_check_variant_advance(&Token::LeftBrace)?;
        // Advance past the Left brace
        parser.advance_tokens();

        let module = parser.parse_module();

        Ok((module_identifier, module))
    }
}