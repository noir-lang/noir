use crate::{
    parser::{Item, ItemKind},
    token::Keyword,
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_items(&mut self) -> Vec<Item> {
        let mut items = Vec::new();

        while let Some(item) = self.parse_item() {
            items.push(item);
        }

        items
    }

    fn parse_item(&mut self) -> Option<Item> {
        let doc_comments = self.parse_outer_doc_comments();

        let start_span = self.current_token_span;
        let kind = self.parse_item_kind()?;
        let span = self.span_since(start_span);

        Some(Item { kind, span, doc_comments })
    }

    fn parse_item_kind(&mut self) -> Option<ItemKind> {
        if let Some(kind) = self.parse_inner_attribute() {
            return Some(ItemKind::InnerAttribute(kind));
        }

        let attributes = self.parse_attributes();

        // Functions allow `unconstrained` before their visibility
        // so we check that before and after the visibility
        let mut unconstrained = self.eat_keyword(Keyword::Unconstrained);

        let visibility = self.parse_item_visibility();

        let start_span = self.current_token_span;

        unconstrained = unconstrained || self.eat_keyword(Keyword::Unconstrained);

        let comptime = self.eat_keyword(Keyword::Comptime);
        let mutable = self.eat_keyword(Keyword::Mut);

        if self.eat_keyword(Keyword::Use) {
            // TODO: error if there's comptime, mutable or unconstrained
            let use_tree = self.parse_use_tree();
            return Some(ItemKind::Import(use_tree, visibility));
        }

        if let Some(is_contract) = self.eat_mod_or_contract() {
            // TODO: error if there's comptime, mutable or unconstrained
            return Some(self.parse_module_or_contract(attributes, is_contract));
        }

        if self.eat_keyword(Keyword::Struct) {
            // TODO: error if there's comptime or mutable or unconstrained
            return Some(ItemKind::Struct(self.parse_struct(attributes, visibility, start_span)));
        }

        if self.eat_keyword(Keyword::Impl) {
            // TODO: error if there's comptime or mutable or unconstrained
            return Some(ItemKind::Impl(self.parse_impl()));
        }

        if self.eat_keyword(Keyword::Global) {
            // TODO: error if there's unconstrained
            return Some(ItemKind::Global(self.parse_global(attributes, comptime, mutable)));
        }

        if self.eat_keyword(Keyword::Type) {
            // TODO: error if there's comptime or mutable or unconstrained
            return Some(ItemKind::TypeAlias(self.parse_type_alias(start_span)));
        }

        if self.eat_keyword(Keyword::Fn) {
            // TODO: error if there's mutable
            return Some(ItemKind::Function(self.parse_function(
                attributes,
                visibility,
                comptime,
                unconstrained,
                false, // allow_self
                start_span,
            )));
        }

        None
    }

    fn eat_mod_or_contract(&mut self) -> Option<bool> {
        if self.eat_keyword(Keyword::Mod) {
            Some(false)
        } else if self.eat_keyword(Keyword::Contract) {
            Some(true)
        } else {
            None
        }
    }
}
