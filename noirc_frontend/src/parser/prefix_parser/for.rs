use super::*;

pub struct ForParser;

impl ForParser {
    pub fn parse(parser: &mut Parser) -> ParserExprResult { 
        if !parser.peek_check_kind_advance(TokenKind::Ident) {
            panic!("Expected an identifier after the For keyword")
        }   

        let identifier = parser.curr_token.token().to_string();

        if !parser.peek_check_variant_advance(&Token::Keyword(Keyword::In)) {
            panic!("Expected the keyword `In` after the Identifier {}", &identifier)
        }
        parser.advance_tokens(); // in
        
        // Parse range
        let start_range = parser.parse_expression(Precedence::Lowest).unwrap();
        if !parser.peek_check_variant_advance(&Token::DoubleDot) {
            panic!("Expected a `..` after the start range in for loop ")
        }
        parser.advance_tokens(); // ..
        let end_range = parser.parse_expression(Precedence::Lowest).unwrap();
        
        if !parser.peek_check_variant_advance(&Token::LeftBrace) {
            panic!("Expected a Left Brace after the range statement")
        };

        // Parse body
        let block = parser.parse_block_statement()?;

        let for_expr = ForExpression {
            identifier: identifier.into(),
            start_range,
            end_range,
            block,
        };

        Ok(Expression::For(Box::new(for_expr)))

    }
}

