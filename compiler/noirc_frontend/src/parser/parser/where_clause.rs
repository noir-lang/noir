use crate::ast::UnresolvedTraitConstraint;

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_where_clause(&mut self) -> Vec<UnresolvedTraitConstraint> {
        // TODO: parse where clause
        Vec::new()
    }
}
