use super::*;

pub struct NameParser;

impl NameParser {
    pub fn parse(parser: &mut Parser) -> ParserExprResult {
        let expr = match parser.curr_token.token() {
            Token::Ident(x) => Expression::Ident(x.clone()),
            _ => panic!("expected an identifier"),
        };

        Ok(expr)
    }
}
