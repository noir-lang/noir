use noirc_errors::Location;

use crate::{ast::ItemVisibility, token::Keyword};

use super::Parser;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct Modifiers {
    pub(crate) visibility: ItemVisibility,
    pub(crate) visibility_location: Location,
    pub(crate) unconstrained: Option<Location>,
    pub(crate) comptime: Option<Location>,
    pub(crate) mutable: Option<Location>,
}

impl<'a> Parser<'a> {
    /// Modifiers = ItemVisibility 'unconstrained'? 'comptime'? 'mut'?
    ///
    /// NOTE: we also allow `unconstrained` before the visibility for backwards compatibility.
    /// The formatter will put it after the visibility.
    pub(crate) fn parse_modifiers(&mut self, allow_mutable: bool) -> Modifiers {
        let unconstrained = if self.eat_keyword(Keyword::Unconstrained) {
            Some(self.previous_token_location)
        } else {
            None
        };

        let start_location = self.current_token_location;
        let visibility = self.parse_item_visibility();
        let visibility_location = self.location_since(start_location);

        let unconstrained = if unconstrained.is_none() {
            if self.eat_keyword(Keyword::Unconstrained) {
                Some(self.previous_token_location)
            } else {
                None
            }
        } else {
            unconstrained
        };

        let comptime = if self.eat_keyword(Keyword::Comptime) {
            Some(self.previous_token_location)
        } else {
            None
        };
        let mutable = if allow_mutable && self.eat_keyword(Keyword::Mut) {
            Some(self.previous_token_location)
        } else {
            None
        };

        Modifiers { visibility, visibility_location, unconstrained, comptime, mutable }
    }
}
