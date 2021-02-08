use super::*;

pub struct ArrayParser;

impl ArrayParser {
    /// Parses Arrays of the form
    /// - [<EXPR>, <EXPR>, <EXPR>, <EXPR>]
    ///
    /// The last expression can end with a comma, before the closing delimiter
    ///
    /// Cursor Start : `[`
    ///
    /// Cursor End : `]`
    pub fn parse(parser: &mut Parser) -> ParserExprKindResult {
        // Current token is '['
        //
        // parse the contents of the array
        let elements = parser.parse_comma_separated_argument_list(Token::RightBracket)?;

        let array_len = elements.len() as u128;

        let expr = ExpressionKind::Literal(Literal::Array(ArrayLiteral {
            contents: elements,
            length: array_len,
            r#type: Type::Unknown,
        }));

        Ok(expr)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        parser::{errors::ParserErrorKind, test_parse},
        token::Token,
        ArrayLiteral, ExpressionKind, Literal, Type,
    };

    use super::ArrayParser;

    fn expr_to_array(expr: ExpressionKind) -> ArrayLiteral {
        let lit = match expr {
            ExpressionKind::Literal(literal) => literal,
            _ => unreachable!("expected a literal"),
        };

        match lit {
            Literal::Array(arr) => arr,
            _ => unreachable!("expected an array"),
        }
    }

    /// This is the standard way to declare an array
    #[test]
    fn valid_syntax() {
        const SRC: &'static str = r#"
            [0,1,2,3,4]
        "#;

        let mut parser = test_parse(SRC);

        let start = parser.curr_token.clone();

        let expr = ArrayParser::parse(&mut parser).unwrap();

        let end = parser.curr_token.clone();

        // First check that the cursor was in the right position at
        // the start and at the end
        assert_eq!(start.token(), &Token::LeftBracket);
        assert_eq!(end.token(), &Token::RightBracket);

        // Second, Check length and type
        let arr_lit = expr_to_array(expr);
        assert_eq!(arr_lit.length, 5);

        // All array types are unknown at parse time
        // This makes parsing simpler. The type checker
        // needs to iterate the whole array to ensure homogeneity
        // so there is no advantage to deducing the type here.
        assert_eq!(arr_lit.r#type, Type::Unknown);
    }

    #[test]
    fn valid_syntax_extra_comma() {
        // This is a valid user error. We return an unexpected token error.
        // We expect a `]` but instead we get an EOF
        const MISSING_END: &'static str = r#"
            [0,1,2,3,4,]
        "#;

        ArrayParser::parse(&mut test_parse(MISSING_END)).unwrap();
    }

    #[test]
    fn missing_starting_bracket() {
        // Since this is a prefix parser, this should be impossible to arrive at
        // unless there is an off-by-one error. The exact error here is not important,
        // since arriving here would signal an ICE
        const MISSING_START: &'static str = r#"
            0,1,2,3,4]
        "#;
        ArrayParser::parse(&mut test_parse(MISSING_START)).unwrap_err();
    }
    #[test]
    fn double_prefix() {
        const DOUBLE_PREFIX: &'static str = r#"
            [[0,1,2,3,4]
        "#;
        ArrayParser::parse(&mut test_parse(DOUBLE_PREFIX)).unwrap_err();
    }
    #[test]
    fn double_delimiter() {
        const DOUBLE_DELIMITER: &'static str = r#"
            [0,1,2,,]
        "#;
        ArrayParser::parse(&mut test_parse(DOUBLE_DELIMITER)).unwrap_err();
    }
    #[test]
    fn not_a_bug_double_postfix() {
        /// The following is invalid, however it is not a bug for the ArrayParser
        /// Note that the ArrayParser parses the first array which is [0,1,2,3,4]
        /// Then whatever comes after that is no longer the responsibility of the
        /// Array Parser
        const DOUBLE_POSTFIX: &'static str = r#"
            [0,1,2,3,4]]
        "#;
        ArrayParser::parse(&mut test_parse(DOUBLE_POSTFIX)).unwrap();
    }

    #[test]
    fn missing_closing_bracket() {
        // This is a valid user error. We return an unexpected token error.
        // We expect a `]` but instead we get an EOF
        const MISSING_END: &'static str = r#"
            [0,1,2,3,4
        "#;

        let err = ArrayParser::parse(&mut test_parse(MISSING_END)).unwrap_err();
        match err.kind {
            ParserErrorKind::UnexpectedToken {
                span: _,
                expected,
                found,
            } => {
                assert_eq!(expected, Token::RightBracket);
                assert_eq!(found, Token::EOF)
            }
            _ => unreachable!("expected an unexpected token error"),
        }
    }
}
