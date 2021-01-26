use super::*;

pub struct BlockParser;

impl BlockParser {
    pub fn parse(parser: &mut Parser) -> ParserExprKindResult { 
        // Currently on the `{`
        let block_expr = BlockParser::parse_block_expression(parser)?;
        
        Ok(ExpressionKind::Block(block_expr))

    }
    pub(crate) fn parse_block_expression(parser: &mut Parser) -> Result<BlockExpression, ParserError> {
        let mut statements: Vec<Statement> = Vec::new();
        
        // Advance past the current token which is the left brace which was used to start the block statement
        // XXX: Check consistency with for parser, if parser and func parser
        parser.advance_tokens();
    
        while (parser.curr_token != Token::RightBrace) && (parser.curr_token != Token::EOF) {
            statements.push(parser.parse_statement()?);
            parser.advance_tokens();
        }
    
        if parser.curr_token != Token::RightBrace {
            return Err(ParserErrorKind::UnstructuredError{message : format!("Expected a }} to end the block expression"), span : parser.curr_token.into_span()}.into_err(parser.file_id));
        }
    
        Ok(BlockExpression(statements))
    }
}
