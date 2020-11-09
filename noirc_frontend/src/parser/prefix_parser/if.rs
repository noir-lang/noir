use super::*;

pub struct IfParser;

impl IfParser {
    pub fn parse(parser: &mut Parser) -> ParserExprKindResult { 
        
        parser.peek_check_variant_advance(&Token::LeftParen)?;
        
        parser.advance_tokens();
        let condition = parser.parse_expression(Precedence::Lowest)?;

        parser.peek_check_variant_advance(&Token::RightParen)?;

        parser.peek_check_variant_advance(&Token::LeftBrace)?;
        let consequence = parser.parse_block_statement()?;

        let mut alternative: Option<BlockStatement> = None;
        if parser.peek_token == Token::Keyword(Keyword::Else) {
            parser.advance_tokens();

            parser.peek_check_variant_advance(&Token::LeftBrace)?;

            alternative = Some(parser.parse_block_statement()?);
        }

        let if_expr = IfExpression {
            condition,
            consequence,
            alternative: alternative,
        };
        Ok(ExpressionKind::If(Box::new(if_expr)))
    }
}