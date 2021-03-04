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
            // `x as pub u8` is not supported. If we move away from `pub, priv and const` declarations
            // then it would make sense to have it.
            // It would then look like : `let z = x as pub u8`
            //
            // XXX: We could check for base types here, however in the future it would be nice
            // to be able to convert between array types.
            //
            // An example is that if we have [4]u32 we can use `as` to convert it to a [4]u8
            // let x = [a,b,c,d]
            // let z = x as [4]u8
            r#type: parser.parse_type(false)?,
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
