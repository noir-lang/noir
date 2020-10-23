use super::*;

/// The LiteralParser specifies how we will parse all Literal Tokens
/// Except for function literals
pub struct LiteralParser;

impl LiteralParser {
    /// Parses a Literal token
    pub fn parse(parser: &mut Parser) -> ParserExprResult {
       let expr =  match parser.curr_token.clone().into() {
            Token::Int(x) => Expression::Literal(Literal::Integer(x)),
            Token::Str(x) => Expression::Literal(Literal::Str(x)),
            Token::Bool(x) => Expression::Literal(Literal::Bool(x)),
            Token::IntType(x) => Expression::Literal(Literal::Type(Type::from(&x))),
            x => panic!("expected a literal token, but found {}", x.to_string()),
        };
        Ok(expr)
    }
}
