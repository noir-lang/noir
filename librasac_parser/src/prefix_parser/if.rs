use super::*;

pub struct IfParser;

impl PrefixParser for IfParser {
    fn parse(parser: &mut Parser) -> Expression {
        if !parser.peek_check_variant_advance(Token::LeftParen) {
            panic!("Expected a Left parenthesis")
        };
        parser.advance_tokens();
        let condition = parser.parse_expression(Precedence::Lowest).unwrap();

        if !parser.peek_check_variant_advance(Token::RightParen) {
            panic!("Expected a Right parenthesis")
        };

        if !parser.peek_check_variant_advance(Token::LeftBrace) {
            panic!("Expected a Left Brace")
        };
        let consequence = parser.parse_block_statement();

        let mut alternative: Option<BlockStatement> = None;
        if parser.peek_token == Token::Keyword(Keyword::Else) {
            parser.advance_tokens();

            if !parser.peek_check_variant_advance(Token::LeftBrace) {
                panic!("Expected a Left Brace")
            };

            alternative = Some(parser.parse_block_statement());
        }

        let if_expr = IfExpression {
            condition,
            consequence,
            alternative: alternative,
        };

        Expression::If(Box::new(if_expr))
    }
}
