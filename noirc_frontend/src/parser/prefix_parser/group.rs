use super::*;

pub struct GroupParser;

impl GroupParser {
    /// The Group Parser is a precedent lifter.
    /// Cursor Start : `(`
    ///
    /// Cursor End : `)`
    pub fn parse(parser: &mut Parser) -> ParserExprKindResult {
        // Current token is `(`
        //
        // Bump cursor. The next token should be
        // the start of an expression
        parser.advance_tokens();

        // There is no explicit unit type representation
        // as an expression. So we do not check for `()`

        // Use the lowest precedence and parse the expression
        let exp = parser.parse_expression(Precedence::Lowest)?;

        // Once the expression is parsed, the next token should
        // be the `)`
        parser.peek_check_variant_advance(&Token::RightParen)?;

        Ok(exp.kind)
    }
}

#[cfg(test)]
mod test {
    use crate::parser::test_parse;

    use super::GroupParser;

    #[test]
    fn valid_syntax() {
        const SRC: &'static str = r#"
            (x+a)
        "#;
        /// Remember that although this may fail overall
        /// for the GroupParser, it is locally correct
        const SRC_DOUBLE_RPAREN: &'static str = r#"
            (x+a))
        "#;

        GroupParser::parse(&mut test_parse(SRC)).unwrap();
        GroupParser::parse(&mut test_parse(SRC_DOUBLE_RPAREN)).unwrap();
    }
    #[test]
    fn invalid_syntax() {
        const SRC_MISSING_RPAREN: &'static str = r#"
            (x+a
        "#;
        const SRC_DOUBLE_LPAREN: &'static str = r#"
            ((x+a)
        "#;

        const SRC_EMPTY_EXPR: &'static str = r#"
            ()
        "#;

        GroupParser::parse(&mut test_parse(SRC_MISSING_RPAREN)).unwrap_err();
        GroupParser::parse(&mut test_parse(SRC_DOUBLE_LPAREN)).unwrap_err();
        GroupParser::parse(&mut test_parse(SRC_EMPTY_EXPR)).unwrap_err();
    }
}
