use super::*;
use crate::lexer::token::SpannedToken;
use noirc_errors::Spanned;
pub struct BinaryParser;

impl BinaryParser {
    /// Parses all expressions containing binary operations
    ///
    /// EXPR_LHS OP EXPR_RHS
    ///
    /// Cursor Start : `OP`
    ///
    /// Cursor End : `EXPR_RHS`
    pub fn parse(parser: &mut Parser, lhs: Expression) -> ParserExprKindResult {
        let operator = token_to_binary_op(&parser.curr_token, parser.file_id)?;

        // Check if the operator is a predicate
        // so that we can eagerly wrap it as a Predicate expression
        let is_predicate_op = operator.contents.is_comparator();

        // Parse rhs, precedence is extracted so that the
        // expression is grouped correctly
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
fn token_to_binary_op(
    spanned_tok: &SpannedToken,
    file_id: usize,
) -> Result<BinaryOp, ParserErrorKind> {
    let bin_op_kind: Option<BinaryOpKind> = spanned_tok.token().into();
    let bin_op_kind = bin_op_kind.ok_or(ParserErrorKind::TokenNotBinaryOp {
        spanned_token: spanned_tok.clone(),
    })?;
    Ok(Spanned::from(spanned_tok.into_span(), bin_op_kind))
}

#[cfg(test)]
mod test {

    use crate::{
        parser::{dummy_expr, test_parse},
        Expression,
    };

    use super::BinaryParser;

    #[test]
    fn valid_syntax() {
        let vectors = vec![" + 6", " - k", " + (x + a)", " * (x + a) + (x - 4)"];

        for src in vectors {
            let mut parser = test_parse(src);
            let _ = BinaryParser::parse(&mut parser, dummy_expr()).unwrap();
        }
    }

    #[test]
    fn start_end_cursor() {
        const SRC: &'static str = " + 6";

        let mut parser = test_parse(SRC);

        let start = parser.curr_token.clone();

        let _ = BinaryParser::parse(&mut parser, dummy_expr()).unwrap();

        let end = parser.curr_token.clone();

        assert_eq!(start, crate::token::Token::Plus);
        assert_eq!(end, crate::token::Token::Int(6.into()));
    }
    #[test]
    fn invalid_syntax() {
        let vectors = vec!["! x"];

        for src in vectors {
            let mut parser = test_parse(src);
            let _ = BinaryParser::parse(&mut parser, dummy_expr()).unwrap_err();
        }
    }
}
