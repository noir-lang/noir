use super::*;

pub struct CallParser;

impl InfixParser for CallParser {
    fn parse(parser: &mut Parser, func_name: Expression) -> Expression {
        let arguments = CallParser::parse_call_arguments(parser);

        let func_name_string = match func_name {
            Expression::Ident(x) => x,
            _ => unimplemented!("function name expression should only be an identifier"),
        };

        let call_expr = CallExpression {
            func_name: func_name_string.into(),
            arguments,
        };

        Expression::Call(Box::new(call_expr))
    }
}
impl CallParser {
    fn parse_call_arguments(parser: &mut Parser) -> Vec<Expression> {
        if parser.peek_token == Token::RightParen {
            parser.advance_tokens();
            return Vec::new();
        }
        let mut arguments: Vec<Expression> = Vec::new();

        parser.advance_tokens();
        arguments.push(parser.parse_expression(Precedence::Lowest).unwrap());
        while parser.peek_token == Token::Comma {
            parser.advance_tokens();
            parser.advance_tokens();

            arguments.push(parser.parse_expression(Precedence::Lowest).unwrap());
        }

        if !parser.peek_check_variant_advance(Token::RightParen) {
            panic!("Expected a Right Parenthesis")
        };

        parser.advance_tokens(); // Skip the ')' parenthesis

        arguments
    }
}
