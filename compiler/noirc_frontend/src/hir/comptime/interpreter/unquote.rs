use noirc_errors::Location;

use crate::{
    hir::comptime::{InterpreterError, errors::IResult},
    token::{LocatedToken, Token, Tokens},
};

use super::Interpreter;

impl Interpreter<'_, '_> {
    /// Evaluates any expressions within UnquoteMarkers in the given token list
    /// and replaces the expression held by the marker with the evaluated value
    /// in expression form.
    pub(super) fn substitute_unquoted_values_into_tokens(
        &mut self,
        tokens: Tokens,
    ) -> IResult<Vec<LocatedToken>> {
        let mut new_tokens = Vec::with_capacity(tokens.0.len());
        let mut tokens = tokens.0.into_iter();

        while let Some(token) = tokens.next() {
            let location = token.location();
            match token.into_token() {
                Token::UnquoteMarker(id) => {
                    let value = self.evaluate(id)?;
                    let tokens = value.into_tokens(self.elaborator.interner, location)?;
                    new_tokens.extend(tokens);
                }
                Token::Quote(tokens) => {
                    // Make sure to substitute in nested `quote { ... }` as well.
                    let tokens = self.substitute_unquoted_values_into_tokens(tokens)?;
                    new_tokens.push(LocatedToken::new(Token::Quote(Tokens(tokens)), location));
                }
                Token::Backslash => {
                    Self::escape_token(&mut tokens, &mut new_tokens, location)?;
                }
                token => new_tokens.push(LocatedToken::new(token, location)),
            }
        }

        Ok(new_tokens)
    }

    /// Escape the next token in the quoted token stream, issuing an error if it is not `$`.
    /// If it is `$`, it is appended to the `new_tokens` Vec.
    pub(crate) fn escape_token(
        tokens: &mut impl Iterator<Item = LocatedToken>,
        new_tokens: &mut Vec<LocatedToken>,
        default_location: Location,
    ) -> IResult<()> {
        match tokens.next() {
            Some(token) if *token.token() == Token::DollarSign => {
                new_tokens.push(token);
                Ok(())
            }
            Some(token) => {
                let location = token.location();
                let token = Some(token.into_token());
                Err(InterpreterError::UnexpectedEscapedTokenInQuote { token, location })
            }
            None => Err(InterpreterError::UnexpectedEscapedTokenInQuote {
                token: None,
                location: default_location,
            }),
        }
    }
}
