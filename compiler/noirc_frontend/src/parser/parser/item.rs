use crate::{
    parser::{labels::ParsingRuleLabel, Item, ItemKind},
    token::{Keyword, Token},
};

use super::{impls::Impl, parse_many::without_separator, Parser};

impl<'a> Parser<'a> {
    pub(crate) fn parse_top_level_items(&mut self) -> Vec<Item> {
        self.parse_module_items(
            false, // nested
        )
    }

    pub(crate) fn parse_module_items(&mut self, nested: bool) -> Vec<Item> {
        self.parse_many("items", without_separator(), |parser| {
            parser.parse_module_item_in_list(nested)
        })
    }

    fn parse_module_item_in_list(&mut self, nested: bool) -> Option<Item> {
        loop {
            // We only break out of the loop on `}` if we are inside a `mod { ..`
            if nested && self.at(Token::RightBrace) {
                return None;
            }

            // We always break on EOF (we don't error because if we are inside `mod { ..`
            // the outer parsing logic will error instead)
            if self.at_eof() {
                return None;
            }

            let Some(item) = self.parse_item() else {
                // If we couldn't parse an item we check which token we got
                match self.token.token() {
                    Token::RightBrace if nested => {
                        return None;
                    }
                    Token::EOF => return None,
                    _ => (),
                }

                self.expected_label(ParsingRuleLabel::Item);
                // We'll try parsing an item starting on the next token
                self.bump();
                continue;
            };

            return Some(item);
        }
    }

    /// Parses an item inside an impl or trait, with good recovery:
    /// - If we run into EOF, we error that we expect a '}'
    /// - If we can't parse an item and we don't end up in '}', error but try with the next token
    pub(super) fn parse_item_in_list<T, F>(
        &mut self,
        label: ParsingRuleLabel,
        mut f: F,
    ) -> Option<T>
    where
        F: FnMut(&mut Parser<'a>) -> Option<T>,
    {
        loop {
            if self.at_eof() {
                self.expected_token(Token::RightBrace);
                return None;
            }

            let Some(item) = f(self) else {
                if !self.at(Token::RightBrace) {
                    self.expected_label(label.clone());

                    // Try with the next token
                    self.bump();
                    continue;
                }

                return None;
            };

            return Some(item);
        }
    }

    /// Item = OuterDocComments ItemKind
    fn parse_item(&mut self) -> Option<Item> {
        let start_span = self.current_token_span;
        let doc_comments = self.parse_outer_doc_comments();
        let kind = self.parse_item_kind()?;
        let span = self.span_since(start_span);

        Some(Item { kind, span, doc_comments })
    }

    /// ItemKind
    ///     = InnerAttribute
    ///     | Attributes Modifiers
    ///         ( Use
    ///         | ModOrContract
    ///         | Struct
    ///         | Impl
    ///         | Trait
    ///         | Global
    ///         | TypeAlias
    ///         | Function
    ///         )
    fn parse_item_kind(&mut self) -> Option<ItemKind> {
        if let Some(kind) = self.parse_inner_attribute() {
            return Some(ItemKind::InnerAttribute(kind));
        }

        let start_span = self.current_token_span;
        let attributes = self.parse_attributes();

        let modifiers = self.parse_modifiers(
            true, // allow mut
        );

        if self.eat_keyword(Keyword::Use) {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            let use_tree = self.parse_use_tree();
            return Some(ItemKind::Import(use_tree, modifiers.visibility));
        }

        if let Some(is_contract) = self.eat_mod_or_contract() {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            return Some(self.parse_mod_or_contract(attributes, is_contract, modifiers.visibility));
        }

        if self.eat_keyword(Keyword::Struct) {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            return Some(ItemKind::Struct(self.parse_struct(
                attributes,
                modifiers.visibility,
                start_span,
            )));
        }

        if self.eat_keyword(Keyword::Impl) {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            return Some(match self.parse_impl() {
                Impl::Impl(type_impl) => ItemKind::Impl(type_impl),
                Impl::TraitImpl(noir_trait_impl) => ItemKind::TraitImpl(noir_trait_impl),
            });
        }

        if self.eat_keyword(Keyword::Trait) {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            return Some(ItemKind::Trait(self.parse_trait(
                attributes,
                modifiers.visibility,
                start_span,
            )));
        }

        if self.eat_keyword(Keyword::Global) {
            self.unconstrained_not_applicable(modifiers);

            return Some(ItemKind::Global(
                self.parse_global(
                    attributes,
                    modifiers.comptime.is_some(),
                    modifiers.mutable.is_some(),
                ),
                modifiers.visibility,
            ));
        }

        if self.eat_keyword(Keyword::Type) {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            return Some(ItemKind::TypeAlias(
                self.parse_type_alias(modifiers.visibility, start_span),
            ));
        }

        if self.eat_keyword(Keyword::Fn) {
            self.mutable_not_applicable(modifiers);

            return Some(ItemKind::Function(self.parse_function(
                attributes,
                modifiers.visibility,
                modifiers.comptime.is_some(),
                modifiers.unconstrained.is_some(),
                false, // allow_self
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

#[cfg(test)]
mod tests {
    use crate::{
        parse_program,
        parser::parser::tests::{get_single_error, get_source_with_error_span},
    };

    #[test]
    fn recovers_on_unknown_item() {
        let src = "
        fn foo() {} hello fn bar() {}
                    ^^^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let (module, errors) = parse_program(&src);
        assert_eq!(module.items.len(), 2);
        let error = get_single_error(&errors, span);
        assert_eq!(error.to_string(), "Expected an item but found 'hello'");
    }

    #[test]
    fn errors_on_eof_in_nested_mod() {
        let src = "
        mod foo { fn foo() {} 
                             ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let (module, errors) = parse_program(&src);
        assert_eq!(module.items.len(), 1);
        let error = get_single_error(&errors, span);
        assert_eq!(error.to_string(), "Expected a '}' but found end of input");
    }
}
