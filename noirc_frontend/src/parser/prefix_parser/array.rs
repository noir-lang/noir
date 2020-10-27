use super::*;

pub struct ArrayParser;

impl ArrayParser {
    // Arrays are of the form [a, b, c,d]
    pub fn parse(parser: &mut Parser) -> ParserExprKindResult {
        // Current token is '['
        //
        // parse the contents of the array
        let elements = parser.parse_comma_separated_argument_list(Token::RightBracket)?;

        let array_len = elements.len() as u128;

        let expr = ExpressionKind::Literal(Literal::Array(ArrayLiteral {
            contents: elements,
            length: array_len,
            r#type: Type::Unknown, // XXX: Can't figure it out at the moment, but analyser should be able to and also it should be able to check contents to make sure they are all same type
        }));

        Ok(expr)
    }
}
