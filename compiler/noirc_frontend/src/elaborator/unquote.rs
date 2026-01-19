//! Token stream processing for macro unquoting and variable interpolation.

use crate::{
    ast::Path,
    token::{Keyword, LocatedToken, Token, Tokens},
};

use super::Elaborator;

impl Elaborator<'_> {
    /// Go through the given tokens looking for a '$' token followed by a variable to unquote.
    /// Each time these two tokens are found, they are replaced by a new UnquoteMarker token
    /// containing the ExprId of the resolved variable to unquote.
    pub fn find_unquoted_exprs_tokens(&mut self, tokens: Tokens) -> Tokens {
        let token_count = tokens.0.len();
        let mut new_tokens = Vec::with_capacity(token_count);
        let mut tokens = tokens.0.into_iter();

        while let Some(token) = tokens.next() {
            let is_unquote = matches!(token.token(), Token::DollarSign);
            new_tokens.push(token);

            if is_unquote {
                if let Some(next) = tokens.next() {
                    let location = next.location();

                    match next.into_token() {
                        Token::Ident(name) => {
                            // Don't want the leading `$` anymore
                            new_tokens.pop();
                            let path = Path::from_single(name, location);
                            let (expr_id, _) = self.elaborate_variable(path);
                            new_tokens
                                .push(LocatedToken::new(Token::UnquoteMarker(expr_id), location));
                        }
                        // If we see `$crate` resolve the crate to the current crate now so it
                        // stays even if the rest of the quote is unquoted elsewhere.
                        Token::Keyword(Keyword::Crate) => {
                            new_tokens.pop();
                            let token = Token::InternedCrate(self.crate_id);
                            new_tokens.push(LocatedToken::new(token, location));
                        }
                        other_next => new_tokens.push(LocatedToken::new(other_next, location)),
                    }
                }
            }
        }

        Tokens(new_tokens)
    }
}
