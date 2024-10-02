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
    /// Modifiers = 'unconstrained'? ItemVisibility 'comptime'? 'mut'?
    pub(crate) fn parse_modifiers(&mut self, allow_mutable: bool) -> Modifiers {
        let unconstrained = self.parse_modifier(Keyword::Unconstrained);

        let start_span = self.current_token_span;
        let visibility = self.parse_item_visibility();
        let visibility_span = self.span_since(start_span);

        let comptime = self.parse_modifier(Keyword::Comptime);
        let mutable = if allow_mutable { self.parse_modifier(Keyword::Mut) } else { None };

        Modifiers { visibility, visibility_span, unconstrained, comptime, mutable }
    }

    fn parse_modifier(&mut self, keyword: Keyword) -> Option<Span> {
        if self.eat_keyword(keyword) {
            Some(self.previous_token_span)
        } else {
            None
        }
    }
}
