use noirc_frontend::{
    ast::{TraitBound, UnresolvedTraitConstraint},
    token::{Keyword, Token},
};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_where_clause(
        &mut self,
        constraints: Vec<UnresolvedTraitConstraint>,
        write_trailing_comma_and_new_line: bool,
    ) {
        assert!(!constraints.is_empty());

        self.skip_comments_and_whitespace();
        self.write_line();
        self.write_indentation();
        self.write_keyword(Keyword::Where);
        self.increase_indentation();

        // If we have `where F: Foo + Bar`, that's actually parsed as two constraints: `F: Foo` and `F: Bar`.
        // To format it we'll have to skip the second type `F` if we find a `+` token.
        let mut write_type = true;

        for constraint in constraints {
            if write_type {
                self.write_line();
                self.write_indentation();
                self.format_type(constraint.typ);
                self.write_token(Token::Colon);
                self.write_space();
            }

            self.format_trait_bound(constraint.trait_bound);
            self.skip_comments_and_whitespace();

            if self.is_at(Token::Plus) {
                self.write_space();
                self.write_token(Token::Plus);
                self.write_space();
                write_type = false;
                continue;
            }

            write_type = true;

            if self.is_at(Token::Comma) {
                if write_trailing_comma_and_new_line {
                    self.write_token(Token::Comma);
                } else {
                    self.skip_comments_and_whitespace();
                    self.bump();
                }
            } else if write_trailing_comma_and_new_line {
                self.write(",");
            }
        }

        self.decrease_indentation();

        if write_trailing_comma_and_new_line {
            self.write_line();
            self.write_indentation();
        }
    }

    pub(super) fn format_trait_bound(&mut self, trait_bound: TraitBound) {
        self.format_path(trait_bound.trait_path);
        self.format_generic_type_args(trait_bound.trait_generics);
    }
}
