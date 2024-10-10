use noirc_errors::Span;

use crate::{ast::ItemVisibility, token::Keyword};

use super::Parser;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct Modifiers {
    pub(crate) visibility: ItemVisibility,
    pub(crate) visibility_span: Span,
    pub(crate) unconstrained: Option<Span>,
    pub(crate) comptime: Option<Span>,
    pub(crate) mutable: Option<Span>,
}

impl<'a> Parser<'a> {
    /// Modifiers = ItemVisibility 'unconstrained'? 'comptime'? 'mut'?
    ///
    /// NOTE: we also allow `unconstrained` before the visibility for backwards compatibility.
    /// The formatter will put it after the visibility.
    pub(crate) fn parse_modifiers(&mut self, allow_mutable: bool) -> Modifiers {
        let unconstrained = if self.eat_keyword(Keyword::Unconstrained) {
            Some(self.previous_token_span)
        } else {
            None
        };

        let start_span = self.current_token_span;
        let visibility = self.parse_item_visibility();
        let visibility_span = self.span_since(start_span);

        let unconstrained = if unconstrained.is_none() {
            if self.eat_keyword(Keyword::Unconstrained) {
                Some(self.previous_token_span)
            } else {
                None
            }
        } else {
            unconstrained
        };

        let comptime =
            if self.eat_keyword(Keyword::Comptime) { Some(self.previous_token_span) } else { None };
        let mutable = if allow_mutable && self.eat_keyword(Keyword::Mut) {
            Some(self.previous_token_span)
        } else {
            None
        };

        Modifiers { visibility, visibility_span, unconstrained, comptime, mutable }
    }
}
