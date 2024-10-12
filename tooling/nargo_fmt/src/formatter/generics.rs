use noirc_frontend::{
    ast::UnresolvedGeneric,
    token::{Keyword, Token},
};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_generics(&mut self, generics: Vec<UnresolvedGeneric>) {
        if self.token != Token::Less {
            return;
        }

        self.write_token(Token::Less);
        for (index, generic) in generics.into_iter().enumerate() {
            if index > 0 {
                self.write_comma();
                self.write_space();
            }
            self.format_generic(generic);
        }
        self.skip_comments_and_whitespace();

        // Trailing comma
        if self.token == Token::Comma {
            self.bump();
        }

        self.write_token(Token::Greater);
    }

    fn format_generic(&mut self, generic: UnresolvedGeneric) {
        self.skip_comments_and_whitespace();
        match generic {
            UnresolvedGeneric::Variable(ident) => {
                self.write_identifier(ident);
            }
            UnresolvedGeneric::Numeric { ident, typ } => {
                self.write_keyword(Keyword::Let);
                self.write_space();
                self.write_identifier(ident);
                self.write_token(Token::Colon);
                self.write_space();
                self.format_type(typ);
            }
            UnresolvedGeneric::Resolved(..) => {
                unreachable!("Resolved generics should not be present in the AST")
            }
        }
    }
}
