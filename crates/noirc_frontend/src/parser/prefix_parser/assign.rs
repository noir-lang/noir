use super::*;
use crate::ast::AssignStatement;

pub struct AssignParser;

/// Parses statements of the form
/// - IDENT = <EXPR>
///
/// Cursor Start : `IDENT`
///
/// Cursor End : `;`
impl AssignParser {
    pub(crate) fn parse_statement(parser: &mut Parser) -> Result<AssignStatement, ParserErrorKind> {
        let identifier: Ident = parser.curr_token.clone().into();

        parser.peek_check_variant_advance(&Token::Assign)?;
        parser.advance_tokens();

        // Current token should now be the start of the expression
        let expression = parser.parse_expression(Precedence::Lowest)?;

        // XXX: Add a `help` note to tell the user to add a semi colon here
        parser.peek_check_variant_advance(&Token::Semicolon)?;

        Ok(AssignStatement {
            identifier,
            expression,
        })
    }
}
