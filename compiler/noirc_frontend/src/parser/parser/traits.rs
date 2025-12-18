use iter_extended::vecmap;

use noirc_errors::{Located, Location};

use crate::ast::{
    Documented, GenericTypeArg, GenericTypeArgs, ItemVisibility, NoirTrait, Path, Pattern,
    TraitItem, UnresolvedGeneric, UnresolvedTraitConstraint, UnresolvedType,
};
use crate::{
    ast::{Ident, UnresolvedTypeData},
    parser::{NoirTraitImpl, ParserErrorReason, labels::ParsingRuleLabel},
    token::{Attribute, Keyword, SecondaryAttribute, Token},
};

use super::Parser;
use super::parse_many::without_separator;

impl Parser<'_> {
    /// Trait = 'trait' identifier Generics ( ':' TraitBounds )? WhereClause TraitBody
    ///       | 'trait' identifier Generics '=' TraitBounds WhereClause ';'
    pub(crate) fn parse_trait(
        &mut self,
        attributes: Vec<(Attribute, Location)>,
        visibility: ItemVisibility,
        start_location: Location,
    ) -> (NoirTrait, Option<NoirTraitImpl>) {
        let attributes = self.validate_secondary_attributes(attributes);

        let Some(name) = self.eat_ident() else {
            self.expected_identifier();
            let noir_trait =
                empty_trait(attributes, visibility, self.location_since(start_location));
            let no_implicit_impl = None;
            return (noir_trait, no_implicit_impl);
        };

        let generics = self.parse_generics_allowing_trait_bounds();

        // Trait aliases:
        // trait Foo<..> = A + B + E where ..;
        let (bounds, where_clause, items, is_alias) = if self.eat_assign() {
            let bounds = self.parse_trait_bounds();

            if bounds.is_empty() {
                self.push_error(ParserErrorReason::EmptyTraitAlias, self.previous_token_location);
            }

            let where_clause = self.parse_where_clause();
            let items = Vec::new();
            self.eat_semicolon_or_error();

            let is_alias = true;
            (bounds, where_clause, items, is_alias)
        } else {
            let bounds = if self.eat_colon() { self.parse_trait_bounds() } else { Vec::new() };
            let where_clause = self.parse_where_clause();
            let items = self.parse_trait_body();
            let is_alias = false;
            (bounds, where_clause, items, is_alias)
        };

        let location = self.location_since(start_location);

        let noir_impl = is_alias.then(|| {
            let object_type_ident = Ident::from(Located::from(location, "#T".to_string()));
            let object_type_path = Path::from_ident(object_type_ident.clone());
            let object_type_generic = UnresolvedGeneric::from(object_type_ident.clone());

            let is_synthesized = true;
            let object_type = UnresolvedType {
                typ: UnresolvedTypeData::Named(object_type_path, vec![].into(), is_synthesized),
                location,
            };

            let mut impl_generics = generics.clone();
            impl_generics.push(object_type_generic);

            let trait_name = Path::from_ident(name.clone());
            let trait_generics: GenericTypeArgs = vecmap(generics.clone(), |generic| {
                let is_synthesized = true;

                let typ = match generic.ident().ident() {
                    Some(ident) => UnresolvedTypeData::Named(
                        Path::from_ident(ident.clone()),
                        vec![].into(),
                        is_synthesized,
                    ),
                    None => UnresolvedTypeData::Error,
                };

                let generic_type = UnresolvedType { typ, location };

                GenericTypeArg::Ordered(generic_type)
            })
            .into();

            let r#trait = UnresolvedType {
                typ: UnresolvedTypeData::Named(trait_name, trait_generics, false),
                location,
            };

            // bounds from trait
            let mut where_clause = where_clause.clone();
            for bound in bounds.clone() {
                where_clause.push(UnresolvedTraitConstraint {
                    typ: object_type.clone(),
                    trait_bound: bound,
                });
            }

            let items = vec![];
            let is_synthetic = true;

            NoirTraitImpl { impl_generics, r#trait, object_type, where_clause, items, is_synthetic }
        });

        let noir_trait = NoirTrait {
            name,
            generics,
            bounds,
            where_clause,
            location,
            items,
            attributes,
            visibility,
            is_alias,
        };

        (noir_trait, noir_impl)
    }

    /// TraitBody = '{' ( OuterDocComments TraitItem )* '}'
    fn parse_trait_body(&mut self) -> Vec<Documented<TraitItem>> {
        if !self.eat_left_brace() {
            self.expected_token(Token::LeftBrace);
            return Vec::new();
        }

        self.parse_many(
            "trait items",
            without_separator().until(Token::RightBrace),
            Self::parse_trait_item_in_vector,
        )
    }

    fn parse_trait_item_in_vector(&mut self) -> Option<Documented<TraitItem>> {
        self.parse_item_in_vector(ParsingRuleLabel::TraitItem, |parser| {
            let doc_comments = parser.parse_outer_doc_comments();
            parser.parse_trait_item().map(|item| Documented::new(item, doc_comments))
        })
    }

    /// TraitItem
    ///     = TraitType
    ///     | TraitConstant
    ///     | TraitFunction
    fn parse_trait_item(&mut self) -> Option<TraitItem> {
        if let Some(item) = self.parse_trait_type() {
            return Some(item);
        }

        if let Some(item) = self.parse_trait_constant() {
            return Some(item);
        }

        if let Some(item) = self.parse_trait_function() {
            return Some(item);
        }

        None
    }

    /// TraitType = 'type' identifier ( ':' TraitBounds ) ';'
    fn parse_trait_type(&mut self) -> Option<TraitItem> {
        if !self.eat_keyword(Keyword::Type) {
            return None;
        }

        let name = match self.eat_ident() {
            Some(name) => name,
            None => {
                self.expected_identifier();
                self.unknown_ident_at_previous_token_end()
            }
        };

        let bounds = if self.eat_colon() { self.parse_trait_bounds() } else { Vec::new() };

        self.eat_semicolon_or_error();

        Some(TraitItem::Type { name, bounds })
    }

    /// TraitConstant = 'let' identifier ':' Type ( '=' Expression )? ';'
    fn parse_trait_constant(&mut self) -> Option<TraitItem> {
        if !self.eat_keyword(Keyword::Let) {
            return None;
        }

        let name = match self.eat_ident() {
            Some(name) => name,
            None => {
                self.expected_identifier();
                self.unknown_ident_at_previous_token_end()
            }
        };

        let typ = if self.eat_colon() {
            Some(self.parse_type_or_error())
        } else {
            self.push_error(
                ParserErrorReason::MissingTypeForAssociatedConstant,
                self.previous_token_location,
            );
            None
        };

        if self.eat_assign() {
            let expr_start_location = self.current_token_location;
            let _ = self.parse_expression_or_error();
            self.push_error(
                ParserErrorReason::AssociatedTraitConstantDefaultValuesAreNotSupported,
                self.location_since(expr_start_location),
            );
        }

        self.eat_semicolon_or_error();

        Some(TraitItem::Constant { name, typ })
    }

    /// TraitFunction = Modifiers Function
    fn parse_trait_function(&mut self) -> Option<TraitItem> {
        let modifiers = self.parse_modifiers(
            false, // allow mut
        );

        if modifiers.visibility != ItemVisibility::Private {
            self.push_error(
                ParserErrorReason::TraitVisibilityIgnored,
                modifiers.visibility_location,
            );
        }

        if !self.eat_keyword(Keyword::Fn) {
            self.modifiers_not_followed_by_an_item(modifiers);
            return None;
        }

        let function = self.parse_function_definition_with_optional_body(
            true, // allow optional body
            true, // allow self
        );

        let parameters = function
            .parameters
            .into_iter()
            .filter_map(|param| {
                if let Pattern::Identifier(ident) = param.pattern {
                    Some((ident, param.typ))
                } else {
                    self.push_error(ParserErrorReason::InvalidPattern, param.pattern.location());
                    None
                }
            })
            .collect();

        Some(TraitItem::Function {
            is_unconstrained: modifiers.unconstrained.is_some(),
            visibility: modifiers.visibility,
            is_comptime: modifiers.comptime.is_some(),
            name: function.name,
            generics: function.generics,
            parameters,
            return_type: function.return_type,
            where_clause: function.where_clause,
            body: function.body,
        })
    }
}

fn empty_trait(
    attributes: Vec<SecondaryAttribute>,
    visibility: ItemVisibility,
    location: Location,
) -> NoirTrait {
    NoirTrait {
        name: Ident::new(String::new(), location),
        generics: Vec::new(),
        bounds: Vec::new(),
        where_clause: Vec::new(),
        location,
        items: Vec::new(),
        attributes,
        visibility,
        is_alias: false,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{NoirTrait, NoirTraitImpl, TraitItem, UnresolvedTypeData},
        parse_program_with_dummy_file,
        parser::{
            ItemKind,
            parser::{
                ParserErrorReason,
                tests::{expect_no_errors, get_single_error, get_source_with_error_span},
            },
        },
    };

    fn parse_trait_opt_impl_no_errors(src: &str) -> (NoirTrait, Option<NoirTraitImpl>) {
        let (mut module, errors) = parse_program_with_dummy_file(src);
        expect_no_errors(&errors);
        let (item, impl_item) = if module.items.len() == 2 {
            let item = module.items.remove(0);
            let impl_item = module.items.remove(0);
            (item, Some(impl_item))
        } else {
            assert_eq!(module.items.len(), 1);
            let item = module.items.remove(0);
            (item, None)
        };
        let ItemKind::Trait(noir_trait) = item.kind else {
            panic!("Expected trait");
        };
        let noir_trait_impl = impl_item.map(|impl_item| {
            let ItemKind::TraitImpl(noir_trait_impl) = impl_item.kind else {
                panic!("Expected impl");
            };
            noir_trait_impl
        });
        (noir_trait, noir_trait_impl)
    }

    fn parse_trait_with_impl_no_errors(src: &str) -> (NoirTrait, NoirTraitImpl) {
        let (noir_trait, noir_trait_impl) = parse_trait_opt_impl_no_errors(src);
        (noir_trait, noir_trait_impl.expect("expected a NoirTraitImpl"))
    }

    fn parse_trait_no_errors(src: &str) -> NoirTrait {
        parse_trait_opt_impl_no_errors(src).0
    }

    #[test]
    fn parse_empty_trait() {
        let src = "trait Foo {}";
        let noir_trait = parse_trait_no_errors(src);
        assert_eq!(noir_trait.name.to_string(), "Foo");
        assert!(noir_trait.generics.is_empty());
        assert!(noir_trait.where_clause.is_empty());
        assert!(noir_trait.items.is_empty());
        assert!(!noir_trait.is_alias);
    }

    #[test]
    fn parse_empty_trait_alias() {
        let src = "trait Foo = ;";
        let (_module, errors) = parse_program_with_dummy_file(src);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[1].reason(), Some(ParserErrorReason::EmptyTraitAlias).as_ref());
    }

    #[test]
    fn parse_trait_with_generics() {
        let src = "trait Foo<A, B> {}";
        let noir_trait = parse_trait_no_errors(src);
        assert_eq!(noir_trait.name.to_string(), "Foo");
        assert_eq!(noir_trait.generics.len(), 2);
        assert!(noir_trait.where_clause.is_empty());
        assert!(noir_trait.items.is_empty());
        assert!(!noir_trait.is_alias);
    }

    #[test]
    fn parse_trait_alias_with_generics() {
        let src = "trait Foo<A, B> = Bar + Baz<A>;";
        let (noir_trait_alias, noir_trait_impl) = parse_trait_with_impl_no_errors(src);
        assert_eq!(noir_trait_alias.name.to_string(), "Foo");
        assert_eq!(noir_trait_alias.generics.len(), 2);
        assert_eq!(noir_trait_alias.bounds.len(), 2);
        assert_eq!(noir_trait_alias.bounds[0].to_string(), "Bar");
        assert_eq!(noir_trait_alias.bounds[1].to_string(), "Baz<A>");
        assert!(noir_trait_alias.where_clause.is_empty());
        assert!(noir_trait_alias.items.is_empty());
        assert!(noir_trait_alias.is_alias);

        let UnresolvedTypeData::Named(trait_name, trait_generics, _) = noir_trait_impl.r#trait.typ
        else {
            panic!("Expected name type");
        };

        assert_eq!(trait_name.to_string(), "Foo");
        assert_eq!(noir_trait_impl.impl_generics.len(), 3);
        assert_eq!(trait_generics.ordered_args.len(), 2);
        assert_eq!(noir_trait_impl.where_clause.len(), 2);
        assert_eq!(noir_trait_alias.bounds.len(), 2);
        assert_eq!(noir_trait_alias.bounds[0].to_string(), "Bar");
        assert_eq!(noir_trait_alias.bounds[1].to_string(), "Baz<A>");
        assert!(noir_trait_impl.items.is_empty());
        assert!(noir_trait_impl.is_synthetic);

        // Equivalent to
        let src = "trait Foo<A, B>: Bar + Baz<A> {}";
        let noir_trait = parse_trait_no_errors(src);
        assert_eq!(noir_trait.name.to_string(), noir_trait_alias.name.to_string());
        assert_eq!(noir_trait.generics.len(), noir_trait_alias.generics.len());
        assert_eq!(noir_trait.bounds.len(), noir_trait_alias.bounds.len());
        assert_eq!(noir_trait.bounds[0].to_string(), noir_trait_alias.bounds[0].to_string());
        assert_eq!(noir_trait.where_clause.is_empty(), noir_trait_alias.where_clause.is_empty());
        assert_eq!(noir_trait.items.is_empty(), noir_trait_alias.items.is_empty());
        assert!(!noir_trait.is_alias);
    }

    #[test]
    fn parse_empty_trait_alias_with_generics() {
        let src = "trait Foo<A, B> = ;";
        let (_module, errors) = parse_program_with_dummy_file(src);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[1].reason(), Some(ParserErrorReason::EmptyTraitAlias).as_ref());
    }

    #[test]
    fn parse_trait_with_where_clause() {
        let src = "trait Foo<A, B> where A: Z {}";
        let noir_trait = parse_trait_no_errors(src);
        assert_eq!(noir_trait.name.to_string(), "Foo");
        assert_eq!(noir_trait.generics.len(), 2);
        assert_eq!(noir_trait.where_clause.len(), 1);
        assert!(noir_trait.items.is_empty());
        assert!(!noir_trait.is_alias);
    }

    #[test]
    fn parse_trait_alias_with_where_clause() {
        let src = "trait Foo<A, B> = Bar + Baz<A> where A: Z;";
        let (noir_trait_alias, noir_trait_impl) = parse_trait_with_impl_no_errors(src);
        assert_eq!(noir_trait_alias.name.to_string(), "Foo");
        assert_eq!(noir_trait_alias.generics.len(), 2);
        assert_eq!(noir_trait_alias.bounds.len(), 2);
        assert_eq!(noir_trait_alias.bounds[0].to_string(), "Bar");
        assert_eq!(noir_trait_alias.bounds[1].to_string(), "Baz<A>");
        assert_eq!(noir_trait_alias.where_clause.len(), 1);
        assert!(noir_trait_alias.items.is_empty());
        assert!(noir_trait_alias.is_alias);

        let UnresolvedTypeData::Named(trait_name, trait_generics, _) = noir_trait_impl.r#trait.typ
        else {
            panic!("Expected name type");
        };

        assert_eq!(trait_name.to_string(), "Foo");
        assert_eq!(noir_trait_impl.impl_generics.len(), 3);
        assert_eq!(trait_generics.ordered_args.len(), 2);
        assert_eq!(noir_trait_impl.where_clause.len(), 3);
        assert_eq!(noir_trait_impl.where_clause[0].to_string(), "A: Z");
        assert_eq!(noir_trait_impl.where_clause[1].to_string(), "#T: Bar");
        assert_eq!(noir_trait_impl.where_clause[2].to_string(), "#T: Baz<A>");
        assert!(noir_trait_impl.items.is_empty());
        assert!(noir_trait_impl.is_synthetic);

        // Equivalent to
        let src = "trait Foo<A, B>: Bar + Baz<A> where A: Z {}";
        let noir_trait = parse_trait_no_errors(src);
        assert_eq!(noir_trait.name.to_string(), noir_trait_alias.name.to_string());
        assert_eq!(noir_trait.generics.len(), noir_trait_alias.generics.len());
        assert_eq!(noir_trait.bounds.len(), noir_trait_alias.bounds.len());
        assert_eq!(noir_trait.bounds[0].to_string(), noir_trait_alias.bounds[0].to_string());
        assert_eq!(noir_trait.where_clause.len(), noir_trait_alias.where_clause.len());
        assert_eq!(
            noir_trait.where_clause[0].to_string(),
            noir_trait_alias.where_clause[0].to_string()
        );
        assert_eq!(noir_trait.items.is_empty(), noir_trait_alias.items.is_empty());
        assert!(!noir_trait.is_alias);
    }

    #[test]
    fn parse_empty_trait_alias_with_where_clause() {
        let src = "trait Foo<A, B> = where A: Z;";
        let (_module, errors) = parse_program_with_dummy_file(src);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[1].reason(), Some(ParserErrorReason::EmptyTraitAlias).as_ref());
    }

    #[test]
    fn parse_trait_with_type() {
        let src = "trait Foo { type Elem; }";
        let mut noir_trait = parse_trait_no_errors(src);
        assert_eq!(noir_trait.items.len(), 1);

        let item = noir_trait.items.remove(0).item;
        let TraitItem::Type { name, bounds } = item else {
            panic!("Expected type");
        };
        assert_eq!(name.to_string(), "Elem");
        assert!(!noir_trait.is_alias);
        assert!(bounds.is_empty());
    }

    #[test]
    fn parse_trait_with_type_and_bounds() {
        let src = "trait Foo { type Elem: Bound; }";
        let mut noir_trait = parse_trait_no_errors(src);
        assert_eq!(noir_trait.items.len(), 1);

        let item = noir_trait.items.remove(0).item;
        let TraitItem::Type { name, bounds } = item else {
            panic!("Expected type");
        };
        assert_eq!(name.to_string(), "Elem");
        assert!(!noir_trait.is_alias);
        assert!(bounds.len() == 1);
        assert_eq!(bounds[0].to_string(), "Bound");
    }

    #[test]
    fn parse_trait_with_constant() {
        let src = "trait Foo { let x: Field; }";
        let mut noir_trait = parse_trait_no_errors(src);
        assert_eq!(noir_trait.items.len(), 1);

        let item = noir_trait.items.remove(0).item;
        let TraitItem::Constant { name, typ } = item else {
            panic!("Expected constant");
        };
        assert_eq!(name.to_string(), "x");
        assert_eq!(typ.unwrap().to_string(), "Field");
        assert!(!noir_trait.is_alias);
    }

    #[test]
    fn parse_trait_with_constant_default_value() {
        let src = "
        trait Foo { let x: Field = 1 + 2; }
                                   ^^^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let (_module, errors) = parse_program_with_dummy_file(&src);
        let error = get_single_error(&errors, span).to_string();
        assert!(error.contains("Associated trait constant default values are not supported"));
    }

    #[test]
    fn parse_trait_with_function_no_body() {
        let src = "trait Foo { fn foo(); }";
        let mut noir_trait = parse_trait_no_errors(src);
        assert_eq!(noir_trait.items.len(), 1);

        let item = noir_trait.items.remove(0).item;
        let TraitItem::Function { body, .. } = item else {
            panic!("Expected function");
        };
        assert!(body.is_none());
        assert!(!noir_trait.is_alias);
    }

    #[test]
    fn parse_trait_with_function_with_body() {
        let src = "trait Foo { fn foo() {} }";
        let mut noir_trait = parse_trait_no_errors(src);
        assert_eq!(noir_trait.items.len(), 1);

        let item = noir_trait.items.remove(0).item;
        let TraitItem::Function { body, .. } = item else {
            panic!("Expected function");
        };
        assert!(body.is_some());
        assert!(!noir_trait.is_alias);
    }

    #[test]
    fn parse_trait_function_with_visibility() {
        let src = "
        trait Foo { pub fn foo(); }
                    ^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let (_module, errors) = parse_program_with_dummy_file(&src);
        let error = get_single_error(&errors, span);
        assert!(error.to_string().contains("Visibility is ignored on a trait method"));
    }

    #[test]
    fn parse_trait_inheritance() {
        let src = "trait Foo: Bar + Baz {}";
        let noir_trait = parse_trait_no_errors(src);
        assert_eq!(noir_trait.bounds.len(), 2);

        assert_eq!(noir_trait.bounds[0].to_string(), "Bar");
        assert_eq!(noir_trait.bounds[1].to_string(), "Baz");

        assert_eq!(noir_trait.to_string(), "trait Foo: Bar + Baz {\n}");
        assert!(!noir_trait.is_alias);
    }

    #[test]
    fn parse_trait_alias() {
        let src = "trait Foo = Bar + Baz;";
        let (noir_trait_alias, noir_trait_impl) = parse_trait_with_impl_no_errors(src);
        assert_eq!(noir_trait_alias.bounds.len(), 2);

        assert_eq!(noir_trait_alias.bounds[0].to_string(), "Bar");
        assert_eq!(noir_trait_alias.bounds[1].to_string(), "Baz");

        assert_eq!(noir_trait_alias.to_string(), "trait Foo = Bar + Baz;");
        assert!(noir_trait_alias.is_alias);

        let UnresolvedTypeData::Named(trait_name, trait_generics, _) = noir_trait_impl.r#trait.typ
        else {
            panic!("Expected name type");
        };

        assert_eq!(trait_name.to_string(), "Foo");
        assert_eq!(noir_trait_impl.impl_generics.len(), 1);
        assert_eq!(trait_generics.ordered_args.len(), 0);
        assert_eq!(noir_trait_impl.where_clause.len(), 2);
        assert_eq!(noir_trait_impl.where_clause[0].to_string(), "#T: Bar");
        assert_eq!(noir_trait_impl.where_clause[1].to_string(), "#T: Baz");
        assert!(noir_trait_impl.items.is_empty());
        assert!(noir_trait_impl.is_synthetic);

        // Equivalent to
        let src = "trait Foo: Bar + Baz {}";
        let noir_trait = parse_trait_no_errors(src);
        assert_eq!(noir_trait.name.to_string(), noir_trait_alias.name.to_string());
        assert_eq!(noir_trait.generics.len(), noir_trait_alias.generics.len());
        assert_eq!(noir_trait.bounds.len(), noir_trait_alias.bounds.len());
        assert_eq!(noir_trait.bounds[0].to_string(), noir_trait_alias.bounds[0].to_string());
        assert_eq!(noir_trait.where_clause.is_empty(), noir_trait_alias.where_clause.is_empty());
        assert_eq!(noir_trait.items.is_empty(), noir_trait_alias.items.is_empty());
        assert!(!noir_trait.is_alias);
    }

    #[test]
    fn parse_trait_with_constant_missing_semicolon() {
        let src = "trait Foo { let x: Field = 1 }";
        let (_, errors) = parse_program_with_dummy_file(src);
        assert!(!errors.is_empty());
    }

    #[test]
    fn parse_trait_with_constant_missing_type() {
        let src = "trait Foo { let x = 1; }";
        let (_, errors) = parse_program_with_dummy_file(src);
        assert!(!errors.is_empty());
    }

    #[test]
    fn parse_trait_with_type_missing_semicolon() {
        let src = "trait Foo { type X }";
        let (_, errors) = parse_program_with_dummy_file(src);
        assert!(!errors.is_empty());
    }
}
