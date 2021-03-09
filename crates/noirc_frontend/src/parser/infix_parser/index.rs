use super::*;

pub struct IndexParser;

impl IndexParser {
    /// Parses an Index Expression of the form:
    ///
    /// EXPR[EXPR]
    ///
    /// Cursor Start : `[`
    ///
    /// Cursor End : `]`
    pub fn parse(parser: &mut Parser, collection_name: Expression) -> ParserExprKindResult {
        // XXX: We will clean up these unnecessary format! allocations in the refactor after the next
        let msg = format!("expected an identifier for the collection name. Arbitrary expressions are yet to arrive");
        let err = ParserErrorKind::UnstructuredError {
            message: msg,
            span: collection_name.span,
        };

        let collection_name = match collection_name.kind {
            ExpressionKind::Path(path) => path.into_ident().ok_or(err)?,
            _ => return Err(err),
        };

        // Current token is '['
        //
        // Bump Cursor.
        //
        // For a well-formed Index Expression,
        // this will put the cursor at the beginning of the index's token
        //
        parser.advance_tokens();
        let index = parser.parse_expression(Precedence::Lowest)?;

        // Current token is now at the end of the expression
        //
        // Peek ahead and check if the next token is `]`
        parser.peek_check_variant_advance(&Token::RightBracket)?;

        let index_expr = IndexExpression {
            collection_name,
            index,
        };

        Ok(ExpressionKind::Index(Box::new(index_expr)))
    }
}

#[cfg(test)]
mod test {

    use super::IndexParser;
    use crate::parser::{dummy_expr, test_parse};

    #[test]
    fn valid_syntax() {
        let vectors = vec!["[9]", "[x+a]", "[foo+5]", "[bar]"];

        for src in vectors {
            let mut parser = test_parse(src);

            let start = parser.curr_token.clone();

            IndexParser::parse(&mut parser, dummy_expr()).unwrap();

            let end = parser.curr_token.clone();

            assert_eq!(start, crate::token::Token::LeftBracket);
            assert_eq!(end, crate::token::Token::RightBracket);
        }
    }
}
