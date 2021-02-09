use super::*;

pub struct IfParser;

impl IfParser {
    /// Parses if statements of the form:
    ///
    /// if (EXPR) BLOCK_EXPR (ELSE BLOCK_EXPR)?
    ///
    ///
    /// Cursor Start : `if`
    ///
    /// Cursor End : `}`
    pub fn parse(parser: &mut Parser) -> ParserExprKindResult {
        // Current token is `if`
        //
        // Bump cursor.
        parser.advance_tokens();

        // Current token is the start of the expression (condition)
        //
        let condition = parser.parse_expression(Precedence::Lowest)?;

        // Current token is `)`
        //
        // Peek ahead and check if the next token is `{`
        parser.peek_check_variant_advance(&Token::LeftBrace)?;

        // Current token is `{`
        // Which is the correct condition to call the block expression
        // parser as a sub procedure.
        let consequence = BlockParser::parse_block_expression(parser)?;

        // Parse the optional else condition
        let mut alternative: Option<BlockExpression> = None;
        if parser.peek_token == Token::Keyword(Keyword::Else) {
            // Current token is a `}`
            //
            // Bump Cursor to the `else`
            parser.advance_tokens();

            // Current token is `else`
            //
            // Peek ahead and check if the next token is `{`
            parser.peek_check_variant_advance(&Token::LeftBrace)?;

            alternative = Some(BlockParser::parse_block_expression(parser)?);
        }

        // The cursor position is inherited from the block expression
        // parsing procedure which is `}`

        let if_expr = IfExpression {
            condition,
            consequence,
            alternative,
        };
        Ok(ExpressionKind::If(Box::new(if_expr)))
    }
}

#[cfg(test)]
mod test {
    use crate::parser::test_parse;

    use super::IfParser;

    #[test]
    fn valid_syntax() {
        const SRC: &'static str = r#"
            if x + a {

            } else {

            }
        "#;
        const SRC_NO_ELSE: &'static str = r#"
            if x {

            }
        "#;

        IfParser::parse(&mut test_parse(SRC)).unwrap();
        IfParser::parse(&mut test_parse(SRC_NO_ELSE)).unwrap();
    }
    #[test]
    fn no_else_brace() {
        const SRC: &'static str = r#"
            if (x / a) + 1 {

            } else
        "#;

        IfParser::parse(&mut test_parse(SRC)).unwrap_err();
    }
}
