use super::*;

pub struct GroupParser;

impl PrefixParser for GroupParser {
    fn parse(parser: &mut Parser) -> Expression {
        parser.advance_tokens();

        let exp = parser.parse_expression(Precedence::Lowest);
        if !parser.peek_check_variant_advance(Token::RightParen) {
            panic!("Expected a right parentheses to end the expression")
        }

        exp.unwrap()
    }
}
