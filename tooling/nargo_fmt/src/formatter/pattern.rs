use noirc_frontend::ast::Pattern;

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_pattern(&mut self, pattern: Pattern) {
        self.skip_comments_and_whitespace();

        match pattern {
            Pattern::Identifier(ident) => self.write_identifier(ident),
            Pattern::Mutable(pattern, span, _) => todo!(),
            Pattern::Tuple(vec, span) => todo!(),
            Pattern::Struct(path, vec, span) => todo!(),
            Pattern::Interned(interned_pattern, span) => todo!(),
        }
    }
}
