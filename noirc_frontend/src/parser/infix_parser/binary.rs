use super::*;

pub struct BinaryParser;

impl InfixParser for BinaryParser {
    fn parse(parser: &mut Parser, lhs: Expression) -> Expression {
        let operator: BinaryOp = parser.curr_token.clone().into();

        let curr_precedence = Precedence::from(&parser.curr_token);
        parser.advance_tokens();

        let rhs = parser.parse_expression(curr_precedence).unwrap();
        let infix_expression = Box::new(InfixExpression {
            lhs: lhs,
            operator,
            rhs: rhs.clone(),
        });
        
        if operator == BinaryOp::Assign {
            let identifier = infix_expression.lhs.identifier().expect("Expected the LHs of an assign operation to be an identifier");
            
            let assign_expr = AssignExpression{
                identifier,
                rhs
            };
            return Expression::Assign(Box::new(assign_expr))
        }

        if operator.is_comparator() {
            return Expression::Predicate(infix_expression);
        }

        if operator != BinaryOp::As {
            return Expression::Infix(infix_expression);
        }


        match rhs.r#type() {
            Some(typ) => Expression::Cast(Box::new(CastExpression { lhs: infix_expression.lhs, r#type: typ })),
            None => panic!("The operator being used is as, however the RHS is not a Type"),
        }
    }
}
