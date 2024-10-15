use noirc_frontend::{
    ast::{Path, PathKind},
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

            if let Some(..) = segment.generics {
                todo!("Format path generics");
            }
        }
    }
}
