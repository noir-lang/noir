use super::*;

pub struct CastParser;

impl CastParser {
    /// Parses a Cast Expression of the form:
    ///
    /// EXPR as TYPE
    ///
    /// Cursor Start : `as`
    ///
    /// Cursor End : `TYPE`
    pub fn parse(parser: &mut Parser, lhs: Expression) -> ParserExprKindResult {
        // Current Token is `as`
        //
        // Bump Cursor. Next token should be type
        parser.advance_tokens();

        Ok(ExpressionKind::Cast(Box::new(CastExpression {
            lhs,
            r#type: parser.parse_type()?,
        })))
    }
}

#[cfg(test)]
mod test {

    use super::CastParser;
    use crate::parser::{dummy_expr, test_parse};

    #[test]
    fn valid_syntax() {
        let vectors = vec![
            " as u8", //
            " as Witness",
            " as [8]Witness",
        ];

        for src in vectors {
            let mut parser = test_parse(src);

            let start = parser.curr_token.clone();

            let _ = CastParser::parse(&mut parser, dummy_expr()).unwrap();

            assert_eq!(
                start,
                crate::token::Token::Keyword(crate::token::Keyword::As)
            );
        }
    }
}
