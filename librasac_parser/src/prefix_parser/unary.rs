use super::*;
use librasac_ast::UnaryOp;

pub struct UnaryParser;

impl UnaryParser {
    /// Converts a token to a unary operator
    /// If you want the parser to recognise another Token as being a prefix operator, it is defined here
    fn to_unary_op(token: &Token) -> UnaryOp {
        match token {
            Token::Minus => UnaryOp::Minus,
            Token::Bang => UnaryOp::Not,
            _ => panic!(
                "The token {} has not been linked to a unary operator",
                token
            ),
        }
    }
}

impl PrefixParser for UnaryParser {
    fn parse(parser: &mut Parser) -> Expression {
        let operator = UnaryParser::to_unary_op(&parser.curr_token);
        parser.advance_tokens();
        let rhs = parser.parse_expression(Precedence::Prefix).unwrap();
        Expression::Prefix(Box::new(PrefixExpression { operator, rhs }))
    }
}
