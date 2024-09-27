use noirc_errors::Span;

use crate::{ast::ItemVisibility, token::Keyword};

use super::Parser;

pub(crate) struct Modifiers {
    pub(crate) visibility: ItemVisibility,
    pub(crate) visibility_span: Span,
    pub(crate) unconstrained: Option<Span>,
    pub(crate) comptime: Option<Span>,
    pub(crate) mutable: Option<Span>,
}

impl<'a> Parser<'a> {
    pub(crate) fn parse_modifiers(&mut self) -> Modifiers {
        let unconstrained = self.parse_modifier(Keyword::Unconstrained);

        let start_span = self.current_token_span;
        let visibility = self.parse_item_visibility();
        let visibility_span = self.span_since(start_span);

        let comptime = self.parse_modifier(Keyword::Comptime);
        let mutable = self.parse_modifier(Keyword::Mut);

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
