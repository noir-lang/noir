use super::*;
use crate::ast::UnaryOp;

pub struct UnaryParser;

impl UnaryParser {
    pub fn parse(parser: &mut Parser) -> ParserExprResult {
        let operator = UnaryOp::from(parser.curr_token.token());
        parser.advance_tokens();
        let rhs = parser.parse_expression(Precedence::Prefix).unwrap();
        Ok(Expression::Prefix(Box::new(PrefixExpression { operator, rhs })))
    }
}
