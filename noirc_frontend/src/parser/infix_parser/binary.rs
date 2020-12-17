use super::*;
use noirc_errors::Spanned;
use crate::{lexer::token::SpannedToken};
pub struct BinaryParser;

impl BinaryParser {
    pub fn parse(parser: &mut Parser, lhs: Expression) -> ParserExprKindResult {
        let operator = token_to_binary_op(&parser.curr_token, parser.file_id)?;

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
fn token_to_binary_op(spanned_tok : &SpannedToken, file_id : usize) -> Result<BinaryOp, ParserError> {
    let bin_op_kind : Option<BinaryOpKind> = spanned_tok.token().into();
    let bin_op_kind = bin_op_kind.ok_or(ParserErrorKind::TokenNotBinaryOp{spanned_token : spanned_tok.clone()}.into_err(file_id))?;
    Ok(Spanned::from(spanned_tok.into_span(), bin_op_kind))
}