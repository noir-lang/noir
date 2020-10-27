use super::*;

pub struct GroupParser;

impl GroupParser {
  pub fn parse(parser: &mut Parser) -> ParserExprKindResult {
        parser.advance_tokens();

        let exp = parser.parse_expression(Precedence::Lowest)?;
        parser.peek_check_variant_advance(&Token::RightParen)?;

        Ok(exp.kind)
    }
}
