use noirc_errors::Span;

use crate::{ast::ItemVisibility, token::Keyword};

use super::Parser;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct Modifiers {
    pub(crate) visibility: ItemVisibility,
    pub(crate) visibility_span: Span,
    pub(crate) unconstrained_before_visibility: Option<Span>,
    pub(crate) unconstrained_after_visibility: Option<Span>,
    pub(crate) comptime: Option<Span>,
    pub(crate) mutable: Option<Span>,
}

impl Modifiers {
    pub(crate) fn is_unconstrained(&self) -> bool {
        self.unconstrained_before_visibility.is_some()
            || self.unconstrained_after_visibility.is_some()
    }
}

impl<'a> Parser<'a> {
    /// Modifiers = 'unconstrained'? ItemVisibility 'unconstrained'? 'comptime'? 'mut'?
    ///
    /// NOTE: we allow `unconstrained` to be before and after the visibility for backwards compatibility
    /// (we don't error in this case because the formatter doesn't format when there are errors).
    /// The formatter will remove the duplicate one and put it after the visibility.
    /// After some time we can change this to only allow `unconstrained` after the visibility.
    pub(crate) fn parse_modifiers(&mut self, allow_mutable: bool) -> Modifiers {
        let unconstrained_before_visibility = if self.eat_keyword(Keyword::Unconstrained) {
            Some(self.previous_token_span)
        } else {
            None
        };

        let start_span = self.current_token_span;
        let visibility = self.parse_item_visibility();
        let visibility_span = self.span_since(start_span);

        let unconstrained_after_visibility = if self.eat_keyword(Keyword::Unconstrained) {
            Some(self.previous_token_span)
        } else {
            None
        };

        let comptime =
            if self.eat_keyword(Keyword::Comptime) { Some(self.previous_token_span) } else { None };
        let mutable = if allow_mutable && self.eat_keyword(Keyword::Mut) {
            Some(self.previous_token_span)
        } else {
            None
        };

        Modifiers {
            visibility,
            visibility_span,
            unconstrained_before_visibility,
            unconstrained_after_visibility,
            comptime,
            mutable,
        }
    }
}
