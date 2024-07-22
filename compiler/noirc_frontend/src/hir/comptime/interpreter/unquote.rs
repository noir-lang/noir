use noirc_errors::Location;

use crate::{
    hir::comptime::{errors::IResult, value::unwrap_rc, Value},
    token::{SpannedToken, Token, Tokens},
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
            let span = token.to_span();
            match token.token() {
                Token::UnquoteMarker(id) => {
                    match self.evaluate(*id)? {
                        // If the value is already quoted we don't want to change the token stream by
                        // turning it into a Quoted block (which would add `quote`, `{`, and `}` tokens).
                        Value::Code(stream) => new_tokens.extend(unwrap_rc(stream).0),
                        value => {
                            let new_id =
                                value.into_hir_expression(self.elaborator.interner, location)?;
                            let new_token = Token::UnquoteMarker(new_id);
                            new_tokens.push(SpannedToken::new(new_token, span));
                        }
                    }
                }
                _ => new_tokens.push(token),
            }
        }

        Ok(Tokens(new_tokens))
    }
}
