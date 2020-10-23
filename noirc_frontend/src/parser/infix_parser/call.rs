use super::*;

pub struct CallParser;

impl CallParser {
    pub fn parse(parser: &mut Parser, func_name: Expression) -> ParserExprResult {
        let arguments = parser.parse_comma_separated_argument_list(Token::RightParen);

        let func_name_string = match func_name {
            Expression::Ident(x) => x,
            _ => unimplemented!("function name expression should only be an identifier"),
        };

        let call_expr = CallExpression {
            func_name: func_name_string.into(),
            arguments,
        };

       Ok( Expression::Call(NoirPath::Current, Box::new(call_expr)))
    }
}
