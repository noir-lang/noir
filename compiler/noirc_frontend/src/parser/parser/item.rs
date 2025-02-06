use iter_extended::vecmap;

use crate::{
    parser::{labels::ParsingRuleLabel, Item, ItemKind, ParserErrorReason},
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
        self.parse_many_to_many("items", without_separator(), |parser| {
            parser.parse_module_item_in_list(nested)
        })
    }

    fn parse_module_item_in_list(&mut self, nested: bool) -> Vec<Item> {
        loop {
            // We only break out of the loop on `}` if we are inside a `mod { ..`
            if nested && self.at(Token::RightBrace) {
                return vec![];
            }

            // We always break on EOF (we don't error because if we are inside `mod { ..`
            // the outer parsing logic will error instead)
            if self.at_eof() {
                return vec![];
            }

            let parsed_items = self.parse_item();
            if parsed_items.is_empty() {
                // If we couldn't parse an item we check which token we got
                match self.token.token() {
                    Token::RightBrace if nested => {
                        return vec![];
                    }
                    Token::EOF => return vec![],
                    _ => (),
                }

                self.expected_label(ParsingRuleLabel::Item);
                // We'll try parsing an item starting on the next token
                self.bump();
                continue;
            };

            return parsed_items;
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
    fn parse_item(&mut self) -> Vec<Item> {
        let start_span = self.current_token_span;
        let doc_comments = self.parse_outer_doc_comments();
        let kinds = self.parse_item_kind();
        let span = self.span_since(start_span);

        if kinds.is_empty() && !doc_comments.is_empty() {
            self.push_error(ParserErrorReason::DocCommentDoesNotDocumentAnything, start_span);
        }

        vecmap(kinds, |(kind, cfg_feature_disabled)| Item { kind, span, doc_comments: doc_comments.clone(), cfg_feature_disabled })
    }

    /// This method returns one 'ItemKind' in the majority of cases.
    /// The current exception is when parsing a trait alias,
    /// which returns both the trait and the impl.
    ///
    /// ItemKind
    ///     = InnerAttribute
    ///     | Attributes Modifiers
    ///         ( Use
    ///         | ModOrContract
    ///         | Struct
    ///         | Enum
    ///         | Impl
    ///         | Trait
    ///         | Global
    ///         | TypeAlias
    ///         | Function
    ///         )
    fn parse_item_kind(&mut self) -> Vec<(ItemKind, bool)> {
        if let Some(kind) = self.parse_inner_attribute() {
            return vec![(ItemKind::InnerAttribute(kind), false)];
        }

        let start_span = self.current_token_span;
        let attributes = self.parse_attributes();
        let cfg_feature_disabled = attributes.iter().any(|attribute| attribute.0.is_disabled_cfg());

        let modifiers = self.parse_modifiers(
            true, // allow mut
        );

        if self.eat_keyword(Keyword::Use) {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            let use_tree = self.parse_use_tree();
            return vec![(ItemKind::Import(use_tree, modifiers.visibility), cfg_feature_disabled)];
        }

        if let Some(is_contract) = self.eat_mod_or_contract() {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            let parsed_mod_or_contract = self.parse_mod_or_contract(attributes, is_contract, modifiers.visibility);
            return vec![(parsed_mod_or_contract, cfg_feature_disabled)];
        }

        if self.eat_keyword(Keyword::Struct) {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            let parsed_struct = self.parse_struct(
                attributes,
                modifiers.visibility,
                start_span,
            );
            return vec![(ItemKind::Struct(parsed_struct), cfg_feature_disabled)];
        }

        if self.eat_keyword(Keyword::Enum) {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            let parsed_enum = self.parse_enum(
                attributes,
                modifiers.visibility,
                start_span,
            );
            return vec![(ItemKind::Enum(parsed_enum), cfg_feature_disabled)];
        }

        if self.eat_keyword(Keyword::Impl) {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            let parsed_impl = self.parse_impl();
            return vec![(match parsed_impl {
                Impl::Impl(type_impl) => ItemKind::Impl(type_impl),
                Impl::TraitImpl(noir_trait_impl) => ItemKind::TraitImpl(noir_trait_impl),
            }, cfg_feature_disabled)];
        }

        if self.eat_keyword(Keyword::Trait) {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            let (noir_trait, noir_impl) =
                self.parse_trait(attributes, modifiers.visibility, start_span);
            let mut output = vec![(ItemKind::Trait(noir_trait), cfg_feature_disabled)];
            if let Some(noir_impl) = noir_impl {
                output.push((ItemKind::TraitImpl(noir_impl), cfg_feature_disabled));
            }

            return output;
        }

        if self.eat_keyword(Keyword::Global) {
            self.unconstrained_not_applicable(modifiers);

            let parsed_global = self.parse_global(
                attributes,
                modifiers.comptime.is_some(),
                modifiers.mutable.is_some(),
            );
            return vec![(ItemKind::Global(
                parsed_global,
                modifiers.visibility,
            ), cfg_feature_disabled)];
        }

        if self.eat_keyword(Keyword::Type) {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            let parsed_type_alias = self.parse_type_alias(modifiers.visibility, start_span);
            return vec![(ItemKind::TypeAlias(
                parsed_type_alias
            ), cfg_feature_disabled)];
        }

        if self.eat_keyword(Keyword::Fn) {
            self.mutable_not_applicable(modifiers);

            let parsed_function = self.parse_function(
                attributes,
                modifiers.visibility,
                modifiers.comptime.is_some(),
                modifiers.unconstrained.is_some(),
                false, // allow_self
            );
            return vec![(ItemKind::Function(parsed_function), cfg_feature_disabled)];
        }

        vec![]
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

    #[test]
    fn errors_on_trailing_doc_comment() {
        let src = "
        fn foo() {}
        /// doc comment
        ^^^^^^^^^^^^^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let (module, errors) = parse_program(&src);
        assert_eq!(module.items.len(), 1);
        let error = get_single_error(&errors, span);
        assert!(error.to_string().contains("Documentation comment does not document anything"));
    }

    // TODO: rename and consider relocating to more specific test location
    #[test]
    fn no_error_on_disabled_cfg_before_main() {
        let src = r#"
        #[cfg(feature = "foo")]
        use foo_module::FOO;

        fn main() { }
        "#;
        let (module, errors) = parse_program(&src);

        // TODO cleanup
        dbg!(&module, &errors);
        assert_eq!(module.items.len(), 1);
        assert_eq!(errors, vec![]);
    }
}
