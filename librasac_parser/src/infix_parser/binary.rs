use super::*;

pub struct BinaryParser;

impl InfixParser for BinaryParser {
    fn parse(parser: &mut Parser, lhs: Expression) -> Expression {
        let operator: BinaryOp = parser.curr_token.clone().into();

        let curr_precedence = Precedence::from(&parser.curr_token);
        parser.advance_tokens();

        let rhs = parser.parse_expression(curr_precedence).unwrap();

        Expression::Infix(Box::new(InfixExpression { lhs, operator, rhs }))
    }
}
