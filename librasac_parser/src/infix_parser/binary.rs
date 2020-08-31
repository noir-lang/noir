use super::*;

pub struct BinaryParser;

// XXX(low) : Check that these are the only possible predicate ops
// predicate operators are capable of returning a 0 or 1
const fn predicate_ops() -> [BinaryOp; 6] {
    [
        BinaryOp::Equal,
        BinaryOp::NotEqual,
        BinaryOp::LessEqual,
        BinaryOp::Less,
        BinaryOp::Greater,
        BinaryOp::GreaterEqual,
    ]
}

impl InfixParser for BinaryParser {
    fn parse(parser: &mut Parser, lhs: Expression) -> Expression {
        let operator: BinaryOp = parser.curr_token.clone().into();

        let curr_precedence = Precedence::from(&parser.curr_token);
        parser.advance_tokens();

        let rhs = parser.parse_expression(curr_precedence).unwrap();

        let infix_expression = Box::new(InfixExpression { lhs, operator, rhs });

        if predicate_ops().contains(&operator) {
            return Expression::Predicate(infix_expression);
        }
        return Expression::Infix(infix_expression);
    }
}
