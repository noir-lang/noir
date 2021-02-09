use super::*;

pub struct FuncParser;

impl FuncParser {
    /// Parses a function definition.
    ///
    /// fn IDENT(IDENT : Type,IDENT : Type,... ) (-> Type)? {
    ///         <STMT> <STMT> ...   
    /// }
    ///
    /// (->Type)? indicates that the return type is optional.
    /// If not return type is supplied, the return type is
    /// implied to be the unit type.
    ///
    /// Cursor Start : `fn`
    ///
    /// Cursor End : `}`
    /// All functions are function definitions. Functions are not first class citizens.
    pub(crate) fn parse_fn_definition(
        parser: &mut Parser,
        attribute: Option<Attribute>,
    ) -> Result<NoirFunction, ParserError> {
        // Current token is `fn`
        //
        // Peek ahead and check if the next token is an identifier
        parser.peek_check_kind_advance(TokenKind::Ident)?;
        let spanned_func_name: Ident = parser.curr_token.clone().into();

        // Current token is the function name
        //
        // Peek ahead and check if the next token is the `(`
        parser.peek_check_variant_advance(&Token::LeftParen)?;

        // Current token is `(`
        //
        // When parameters are successfully parsed, the current token will be
        // `)`
        let parameters = FuncParser::parse_fn_parameters(parser)?;

        // Parse the return type
        //
        let mut return_type = Type::Unit;
        if parser.peek_token == Token::Arrow {
            // Advance past the `)`
            parser.advance_tokens();
            // Advance past the `->`
            //
            // Note we do not need to `peek_check`
            // because of the if statement
            parser.advance_tokens();

            // Current token should now be
            // the start of the type
            return_type = parser.parse_type()?
        }

        parser.peek_check_variant_advance(&Token::LeftBrace)?;

        let start = parser.curr_token.into_span();
        let body = BlockParser::parse_block_expression(parser)?;
        let end = parser.curr_token.into_span();

        // The cursor position is inherited from the block expression
        // parsing procedure which is `}`

        // Currently, we only allow lowlevel, builtin and normal functions
        // In the future, we can add a test attribute.
        // Arbitrary attributes will not be supported.
        let func_def = FunctionDefinition {
            name: spanned_func_name.into(),
            attribute: attribute,
            parameters,
            body,
            span: start.merge(end),
            return_type,
        };

        Ok(func_def.into())
    }

    /// Cursor Start : `(`
    ///
    /// Cursor End : `)`
    fn parse_fn_parameters(parser: &mut Parser) -> Result<Vec<(Ident, Type)>, ParserError> {
        // Current token is `(`
        //
        // Check if we have an empty list
        if parser.peek_token == Token::RightParen {
            parser.advance_tokens();
            return Ok(Vec::new());
        }

        // Current token is still the `(`
        //
        // Since the list is non-empty. We advance to the first
        // parameter name
        parser.advance_tokens();

        let mut parameters: Vec<(Ident, Type)> = Vec::new();

        // Push the first parameter and it's type
        //
        // Notice that parsing the type requires that the
        // cursor starts on the parameter name, which is upheld
        let spanned_name: Ident = parser.curr_token.clone().into();
        parameters.push((spanned_name, FuncParser::parse_fn_type(parser)?));

        while parser.peek_token == Token::Comma {
            // Current token is Type
            //
            // Advance past the next token, which is a comma
            parser.advance_tokens();

            if (parser.curr_token == Token::Comma) && (parser.peek_token == Token::RightParen) {
                // Entering here means there is nothing else to parse;
                // the list has a trailing comma
                break;
            }

            parser.peek_check_kind_advance(TokenKind::Ident)?;

            parameters.push((
                parser.curr_token.clone().into(),
                FuncParser::parse_fn_type(parser)?,
            ));
        }

        parser.peek_check_variant_advance(&Token::RightParen)?;

        Ok(parameters)
    }
    /// Cursor Start : `IDENT`
    ///
    /// Cursor End : `TYPE`
    fn parse_fn_type(parser: &mut Parser) -> Result<Type, ParserError> {
        // Current token is `IDENT`
        //
        // Peek ahead and check if the next token is `:`
        parser.peek_check_variant_advance(&Token::Colon)?;

        // Current token is `:`
        //
        // Bum cursor. Next Token should be the Type
        parser.advance_tokens();

        parser.parse_type()
    }
}

#[cfg(test)]
mod test {
    use crate::parser::test_parse;

    use super::FuncParser;

    #[test]
    fn valid_syntax() {
        const SRC_RET_TYPE: &'static str = r#"
            fn func_name( f: u8, y : Public) -> u8 {
                x + a
            }
        "#;
        const SRC_NO_RET_TYPE: &'static str = r#"
            fn func_name( f: Witness, y : Public, z : [5]u8,)  {

            }
        "#;

        FuncParser::parse_fn_definition(&mut test_parse(SRC_RET_TYPE), None).unwrap();
        FuncParser::parse_fn_definition(&mut test_parse(SRC_NO_RET_TYPE), None).unwrap();
    }
    #[test]
    fn double_comma() {
        const SRC: &'static str = r#"
            fn x2( f: []Witness,,) {

            }
        "#;

        FuncParser::parse_fn_definition(&mut test_parse(SRC), None).unwrap_err();
    }
}
