use super::*;
use crate::ast::UnaryOp;

pub struct UnaryParser;

impl UnaryParser {
    /// Parses a unary expression of the form
    ///
    /// OP EXPR
    ///
    /// Cursor Start : `OP`
    ///
    /// Cursor End : `EXPR`
    pub fn parse(parser: &mut Parser) -> ParserExprKindResult {
        let operator =
            UnaryOp::from(parser.curr_token.token()).ok_or(ParserErrorKind::TokenNotUnaryOp {
                spanned_token: parser.curr_token.clone(),
            })?;

        // Advance past the unary op
        //
        parser.advance_tokens();

        let rhs = parser.parse_expression(Precedence::Prefix)?;

        let kind = ExpressionKind::Prefix(Box::new(PrefixExpression { operator, rhs }));
        Ok(kind)
    }
}

#[cfg(test)]
mod test {
    use crate::parser::test_parse;

    use super::UnaryParser;

    #[test]
    fn valid_syntax() {
        let vectors = vec!["!hello", "-hello", "--hello", "-!hello", "!-hello"];

        for src in vectors {
            UnaryParser::parse(&mut test_parse(src)).unwrap();
        }
    }
    #[test]
    fn invalid_syntax() {
        let vectors = vec!["+hello", "/hello", "hello"];

        for src in vectors {
            UnaryParser::parse(&mut test_parse(src)).unwrap_err();
        }
    }
}
