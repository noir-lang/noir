use super::*;

pub struct IndexParser;

impl InfixParser for IndexParser {
    fn parse(parser: &mut Parser, collection_name: Expression) -> Expression {
        let collection_name_string = match collection_name {
            Expression::Ident(x) => x,
            _ => unimplemented!("collection name expression should only be an identifier"),
        };

        // Current token is now the left bracket that sits between the name of the collection
        // and the index. Since we can have constants be indices, we will just parse an expression
        // and not look for an integer
        let curr_precedence = Precedence::from(&parser.curr_token);
        parser.advance_tokens();
        let index = parser.parse_expression(curr_precedence).unwrap();

        // Skip the ']'
        if !parser.peek_check_variant_advance(&Token::RightBracket) {
            panic!("Expected a Right bracket to end the index operator")
        }

        let index_expr = IndexExpression {
            collection_name: collection_name_string.into(),
            index: index,
        };

        Expression::Index(Box::new(index_expr))
    }
}
