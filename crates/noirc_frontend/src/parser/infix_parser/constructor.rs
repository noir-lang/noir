use crate::{ConstructorExpression, Ident, token::TokenKind};

use super::*;

/// Parses a constructor expression
///
/// ```noir
/// IDENT {
///     <IDENT>: <EXPR>,
///     ...
/// }
///```
///
/// Cursor Start : `{`
/// Cursor End : `}`
pub fn parse(parser: &mut Parser, collection_name: Expression) -> ParserExprKindResult {
    let type_name = match collection_name.kind {
        ExpressionKind::Path(path) => path,
        _ => {
            return Err(ParserErrorKind::UnstructuredError {
                message: "expected an identifier or path for the type name of this constructor expression".to_string(),
                span: collection_name.span,
            });
        }
    };

    // Advance past `{` to either an IDENT or `}`
    parser.advance_tokens();
    let fields = parse_fields(parser)?;

    let constructor = ConstructorExpression { type_name, fields };
    Ok(ExpressionKind::Constructor(Box::new(constructor)))
}

/// Cursor Start : IDENT or `}`
///
/// Cursor End : `}`
fn parse_fields(parser: &mut Parser) -> Result<Vec<(Ident, Expression)>, ParserErrorKind> {
    // Current token is `{`, next should be an Ident or }
    let mut fields: Vec<(Ident, Expression)> = Vec::new();

    while parser.curr_token.kind() == TokenKind::Ident {
        let name: Ident = parser.curr_token.clone().into();

        // Current tokens should be `IDENT` and ':""'
        // Advance past both
        parser.peek_check_variant_advance(&Token::Colon)?;
        parser.advance_tokens();

        let typ = parser.parse_expression(Precedence::Lowest)?;
        parser.advance_tokens();

        // TODO: This also makes commas between fields optional
        if parser.curr_token.token() == &Token::Comma {
            parser.advance_tokens();
        }

        fields.push((name, typ));
    }

    Ok(fields)
}
