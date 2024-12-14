use crate::{
    ast::Path,
    token::{SpannedToken, Token, Tokens},
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
                    let span = next.to_span();

                    match next.into_token() {
                        Token::Ident(name) => {
                            // Don't want the leading `$` anymore
                            new_tokens.pop();
                            let path = Path::from_single(name, span);
                            let (expr_id, _) = self.elaborate_variable(path);
                            new_tokens.push(SpannedToken::new(Token::UnquoteMarker(expr_id), span));
                        }
                        other_next => new_tokens.push(SpannedToken::new(other_next, span)),
                    }
                }
            }
        }

        Tokens(new_tokens)
    }
}
