use crate::{
    ast::Path,
    token::{LocatedToken, Token, Tokens},
};

use super::Elaborator;

impl<'a> Elaborator<'a> {
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
                    let location = next.to_location();

                    match next.into_token() {
                        Token::Ident(name) => {
                            // Don't want the leading `$` anymore
                            new_tokens.pop();
                            let path = Path::from_single(name, location);
                            let (expr_id, _) = self.elaborate_variable(path);
                            new_tokens
                                .push(LocatedToken::new(Token::UnquoteMarker(expr_id), location));
                        }
                        other_next => new_tokens.push(LocatedToken::new(other_next, location)),
                    }
                }
            }
        }

        Tokens(new_tokens)
    }
}
