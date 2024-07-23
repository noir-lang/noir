use noirc_errors::Location;

use crate::{
    hir::comptime::errors::IResult,
    token::{Token, Tokens},
};

use super::Interpreter;

impl<'local, 'interner> Interpreter<'local, 'interner> {
    /// Evaluates any expressions within UnquoteMarkers in the given token list
    /// and replaces the expression held by the marker with the evaluated value
    /// in expression form.
    pub(super) fn substitute_unquoted_values_into_tokens(
        &mut self,
        tokens: Tokens,
        location: Location,
    ) -> IResult<Tokens> {
        let mut new_tokens = Vec::with_capacity(tokens.0.len());

        for token in tokens.0 {
            match token.token() {
                Token::UnquoteMarker(id) => {
                    let value = self.evaluate(*id)?;
                    let tokens = value.into_tokens(self.elaborator.interner, location)?;
                    new_tokens.extend(tokens.0);
                }
                _ => new_tokens.push(token),
            }
        }

        Ok(Tokens(new_tokens))
    }
}
