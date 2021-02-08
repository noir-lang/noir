use super::*;

pub struct IndexParser;

impl IndexParser {
    pub fn parse(parser: &mut Parser, collection_name: Expression) -> ParserExprKindResult {
        let err = ParserErrorKind::UnstructuredError{message: format!("Expected an identifier for the collection name. Arbitrary expressions are yet to arrive"), span : collection_name.span}.into_err(parser.file_id);
        let collection_name = match collection_name.kind {
            ExpressionKind::Path(path) => path.into_ident().ok_or(err)?,
            _ => return Err(err),
        };

        // Current token is now the left bracket that sits between the name of the collection
        // and the index. Since we can have constants be indices, we will just parse an expression
        // and not look for an integer
        let curr_precedence = Precedence::from(&parser.curr_token);
        parser.advance_tokens();
        let index = parser.parse_expression(curr_precedence)?;

        // Skip the ']'
        parser.peek_check_variant_advance(&Token::RightBracket)?;

        let index_expr = IndexExpression {
            collection_name: collection_name.clone(),
            index: index,
        };

        Ok(ExpressionKind::Index(Box::new(index_expr)))
    }
}
