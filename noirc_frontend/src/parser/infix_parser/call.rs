use crate::{parser::{errors::ParserErrorKind}};

use super::*;

pub struct CallParser;

impl CallParser {
    pub fn parse(parser: &mut Parser, func_path: Expression) -> ParserExprKindResult {
        let arguments = parser.parse_comma_separated_argument_list(Token::RightParen)?;

        let func_path = match func_path.kind {
            ExpressionKind::Path(path) => path,
            _ => return Err(ParserErrorKind::UnstructuredError{message: format!("expected a path for the function name"), span : func_path.span}.into_err(parser.file_id))
        };

        let call_expr = CallExpression {
            func_name: func_path,
            arguments,
        };

       Ok(ExpressionKind::Call(Box::new(call_expr)))
    }
}