use super::*;

pub struct ForParser;

impl ForParser {
    /// Parses a for expression.
    ///
    /// ```noir
    /// for IDENT in RANGE_START..RANGE_END {
    ///  <STMT> <STMT> <STMT> ...  
    /// }
    ///```
    ///
    /// Cursor Start : `for`
    /// 
    /// Cursor End : `}` 
    pub fn parse(parser: &mut Parser) -> ParserExprKindResult {
        // Current token is the `for`
        //
        // Peek ahead and check if the next token is an identifier 
        parser.peek_check_kind_advance(TokenKind::Ident)?; 
        let spanned_identifier : Ident = parser.curr_token.clone().into();

        // Current token is the loop identifier
        //
        // Peek ahead and check if the next token is the `in` keyword
        parser.peek_check_variant_advance(&Token::Keyword(Keyword::In))?;
        
        // Current token is now the `in` keyword
        //
        // Advance past the `in` keyword
        parser.advance_tokens();

        // Current token should now be the
        // token that starts the expression for RANGE_START
        let start_range = parser.parse_expression(Precedence::Lowest)?;

        // Current token is now the end of RANGE_START
        //
        // Peek ahead and check if the next token is `..`
        parser.peek_check_variant_advance(&Token::DoubleDot)?;
        
        // Current Token is the `..`
        //
        //  Advance past the `..`
        parser.advance_tokens();

        // Current token should now be the token that starts the expression
        // for RANGE_END
        let end_range = parser.parse_expression(Precedence::Lowest)?;
        
        // Current token is now the end of RANGE_END
        //
        // Peek ahead and check if the next token is `{`
        parser.peek_check_variant_advance(&Token::LeftBrace)?;

        // Parse the for loop body
        //
        // Current token is the `{`
        // This is the correct cursor position to call `parse_block_expression`
        let block = BlockParser::parse_block_expression(parser)?;

        // The cursor position is inherited from the block expression
        // parsing procedure which is `}`

        let for_expr = ForExpression {
            identifier: spanned_identifier,
            start_range,
            end_range,
            block,
        };

        Ok(ExpressionKind::For(Box::new(for_expr)))

    }
}


#[cfg(test)]
mod test {
    use crate::parser::test_parse;

    use super::ForParser;

    #[test]
    fn valid_syntax() {

        /// Why is this allowed?
        ///
        /// The Parser does not check the types of the loops,
        /// it only checks for valid expressions in RANGE_START and
        /// RANGE_END
        const SRC_EXPR_LOOP : &'static str = r#"
            for i in x+y..z {

            }
        "#;
        const SRC_CONST_LOOP : &'static str = r#"
            for i in 0..100 {

            }
        "#;
    
        ForParser::parse(&mut test_parse(SRC_EXPR_LOOP)).unwrap();
        ForParser::parse(&mut test_parse(SRC_CONST_LOOP)).unwrap();
    }

    #[test]
    fn invalid_syntax() {

        /// Cannot have a literal as the loop identifier
        const SRC_LITERAL_IDENT : &'static str = r#"
            for 1 in x+y..z {

            }
        "#;
        /// Currently only the DoubleDot is supported
        const SRC_INCLUSIVE_LOOP : &'static str = r#"
            for i in 0...100 {

            }
        "#;
        /// Currently only the DoubleDot is supported
        const SRC_INCLUSIVE_EQUAL_LOOP : &'static str = r#"
            for i in 0..=100 {

            }
        "#;
    
        ForParser::parse(&mut test_parse(SRC_LITERAL_IDENT)).unwrap_err();
        ForParser::parse(&mut test_parse(SRC_INCLUSIVE_LOOP)).unwrap_err();
        ForParser::parse(&mut test_parse(SRC_INCLUSIVE_EQUAL_LOOP)).unwrap_err();
    }

}