use super::*;

pub struct BinaryParser;

impl InfixParser for BinaryParser {
    fn parse(parser: &mut Parser, lhs: Expression) -> Expression {
        let operator: BinaryOp = parser.curr_token.clone().into();

        let curr_precedence = Precedence::from(&parser.curr_token);
        parser.advance_tokens();

        let rhs = parser.parse_expression(curr_precedence).unwrap();
        let infix_expression = Box::new(InfixExpression {
            lhs: lhs.clone(),
            operator,
            rhs: rhs.clone(),
        });

        if operator.is_comparator() {
            return Expression::Predicate(infix_expression);
        }

        if operator != BinaryOp::As {
            return Expression::Infix(infix_expression);
        }

        match rhs.r#type() {
            Some(typ) => Expression::Cast(Box::new(CastExpression { lhs, r#type: typ })),
            None => panic!("The operator being used is as, however the RHS is not a Type"),
        }
    }
}
