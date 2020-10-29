use super::*;

pub struct BinaryParser;

impl BinaryParser {
    pub fn parse(parser: &mut Parser, lhs: Expression) -> ParserExprKindResult {
        let operator: BinaryOp = (&parser.curr_token).into();

        let is_predicate_op = operator.contents.is_comparator();

        let curr_precedence = Precedence::from(&parser.curr_token);
        parser.advance_tokens();

        let rhs = parser.parse_expression(curr_precedence)?;

        let infix_expression = Box::new(InfixExpression {
            lhs: lhs,
            operator,
            rhs: rhs.clone(),
        });
        
        if is_predicate_op {
            return Ok(ExpressionKind::Predicate(infix_expression));
        }
        return Ok(ExpressionKind::Infix(infix_expression));
    }
}
