use super::*;

pub struct BinaryParser;

impl BinaryParser {
    pub fn parse(parser: &mut Parser, lhs: Expression) -> ParserExprKindResult {
        let operator: BinaryOp = parser.curr_token.token().into();

        let curr_precedence = Precedence::from(&parser.curr_token);
        parser.advance_tokens();

        let rhs = parser.parse_expression(curr_precedence)?;
        let infix_expression = Box::new(InfixExpression {
            lhs: lhs,
            operator,
            rhs: rhs.clone(),
        });
        
        if operator.is_comparator() {
            return Ok(ExpressionKind::Predicate(infix_expression));
        }

        if operator != BinaryOp::As {
            return Ok(ExpressionKind::Infix(infix_expression));
        }


        match rhs.kind.r#type() {
            Some(typ) => Ok(ExpressionKind::Cast(Box::new(CastExpression { lhs: infix_expression.lhs, r#type: typ }))),
            None => {
                return Err(ParserError::UnstructuredError{message: format!("The operator being used is as. Expected a Type"), span : rhs.span});
            }
        }
    }
}
