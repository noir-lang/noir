use super::*;

pub struct FuncParser;

impl FuncParser {
    pub(crate) fn parse_fn_decl(parser: &mut Parser) -> FunctionDefinition {
        let (f_dec, f_lit) = FuncParser::parse_fn(parser);

        let f_dec = match (f_dec, f_lit) {
            (_, Some(_)) => panic!("Unexpected function literal"),
            (None, _) => panic!("Expected function declaration"),
            (Some(f_dec), _) => f_dec,
        };

        f_dec
    }

    pub(crate) fn parse_fn_literal(parser: &mut Parser) -> Expression {
        let (f_dec, f_lit) = FuncParser::parse_fn(parser);

        let f_lit = match (f_dec, f_lit) {
            (Some(_), _) => panic!("Unexpected function declaration"),
            (_, None) => panic!("Expected a function literal"),
            (_, Some(f_lit)) => f_lit,
        };

        Expression::Literal(Literal::Func(f_lit))
    }

    /// It is either a function definition or a function literal.
    /// a function literal will not have a name attached to it, it is essentially always used as a closure
    pub(crate) fn parse_fn(
        parser: &mut Parser,
    ) -> (Option<FunctionDefinition>, Option<FunctionLiteral>) {
        // Check if we have an identifier.
        let func_name = if parser.peek_check_kind_advance(TokenKind::Ident) {
            Some(parser.curr_token.to_string())
        } else {
            None
        };

        if !parser.peek_check_variant_advance(Token::LeftParen) {
            panic!(
                "Expected a Left parenthesis after fn keyword or after identifier, found {}",
                parser.curr_token
            )
        };
        let parameters = FuncParser::parse_fn_parameters(parser);

        if !parser.peek_check_variant_advance(Token::LeftBrace) {
            panic!("Expected a Left Brace `{` after fn parameters")
        };

        let body = parser.parse_block_statement();

        let func_lit = FunctionLiteral {
            parameters: parameters,
            body: body,
        };

        // If a function name was supplied then this is a function definition
        // If not, then it is a function literal
        match func_name {
            Some(f_name) => {
                let func_dec = FunctionDefinition {
                    name: f_name.into(),
                    func: func_lit,
                };
                (Some(func_dec), None)
            }
            None => (None, Some(func_lit)),
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
        let name: Ident = parser.curr_token.to_string().into();
        parameters.push((name, FuncParser::parse_fn_type(parser)));

        while parser.peek_token == Token::Comma {
            parser.advance_tokens(); // curr_token = comma
            parser.advance_tokens(); // curr_token == identifier

            parameters.push((
                parser.curr_token.to_string().into(),
                FuncParser::parse_fn_type(parser),
            ));
        }

        if !parser.peek_check_variant_advance(Token::RightParen) {
            panic!("Expected a Right Parenthesis `)` after comma")
        };

        parameters
    }

    fn parse_fn_type(parser: &mut Parser) -> Type {
        // We should have a colon in the next Token
        if !parser.peek_check_variant_advance(Token::Colon) {
            panic!("Expected a Colon `:` after the parameter name")
        };

        // The current token is now a colon, lets advance the tokens again
        parser.advance_tokens();

        // We should now be on the Token which represents the Type
        // XXX: Currently we only Accept the basic types
        match &parser.curr_token {
            Token::Keyword(Keyword::Constant) => Type::Constant,
            Token::Keyword(Keyword::Witness) => Type::Witness,
            Token::Keyword(Keyword::Public) => Type::Public,
            Token::Keyword(Keyword::Field) => Type::FieldElement,
            k => panic!(
                "Currently, we only accept types that are Constant, Witness or Public. Got {}",
                k.clone()
            ),
        }
    }
}
