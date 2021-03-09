use super::*;

pub struct LiteralParser;

impl LiteralParser {
    /// Cursor Start : `LITERAL`
    ///
    /// Cursor End : `LITERAL`
    pub fn parse(parser: &mut Parser) -> ParserExprKindResult {
        let expr = match parser.curr_token.clone().into() {
            Token::Int(x) => ExpressionKind::Literal(Literal::Integer(x)),
            Token::Str(x) => ExpressionKind::Literal(Literal::Str(x)),
            Token::Bool(x) => ExpressionKind::Literal(Literal::Bool(x)),
            x => {
                return Err(ParserErrorKind::UnexpectedTokenKind {
                    span: parser.curr_token.into_span(),
                    expected: TokenKind::Literal,
                    found: x.kind(),
                })
            }
        };
        Ok(expr)
    }
}

#[cfg(test)]
mod test {

    use crate::{parser::test_parse, ExpressionKind, Literal};

    use super::LiteralParser;

    fn expr_to_lit(expr: ExpressionKind) -> Literal {
        match expr {
            ExpressionKind::Literal(literal) => literal,
            _ => unreachable!("expected a literal"),
        }
    }
    #[test]
    fn valid_syntax_int() {
        const SRC_INT: &'static str = r#"
            5
        "#;
        const SRC_HEX: &'static str = r#"
            0x05
        "#;

        let expr_int = LiteralParser::parse(&mut test_parse(SRC_INT)).unwrap();
        let expr_hex = LiteralParser::parse(&mut test_parse(SRC_HEX)).unwrap();

        let int = match expr_to_lit(expr_int) {
            Literal::Integer(int) => int,
            _ => unreachable!(),
        };
        let hex = match expr_to_lit(expr_hex) {
            Literal::Integer(int) => int,
            _ => unreachable!(),
        };
        assert_eq!(hex, int)
    }

    #[test]
    fn valid_syntax_str() {
        const SRC: &'static str = r#"
            "hello"
        "#;

        let expr = LiteralParser::parse(&mut test_parse(SRC)).unwrap();
        let string = match expr_to_lit(expr) {
            Literal::Str(x) => x,
            _ => unreachable!(),
        };
        assert_eq!(string, "hello")
    }
    #[test]
    fn valid_syntax_bool() {
        const SRC_TRUE: &'static str = r#"
            true
        "#;
        const SRC_FALSE: &'static str = r#"
            false
        "#;

        let expr_true = LiteralParser::parse(&mut test_parse(SRC_TRUE)).unwrap();
        let expr_false = LiteralParser::parse(&mut test_parse(SRC_FALSE)).unwrap();
        let bool_t = match expr_to_lit(expr_true) {
            Literal::Bool(x) => x,
            _ => unreachable!(),
        };
        let bool_f = match expr_to_lit(expr_false) {
            Literal::Bool(x) => x,
            _ => unreachable!(),
        };
        assert_eq!(bool_t, true);
        assert_eq!(bool_f, false)
    }
}
