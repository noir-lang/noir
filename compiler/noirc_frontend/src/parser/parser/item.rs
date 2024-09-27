use crate::{
    parser::{Item, ItemKind},
    token::Keyword,
};

use super::{impls::Impl, Parser};

impl<'a> Parser<'a> {
    pub(crate) fn parse_items(&mut self) -> Vec<Item> {
        let mut items = Vec::new();

        while let Some(item) = self.parse_item() {
            items.push(item);
        }

        items
    }

    fn parse_item(&mut self) -> Option<Item> {
        let start_span = self.current_token_span;
        let doc_comments = self.parse_outer_doc_comments();
        let kind = self.parse_item_kind()?;
        let span = self.span_since(start_span);

        Some(Item { kind, span, doc_comments })
    }

    fn parse_item_kind(&mut self) -> Option<ItemKind> {
        if let Some(kind) = self.parse_inner_attribute() {
            return Some(ItemKind::InnerAttribute(kind));
        }

        let start_span = self.current_token_span;
        let attributes = self.parse_attributes();

        let modifiers = self.parse_modifiers();

        if self.eat_keyword(Keyword::Use) {
            // TODO: error if there's comptime, mutable or unconstrained
            let use_tree = self.parse_use_tree();
            return Some(ItemKind::Import(use_tree, modifiers.visibility));
        }

        if let Some(is_contract) = self.eat_mod_or_contract() {
            // TODO: error if there's comptime, mutable or unconstrained
            return Some(self.parse_module_or_contract(attributes, is_contract));
        }

        if self.eat_keyword(Keyword::Struct) {
            // TODO: error if there's comptime or mutable or unconstrained
            return Some(ItemKind::Struct(self.parse_struct(
                attributes,
                modifiers.visibility,
                start_span,
            )));
        }

        if self.eat_keyword(Keyword::Impl) {
            // TODO: error if there's comptime or mutable or unconstrained
            return Some(match self.parse_impl() {
                Impl::Impl(type_impl) => ItemKind::Impl(type_impl),
                Impl::TraitImpl(noir_trait_impl) => ItemKind::TraitImpl(noir_trait_impl),
            });
        }

        if self.eat_keyword(Keyword::Trait) {
            // TODO: error if there's comptime or mutable or unconstrained
            return Some(ItemKind::Trait(self.parse_trait(
                attributes,
                modifiers.visibility,
                start_span,
            )));
        }

        if self.eat_keyword(Keyword::Global) {
            // TODO: error if there's unconstrained
            return Some(ItemKind::Global(self.parse_global(
                attributes,
                modifiers.comptime.is_some(),
                modifiers.mutable.is_some(),
            )));
        }

        if self.eat_keyword(Keyword::Type) {
            // TODO: error if there's comptime or mutable or unconstrained
            return Some(ItemKind::TypeAlias(self.parse_type_alias(start_span)));
        }

        if self.eat_keyword(Keyword::Fn) {
            // TODO: error if there's mutable
            return Some(ItemKind::Function(self.parse_function(
                attributes,
                modifiers.visibility,
                modifiers.comptime.is_some(),
                modifiers.unconstrained.is_some(),
                false, // allow_self
            )));
        }

        // TODO: error

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
