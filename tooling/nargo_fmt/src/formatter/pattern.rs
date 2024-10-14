use noirc_frontend::ast::Pattern;

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_pattern(&mut self, pattern: Pattern) {
        self.skip_comments_and_whitespace();

        match pattern {
            Pattern::Identifier(ident) => self.write_identifier(ident),
            Pattern::Mutable(_pattern, _span, _) => todo!("Format mutable pattern"),
            Pattern::Tuple(_vec, _span) => todo!("Format tuple pattern"),
            Pattern::Struct(_path, _vec, _span) => todo!("Format struct pattern"),
            Pattern::Interned(..) => {
                unreachable!("Should not be present in the AST")
            }
        }
    }
}
