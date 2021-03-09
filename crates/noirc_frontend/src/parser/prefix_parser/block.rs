use super::*;

pub struct BlockParser;

impl BlockParser {
    /// Parses Blocks of the form
    /// - {<STMT> <STMT> <STMT> <STMT> <STMT> ...}
    ///
    /// Cursor Start : `{`
    ///
    /// Cursor End : `}`
    pub fn parse(parser: &mut Parser) -> ParserExprKindResult {
        let block_expr = BlockParser::parse_block_expression(parser)?;

        Ok(ExpressionKind::Block(block_expr))
    }
    /// Parses a Block, returns a BlockExpression.
    /// This is useful for when the AST Node, will always contain a
    /// BlockExpression, hence parsing it through the BlockParser
    /// which returns an ExpressionKind is redundant.
    pub(crate) fn parse_block_expression(
        parser: &mut Parser,
    ) -> Result<BlockExpression, ParserErrorKind> {
        let mut statements: Vec<Statement> = Vec::new();

        // XXX: Check consistency with for parser, if parser and func parser
        if parser.curr_token != Token::LeftBrace {
            return Err(ParserErrorKind::UnstructuredError {
                message: format!("Expected a {{ to start the block expression"),
                span: parser.curr_token.into_span(),
            });
        }
        parser.advance_tokens();

        while (parser.curr_token != Token::RightBrace) && (parser.curr_token != Token::EOF) {
            statements.push(parser.parse_statement()?);
            parser.advance_tokens();
        }

        if parser.curr_token != Token::RightBrace {
            return Err(ParserErrorKind::UnstructuredError {
                message: format!("Expected a }} to end the block expression"),
                span: parser.curr_token.into_span(),
            });
        }

        Ok(BlockExpression(statements))
    }
}

#[cfg(test)]
mod test {
    use crate::{parser::test_parse, token::Token, BlockExpression, ExpressionKind};

    use super::BlockParser;

    fn expr_to_block(expr: ExpressionKind) -> BlockExpression {
        match expr {
            ExpressionKind::Block(block) => block,
            _ => unreachable!("expected a block expression"),
        }
    }

    /// This is the standard way to declare a block expression
    /// This block expression has a single item in it, an array
    #[test]
    fn valid_syntax() {
        const SRC: &'static str = r#"
            {
                [0,1,2,3,4]
            }
        "#;

        let mut parser = test_parse(SRC);

        let start = parser.curr_token.clone();

        let expr = BlockParser::parse(&mut parser).unwrap();

        let end = parser.curr_token.clone();

        // First check that the cursor was in the right position at
        // the start and at the end
        assert_eq!(start.token(), &Token::LeftBrace);
        assert_eq!(end.token(), &Token::RightBrace);

        // Check that we have a block expression
        // the contents is irrelevant, as this is
        // the job of another parser function.
        // However, note that the contents is also valid syntax
        let _block = expr_to_block(expr);
    }

    #[test]
    fn missing_starting_brace() {
        const SRC: &'static str = r#"
            
                [0,1,2,3,4]
            }
        "#;

        let mut parser = test_parse(SRC);
        let _err = BlockParser::parse(&mut parser).unwrap_err();
    }
    #[test]
    fn missing_end_brace() {
        const SRC: &'static str = r#"
            {
                [0,1,2,3,4]
            
        "#;

        let mut parser = test_parse(SRC);
        let _err = BlockParser::parse(&mut parser).unwrap_err();
    }
    #[test]
    fn invalid_contents() {
        /// Let ensure that although the braces are in the right place
        /// Invalid contents is caught
        ///
        /// Array content is invalid
        const INVALID_SRC: &'static str = r#"
            {
                [0,1,2,,]
            }
        "#;
        /// Array is missing it's ending bracket
        const INVALID_SRC_2: &'static str = r#"
            {
                [0,1,2,3
            }
        "#;
        /// Array is missing it's starting bracket
        const INVALID_SRC_3: &'static str = r#"
            {
                0,1,2,3]
            }
        "#;

        let mut parser = test_parse(INVALID_SRC);
        let _err = BlockParser::parse(&mut parser).unwrap_err();

        let mut parser = test_parse(INVALID_SRC_2);
        let _err = BlockParser::parse(&mut parser).unwrap_err();

        let mut parser = test_parse(INVALID_SRC_3);
        let _err = BlockParser::parse(&mut parser).unwrap_err();
    }
    #[test]
    fn regression_skip_first_token() {
        /// This test previously failed, as we were advancing past the first token
        /// without checking that it was indeed a `{`
        ///
        /// XXX: It's still debateable if this is needed, as this prefix functions
        /// is not called unless there is a left brace.
        /// To be on the conservative side, it should be left in,
        /// incase there is a way to manipulate the syntax
        const SRC: &'static str = r#"
                [[0,1,2,3,4]
            }
        "#;

        let mut parser = test_parse(SRC);
        let _err = BlockParser::parse(&mut parser).unwrap_err();
    }
}
