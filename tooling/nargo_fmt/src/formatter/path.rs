use noirc_frontend::{
    ast::{Path, PathKind, UnresolvedType},
    token::{Keyword, Token},
};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_path(&mut self, path: Path) {
        self.skip_comments_and_whitespace();

        match path.kind {
            PathKind::Plain => (),
            PathKind::Crate => {
                self.write_keyword(Keyword::Crate);
                self.write_token(Token::DoubleColon);
            }
            PathKind::Dep => {
                self.write_keyword(Keyword::Dep);
                self.write_token(Token::DoubleColon);
            }
            PathKind::Super => {
                self.write_keyword(Keyword::Super);
                self.write_token(Token::DoubleColon);
            }
        }

        for (index, segment) in path.segments.into_iter().enumerate() {
            if index > 0 {
                self.write_token(Token::DoubleColon);
            }
            self.write_identifier(segment.ident);

            if let Some(generics) = segment.generics {
                self.format_turbofish(generics);
            }
        }
    }

    pub(super) fn format_turbofish(&mut self, generics: Vec<UnresolvedType>) {
        self.write_token(Token::DoubleColon);
        self.write_token(Token::Less);
        for (index, typ) in generics.into_iter().enumerate() {
            if index > 0 {
                self.write_comma();
                self.write_space();
            }
            self.format_type(typ);
        }

        // Skip trailing comma, if any
        self.skip_comments_and_whitespace();
        if self.is_at(Token::Comma) {
            self.bump();
        }

        self.write_token(Token::Greater);
    }
}
