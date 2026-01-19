use noirc_frontend::{
    ast::{GenericTypeArgKind, GenericTypeArgs, UnresolvedGeneric},
    token::{Keyword, Token},
};

use super::Formatter;

impl Formatter<'_> {
    pub(super) fn format_generics(&mut self, generics: Vec<UnresolvedGeneric>) {
        self.skip_comments_and_whitespace();

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
        if self.is_at(Token::Comma) {
            self.bump();
        }

        self.write_token(Token::Greater);
    }

    fn format_generic(&mut self, generic: UnresolvedGeneric) {
        self.skip_comments_and_whitespace();
        match generic {
            UnresolvedGeneric::Variable(ident, trait_bounds) => {
                let ident =
                    ident.ident().expect("Resolved generics should not be present in the AST");
                self.write_identifier(ident.clone());
                if !trait_bounds.is_empty() {
                    self.write_token(Token::Colon);
                    self.write_space();
                    self.format_trait_bounds(trait_bounds);
                }
            }
            UnresolvedGeneric::Numeric { ident, typ } => {
                self.write_keyword(Keyword::Let);
                self.write_space();
                let ident =
                    ident.ident().expect("Resolved generics should not be present in the AST");
                self.write_identifier(ident.clone());
                self.write_token(Token::Colon);
                self.write_space();
                self.format_type(typ);
            }
        }
    }

    pub(super) fn format_generic_type_args(&mut self, mut generics: GenericTypeArgs) {
        self.skip_comments_and_whitespace();
        if self.token != Token::Less {
            return;
        }

        self.write_token(Token::Less);

        for (index, kind) in generics.kinds.into_iter().enumerate() {
            self.skip_comments_and_whitespace();

            if index > 0 {
                self.write_token(Token::Comma);
                self.write_space();
            }

            match kind {
                GenericTypeArgKind::Ordered => {
                    let typ = generics.ordered_args.remove(0);
                    self.format_type(typ);
                }
                GenericTypeArgKind::Named => {
                    let (name, typ) = generics.named_args.remove(0);
                    self.write_identifier(name);
                    self.write_space();
                    self.write_token(Token::Assign);
                    self.write_space();
                    self.format_type(typ);
                }
            }
        }

        self.skip_comments_and_whitespace();

        // Don't include a trailing comma if there is one
        if self.is_at(Token::Comma) {
            self.bump();
            self.skip_comments_and_whitespace();
        }

        self.write_token(Token::Greater);
    }
}
