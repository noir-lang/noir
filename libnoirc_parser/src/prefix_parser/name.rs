use super::*;

pub struct NameParser;

impl PrefixParser for NameParser {
    fn parse(parser: &mut Parser) -> Expression {
        match &parser.curr_token {
            Token::Ident(x) => Expression::Ident(x.clone()),
            _ => panic!("expected an identifier"),
        }
    }
}
