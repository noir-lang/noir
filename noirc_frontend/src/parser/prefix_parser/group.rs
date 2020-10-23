use super::*;

pub struct GroupParser;

impl GroupParser {
  pub fn parse(parser: &mut Parser) -> ParserExprResult {
        parser.advance_tokens();

        let exp = parser.parse_expression(Precedence::Lowest);
        if !parser.peek_check_variant_advance(&Token::RightParen) {
            panic!(
                "Expected a right parentheses to end the expression, got {}",
                parser.peek_token.token()
            )
        }

        exp
    }
}
