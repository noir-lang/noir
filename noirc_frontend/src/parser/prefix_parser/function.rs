use super::*;

pub struct FuncParser;

impl FuncParser {
    /// All functions are function definitions. Functions are not first class.
    pub(crate) fn parse_fn_definition(
        parser: &mut Parser,
        attribute : Option<Attribute>,
    ) -> Result<FunctionDefinition, ParserError> {

        // Check if we have an identifier.
        parser.peek_check_kind_advance(TokenKind::Ident)?;
        let func_name = parser.curr_token.token().to_string();
        let spanned_func_name = Spanned::from(parser.curr_token.into_span(), func_name);
        
        parser.peek_check_variant_advance(&Token::LeftParen)?;
        let parameters = FuncParser::parse_fn_parameters(parser)?;
        // Parse the type after the parameters have been parsed
        let mut return_type = Type::Unit;
        if parser.peek_token == Token::Arrow {
            parser.advance_tokens(); // Advance past the `)`
            parser.advance_tokens(); // Advance past the `->`
            return_type = parser.parse_type()?
        }

        parser.peek_check_variant_advance(&Token::LeftBrace)?;

        let body = parser.parse_block_statement()?;

        let func_def = FunctionDefinition {
            name: spanned_func_name.into(),
            attribute : attribute,
            parameters,
            body,
            return_type,
        };
        Ok(func_def)
    }

    fn parse_fn_parameters(parser: &mut Parser) -> Result<Vec<(Ident, Type)>, ParserError> {
        if parser.peek_token == Token::RightParen {
            parser.advance_tokens();
            return Ok(Vec::new());
        }

        parser.advance_tokens(); // Advance past the left parenthesis

        let mut parameters: Vec<(Ident, Type)> = Vec::new();

        // next token should be identifier
        let spanned_name : Ident = parser.curr_token.clone().into();
        parameters.push((spanned_name, FuncParser::parse_fn_type(parser)?));
        
        while parser.peek_token == Token::Comma {
            parser.advance_tokens(); // curr_token = comma
            parser.advance_tokens(); // curr_token == identifier

            parameters.push((
                parser.curr_token.clone().into(),
                FuncParser::parse_fn_type(parser)?,
            ));
        }

        parser.peek_check_variant_advance(&Token::RightParen)?;

        Ok(parameters)
    }

    fn parse_fn_type(parser: &mut Parser) -> Result<Type, ParserError> {
        // We should have a colon in the next Token
        parser.peek_check_variant_advance(&Token::Colon)?;

        // The current token is now a colon, lets advance the tokens again
        parser.advance_tokens();

        // We should now be on the Token which represents the Type
        parser.parse_type()
    }
}
