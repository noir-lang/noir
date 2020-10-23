use super::*;

pub struct BinaryParser;

impl BinaryParser {
    pub fn parse(parser: &mut Parser, lhs: Expression) -> ParserExprResult {
        let operator: BinaryOp = parser.curr_token.token().into();

        let curr_precedence = Precedence::from(&parser.curr_token);
        parser.advance_tokens();

        let rhs = parser.parse_expression(curr_precedence).unwrap();
        let infix_expression = Box::new(InfixExpression {
            lhs: lhs,
            operator,
            rhs: rhs.clone(),
        });
        
        if operator.is_comparator() {
            return Ok(Expression::Predicate(infix_expression));
        }

        if operator != BinaryOp::As {
            return Ok(Expression::Infix(infix_expression));
        }


        match rhs.r#type() {
            Some(typ) => Ok(Expression::Cast(Box::new(CastExpression { lhs: infix_expression.lhs, r#type: typ }))),
            None => panic!("The operator being used is as, however the RHS is not a Type"),
        }
    }
}
