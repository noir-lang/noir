use crate::parser::errors::ParserErrorKind;

use super::*;

pub struct CallParser;

impl CallParser {
    pub fn parse(parser: &mut Parser, func_name: Expression) -> ParserExprKindResult {
        let arguments = parser.parse_comma_separated_argument_list(Token::RightParen)?;

        let func_name_string = match func_name.kind {
            ExpressionKind::Ident(x) => x,
            _ => return Err(ParserErrorKind::UnstructuredError{message: format!("Expected an identifier for the function name"), span : func_name.span}.into_err(parser.file_id))
        };

        let call_expr = CallExpression {
            func_name: func_name_string.into(),
            arguments,
        };

       Ok( ExpressionKind::Call(NoirPath::Current, Box::new(call_expr)))
    }
}
