use super::*;

pub struct FuncParser;

impl FuncParser {
    /// All functions are function definitions. Functions are not first class.
    pub(crate) fn parse_fn_definition(
        parser: &mut Parser,
    ) -> FunctionDefinition {

        // Check if we have an identifier.
        if !parser.peek_check_kind_advance(TokenKind::Ident) {
            panic!("Expected the function name after the Function keyword")
        }
        let func_name = parser.curr_token.token().to_string();
        
        if !parser.peek_check_variant_advance(&Token::LeftParen) {
            panic!(
                "Expected a Left parenthesis after the function name, found {}",
                parser.curr_token.token()
            )
        };
        let parameters = FuncParser::parse_fn_parameters(parser);

        // Parse the type after the parameters have been parsed
        let mut return_type = Type::Unit;
        if parser.peek_token == Token::Arrow {
            parser.advance_tokens(); // Advance past the `)`
            parser.advance_tokens(); // Advance past the `->`
            return_type = parser.parse_type()
        }

        if !parser.peek_check_variant_advance(&Token::LeftBrace) {
            panic!("Expected a Left Brace `{` to start the function block")
        };

        let body = parser.parse_block_statement();

        FunctionDefinition {
            name: func_name.into(),
            attribute : None,
            parameters,
            body,
            return_type,
        }
    }

    fn parse_fn_parameters(parser: &mut Parser) -> Vec<(Ident, Type)> {
        if parser.peek_token == Token::RightParen {
            parser.advance_tokens();
            return Vec::new();
        }

        parser.advance_tokens();

        let mut parameters: Vec<(Ident, Type)> = Vec::new();

        // next token should be identifier
        let name: Ident = parser.curr_token.token().to_string().into();
        parameters.push((name, FuncParser::parse_fn_type(parser)));

        while parser.peek_token == Token::Comma {
            parser.advance_tokens(); // curr_token = comma
            parser.advance_tokens(); // curr_token == identifier

            parameters.push((
                parser.curr_token.token().to_string().into(),
                FuncParser::parse_fn_type(parser),
            ));
        }

        if !parser.peek_check_variant_advance(&Token::RightParen) {
            panic!("Expected a Right Parenthesis `)` after comma")
        };

        parameters
    }

    fn parse_fn_type(parser: &mut Parser) -> Type {
        // We should have a colon in the next Token
        if !parser.peek_check_variant_advance(&Token::Colon) {
            panic!("Expected a Colon `:` after the parameter name")
        };

        // The current token is now a colon, lets advance the tokens again
        parser.advance_tokens();

        // We should now be on the Token which represents the Type
        parser.parse_type()
    }
}
