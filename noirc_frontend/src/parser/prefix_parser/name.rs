use super::*;

pub struct NameParser;

impl NameParser {
    /// NameParser is used to parse Identifiers which appear in the
    /// AST as expressions.
    ///
    /// Cursor Start : `IDENT`
    ///
    /// Cursor End : `IDENT`
    pub fn parse(parser: &mut Parser) -> ParserExprKindResult {
        if let Token::Ident(x) = parser.curr_token.token() {
            return Ok(ExpressionKind::Ident(x.clone()));
        }

        return Err(ParserErrorKind::UnexpectedTokenKind {
            span: parser.curr_token.into_span(),
            expected: TokenKind::Ident,
            found: parser.curr_token.kind(),
        });
    }
}
