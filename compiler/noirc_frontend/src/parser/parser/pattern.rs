use crate::ast::Pattern;

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_pattern(&mut self) -> Pattern {
        if let Some(ident) = self.eat_ident() {
            return Pattern::Identifier(ident);
        }

        // TODO: parse other patterns
        todo!("Parser")
    }
}
