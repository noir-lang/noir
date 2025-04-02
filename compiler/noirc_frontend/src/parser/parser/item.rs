use iter_extended::vecmap;
use noirc_errors::Location;

use crate::{
    parser::{Item, ItemKind, ParserErrorReason, labels::ParsingRuleLabel},
    token::{Attribute, Keyword, Token},
};

use super::{Parser, impls::Impl, parse_many::without_separator};

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

    /// Item = ( Attribute | OuterDocComments )* ItemKind
    fn parse_item(&mut self) -> Vec<Item> {
        let start_location = self.current_token_location;

        // Attributes and doc comments can come in any order, and can even be interspersed
        let mut doc_comments = Vec::new();
        let mut attributes = Vec::new();
        loop {
            if let Some(doc_comment) = self.parse_outer_doc_comment() {
                doc_comments.push(doc_comment);
            } else if let Some(attribute) = self.parse_attribute() {
                attributes.push(attribute);
            } else {
                break;
            }
        }

        let kinds = self.parse_item_kind(attributes);
        let location = self.location_since(start_location);

        if kinds.is_empty() && !doc_comments.is_empty() {
            self.push_error(ParserErrorReason::DocCommentDoesNotDocumentAnything, start_location);
        }

        vecmap(kinds, |kind| Item { kind, location, doc_comments: doc_comments.clone() })
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
    fn parse_item_kind(&mut self, attributes: Vec<(Attribute, Location)>) -> Vec<ItemKind> {
        if let Some(kind) = self.parse_inner_attribute() {
            return vec![ItemKind::InnerAttribute(kind)];
        }

        let start_location = self.current_token_location;

        let modifiers = self.parse_modifiers(
            true, // allow mut
        );

        if self.eat_keyword(Keyword::Use) {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            let use_tree = self.parse_use_tree();
            return vec![ItemKind::Import(use_tree, modifiers.visibility)];
        }

        if let Some(is_contract) = self.eat_mod_or_contract() {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            return vec![self.parse_mod_or_contract(attributes, is_contract, modifiers.visibility)];
        }

        if self.eat_keyword(Keyword::Struct) {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            return vec![ItemKind::Struct(self.parse_struct(
                attributes,
                modifiers.visibility,
                start_location,
            ))];
        }

        if self.eat_keyword(Keyword::Enum) {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            return vec![ItemKind::Enum(self.parse_enum(
                attributes,
                modifiers.visibility,
                start_location,
            ))];
        }

        if self.eat_keyword(Keyword::Impl) {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            return vec![match self.parse_impl() {
                Impl::Impl(type_impl) => ItemKind::Impl(type_impl),
                Impl::TraitImpl(noir_trait_impl) => ItemKind::TraitImpl(noir_trait_impl),
            }];
        }

        if self.eat_keyword(Keyword::Trait) {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            let (noir_trait, noir_impl) =
                self.parse_trait(attributes, modifiers.visibility, start_location);
            let mut output = vec![ItemKind::Trait(noir_trait)];
            if let Some(noir_impl) = noir_impl {
                output.push(ItemKind::TraitImpl(noir_impl));
            }

            return output;
        }

        if self.eat_keyword(Keyword::Global) {
            self.unconstrained_not_applicable(modifiers);

            return vec![ItemKind::Global(
                self.parse_global(
                    attributes,
                    modifiers.comptime.is_some(),
                    modifiers.mutable.is_some(),
                ),
                modifiers.visibility,
            )];
        }

        if self.eat_keyword(Keyword::Type) {
            self.comptime_mutable_and_unconstrained_not_applicable(modifiers);

            return vec![ItemKind::TypeAlias(
                self.parse_type_alias(modifiers.visibility, start_location),
            )];
        }

        let is_function = if self.eat_keyword(Keyword::Fn) {
            true
        } else if !modifiers.is_empty()
            && matches!(self.token.token(), Token::Ident(..))
            && self.next_is(Token::LeftParen)
        {
            // If it's something like `pub foo(` then it's likely the user forgot to put `fn` after `pub`,
            // so we error but keep parsing what comes next as a function.
            self.expected_token(Token::Keyword(Keyword::Fn));
            true
        } else {
            false
        };

        if is_function {
            self.mutable_not_applicable(modifiers);

            return vec![ItemKind::Function(self.parse_function(
                attributes,
                modifiers.visibility,
                modifiers.comptime.is_some(),
                modifiers.unconstrained.is_some(),
                false, // allow_self
            ))];
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
        parse_program_with_dummy_file,
        parser::{
            ItemKind, Parser,
            parser::tests::{get_single_error, get_source_with_error_span},
        },
    };

    #[test]
    fn recovers_on_unknown_item() {
        let src = "
        fn foo() {} hello fn bar() {}
                    ^^^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let (module, errors) = parse_program_with_dummy_file(&src);
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
        let (module, errors) = parse_program_with_dummy_file(&src);
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
        let (module, errors) = parse_program_with_dummy_file(&src);
        assert_eq!(module.items.len(), 1);
        let error = get_single_error(&errors, span);
        assert!(error.to_string().contains("This doc comment doesn't document anything"));
    }

    #[test]
    fn parse_item_with_mixed_attributes_and_doc_comments() {
        let src = "
        /// One
        #[one]
        /// Two
        #[two]
        /// Three
        fn foo() {}
        ";

        let (module, errors) = parse_program_with_dummy_file(src);
        assert!(errors.is_empty());

        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        assert_eq!(
            item.doc_comments,
            vec![" One".to_string(), " Two".to_string(), " Three".to_string(),]
        );
        let ItemKind::Function(func) = &item.kind else {
            panic!("Expected function");
        };
        let attributes = &func.attributes().secondary;
        assert_eq!(attributes.len(), 2);
        assert_eq!(attributes[0].to_string(), "#[one]");
        assert_eq!(attributes[1].to_string(), "#[two]");
    }

    #[test]
    fn error_recovery_for_missing_fn_between_visibility_and_name() {
        let src = "
        pub foo() { }
            ^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str_with_dummy_file(&src);
        let module = parser.parse_program();
        assert_eq!(module.items.len(), 1);
        let ItemKind::Function(noir_function) = &module.items[0].kind else {
            panic!("Expected function");
        };
        assert_eq!(noir_function.name(), "foo");

        let reason = get_single_error(&parser.errors, span);
        assert_eq!(reason.to_string(), "Expected a 'fn' but found 'foo'");
    }

    #[test]
    fn error_recovery_for_missing_fn_between_unconstrained_and_name() {
        let src = "
        unconstrained foo() { }
                      ^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str_with_dummy_file(&src);
        let module = parser.parse_program();
        assert_eq!(module.items.len(), 1);
        let ItemKind::Function(noir_function) = &module.items[0].kind else {
            panic!("Expected function");
        };
        assert_eq!(noir_function.name(), "foo");

        let reason = get_single_error(&parser.errors, span);
        assert_eq!(reason.to_string(), "Expected a 'fn' but found 'foo'");
    }
}
