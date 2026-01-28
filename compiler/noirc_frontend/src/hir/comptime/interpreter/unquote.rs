use crate::{
    hir::comptime::errors::IResult,
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

        for token in tokens.0 {
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
                token => new_tokens.push(LocatedToken::new(token, location)),
            }
        }

        Ok(new_tokens)
    }
}
