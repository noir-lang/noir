use super::*;
use crate::ast::UnaryOp;

pub struct UnaryParser;

impl PrefixParser for UnaryParser {
    fn parse(parser: &mut Parser) -> Expression {
        let operator = UnaryOp::from(parser.curr_token.token());
        parser.advance_tokens();
        let rhs = parser.parse_expression(Precedence::Prefix).unwrap();
        Expression::Prefix(Box::new(PrefixExpression { operator, rhs }))
    }
}
