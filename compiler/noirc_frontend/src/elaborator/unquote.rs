//! Token stream processing for macro unquoting and variable interpolation.

use crate::{
    ast::Path,
    hir::comptime::Interpreter,
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
            let location = token.location();
            let token = token.into_token();

            // If we find `quote { ... }` it means this is nested inside another `quote { ... }`.
            // We need to replace `$...` inside the `quote` tokens as well.
            if let Token::Quote(tokens) = token {
                let tokens = self.find_unquoted_exprs_tokens(tokens);
                new_tokens.push(LocatedToken::new(Token::Quote(tokens), location));
                continue;
            }

            if Token::Backslash == token {
                if let Err(error) =
                    Interpreter::escape_token(&mut tokens, &mut new_tokens, location)
                {
                    self.push_err(error);
                }
                continue;
            }

            let is_unquote = matches!(token, Token::DollarSign);
            new_tokens.push(LocatedToken::new(token, location));

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
                        Token::Quote(tokens) => {
                            new_tokens.pop();
                            let tokens = self.find_unquoted_exprs_tokens(tokens);
                            new_tokens.push(LocatedToken::new(Token::Quote(tokens), location));
                        }
                        other_next => new_tokens.push(LocatedToken::new(other_next, location)),
                    }
                }
            }
        }

        Tokens(new_tokens)
    }
}
