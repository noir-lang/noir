use super::*;

pub struct ForParser;

impl ForParser {
    pub fn parse(parser: &mut Parser) -> ParserExprKindResult { 
        parser.peek_check_kind_advance(TokenKind::Ident)?; 
        let spanned_identifier : Ident = parser.curr_token.clone().into();

        parser.peek_check_variant_advance(&Token::Keyword(Keyword::In))?;
        parser.advance_tokens(); // in

        // Parse range
        let start_range = parser.parse_expression(Precedence::Lowest)?;
        parser.peek_check_variant_advance(&Token::DoubleDot)?;
        parser.advance_tokens(); // ..
        let end_range = parser.parse_expression(Precedence::Lowest)?;
        
        parser.peek_check_variant_advance(&Token::LeftBrace)?;

        // Parse body
        let block = BlockParser::parse_block_expression(parser)?;

        let for_expr = ForExpression {
            identifier: spanned_identifier,
            start_range,
            end_range,
            block,
        };

        Ok(ExpressionKind::For(Box::new(for_expr)))

    }
}

