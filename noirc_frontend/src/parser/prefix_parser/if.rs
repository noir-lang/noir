use super::*;

pub struct IfParser;

impl IfParser {
    pub fn parse(parser: &mut Parser) -> ParserExprKindResult { 
        
        parser.peek_check_variant_advance(&Token::LeftParen)?;
        
        parser.advance_tokens();
        let condition = parser.parse_expression(Precedence::Lowest)?;

        parser.peek_check_variant_advance(&Token::RightParen)?;

        parser.peek_check_variant_advance(&Token::LeftBrace)?;
        let consequence = BlockParser::parse_block_expression(parser)?;

        let mut alternative: Option<BlockExpression> = None;
        if parser.peek_token == Token::Keyword(Keyword::Else) {
            parser.advance_tokens();

            parser.peek_check_variant_advance(&Token::LeftBrace)?;
            
            alternative = Some(BlockParser::parse_block_expression(parser)?);
        }

        let if_expr = IfExpression {
            condition,
            consequence,
            alternative: alternative,
        };
        Ok(ExpressionKind::If(Box::new(if_expr)))
    }
}