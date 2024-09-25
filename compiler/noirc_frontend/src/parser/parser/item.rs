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
            return Some(kind);
        }

        let visibility = self.parse_item_visibility();
        let attributes = self.parse_attributes();

        let start_span = self.current_token_span;

        if self.eat_keyword(Keyword::Use) {
            let use_tree = self.parse_use_tree();
            return Some(ItemKind::Import(use_tree, visibility));
        }

        let module_or_contract = if self.eat_keyword(Keyword::Mod) {
            Some(false)
        } else if self.eat_keyword(Keyword::Contract) {
            Some(true)
        } else {
            None
        };

        if self.eat_keyword(Keyword::Struct) {
            return Some(ItemKind::Struct(self.parse_struct(attributes, visibility, start_span)));
        }

        if let Some(is_contract) = module_or_contract {
            return Some(self.parse_module_or_contract(attributes, is_contract));
        }

        None
    }
}
