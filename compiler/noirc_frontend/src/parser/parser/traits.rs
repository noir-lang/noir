use iter_extended::vecmap;

use noirc_errors::Span;

use crate::ast::{
    Documented, GenericTypeArg, GenericTypeArgs, ItemVisibility, NoirTrait, Path, Pattern,
    TraitItem, UnresolvedGeneric, UnresolvedTraitConstraint, UnresolvedType,
};
use crate::{
    ast::{Ident, UnresolvedTypeData},
    parser::{labels::ParsingRuleLabel, NoirTraitImpl, ParserErrorReason},
    token::{Attribute, Keyword, SecondaryAttribute, Token},
};

use super::parse_many::without_separator;
use super::Parser;

impl<'a> Parser<'a> {
    /// Trait = 'trait' identifier Generics ( ':' TraitBounds )? WhereClause TraitBody
    ///       | 'trait' identifier Generics '=' TraitBounds WhereClause ';'
    pub(crate) fn parse_trait(
        &mut self,
        attributes: Vec<(Attribute, Span)>,
        visibility: ItemVisibility,
        start_span: Span,
    ) -> (NoirTrait, Option<NoirTraitImpl>) {
        let attributes = self.validate_secondary_attributes(attributes);

        let Some(name) = self.eat_ident() else {
            self.expected_identifier();
            let noir_trait = empty_trait(attributes, visibility, self.span_since(start_span));
            let no_implicit_impl = None;
            return (noir_trait, no_implicit_impl);
        };

        let generics = self.parse_generics();

        // Trait aliases:
        // trait Foo<..> = A + B + E where ..;
        let (bounds, where_clause, items, is_alias) = if self.eat_assign() {
            let bounds = self.parse_trait_bounds();

            if bounds.is_empty() {
                self.push_error(ParserErrorReason::EmptyTraitAlias, self.previous_token_span);
            }

            let where_clause = self.parse_where_clause();
            let items = Vec::new();
            if !self.eat_semicolon() {
                self.expected_token(Token::Semicolon);
            }

            let is_alias = true;
            (bounds, where_clause, items, is_alias)
        } else {
            let bounds = if self.eat_colon() { self.parse_trait_bounds() } else { Vec::new() };
            let where_clause = self.parse_where_clause();
            let items = self.parse_trait_body();
            let is_alias = false;
            (bounds, where_clause, items, is_alias)
        };

        let span = self.span_since(start_span);

        let noir_impl = is_alias.then(|| {
            let object_type_ident = Ident::new("#T".to_string(), span);
            let object_type_path = Path::from_ident(object_type_ident.clone());
            let object_type_generic = UnresolvedGeneric::Variable(object_type_ident);

            let is_synthesized = true;
            let object_type = UnresolvedType {
                typ: UnresolvedTypeData::Named(object_type_path, vec![].into(), is_synthesized),
                span,
            };

            let mut impl_generics = generics.clone();
            impl_generics.push(object_type_generic);

            let trait_name = Path::from_ident(name.clone());
            let trait_generics: GenericTypeArgs = vecmap(generics.clone(), |generic| {
                let is_synthesized = true;
                let generic_type = UnresolvedType {
                    typ: UnresolvedTypeData::Named(
                        Path::from_ident(generic.ident().clone()),
                        vec![].into(),
                        is_synthesized,
                    ),
                    span,
                };

                GenericTypeArg::Ordered(generic_type)
            })
            .into();

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

            NoirTraitImpl {
                impl_generics,
                trait_name,
                trait_generics,
                object_type,
                where_clause,
                items,
                is_synthetic,
            }
        });

        let noir_trait = NoirTrait {
            name,
            generics,
            bounds,
            where_clause,
            span,
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
            Self::parse_trait_item_in_list,
        )
    }

    fn parse_trait_item_in_list(&mut self) -> Option<Documented<TraitItem>> {
        self.parse_item_in_list(ParsingRuleLabel::TraitItem, |parser| {
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

    /// TraitType = 'type' identifier ';'
    fn parse_trait_type(&mut self) -> Option<TraitItem> {
        if !self.eat_keyword(Keyword::Type) {
            return None;
        }

        let name = match self.eat_ident() {
            Some(name) => name,
            None => {
                self.expected_identifier();
                Ident::default()
            }
        };

        self.eat_semicolons();

        Some(TraitItem::Type { name })
    }

    /// TraitConstant = 'let' identifier ':' Type ( '=' Expression ) ';'
    fn parse_trait_constant(&mut self) -> Option<TraitItem> {
        if !self.eat_keyword(Keyword::Let) {
            return None;
        }

        let name = match self.eat_ident() {
            Some(name) => name,
            None => {
                self.expected_identifier();
                Ident::default()
            }
        };

        let typ = if self.eat_colon() {
            self.parse_type_or_error()
        } else {
            self.expected_token(Token::Colon);
            UnresolvedType { typ: UnresolvedTypeData::Unspecified, span: Span::default() }
        };

        let default_value =
            if self.eat_assign() { Some(self.parse_expression_or_error()) } else { None };

        self.eat_semicolons();

        Some(TraitItem::Constant { name, typ, default_value })
    }

    /// TraitFunction = Modifiers Function
    fn parse_trait_function(&mut self) -> Option<TraitItem> {
        let modifiers = self.parse_modifiers(
            false, // allow mut
        );

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
                    self.push_error(ParserErrorReason::InvalidPattern, param.pattern.span());
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
    span: Span,
) -> NoirTrait {
    NoirTrait {
        name: Ident::default(),
        generics: Vec::new(),
        bounds: Vec::new(),
        where_clause: Vec::new(),
        span,
        items: Vec::new(),
        attributes,
        visibility,
        is_alias: false,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{NoirTrait, NoirTraitImpl, TraitItem},
        parser::{
            parser::{parse_program, tests::expect_no_errors, ParserErrorReason},
            ItemKind,
        },
    };

    fn parse_trait_opt_impl_no_errors(src: &str) -> (NoirTrait, Option<NoirTraitImpl>) {
        let (mut module, errors) = parse_program(src);
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
        let (_module, errors) = parse_program(src);
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

        assert_eq!(noir_trait_impl.trait_name.to_string(), "Foo");
        assert_eq!(noir_trait_impl.impl_generics.len(), 3);
        assert_eq!(noir_trait_impl.trait_generics.ordered_args.len(), 2);
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
        let (_module, errors) = parse_program(src);
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

        assert_eq!(noir_trait_impl.trait_name.to_string(), "Foo");
        assert_eq!(noir_trait_impl.impl_generics.len(), 3);
        assert_eq!(noir_trait_impl.trait_generics.ordered_args.len(), 2);
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
        let (_module, errors) = parse_program(src);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[1].reason(), Some(ParserErrorReason::EmptyTraitAlias).as_ref());
    }

    #[test]
    fn parse_trait_with_type() {
        let src = "trait Foo { type Elem; }";
        let mut noir_trait = parse_trait_no_errors(src);
        assert_eq!(noir_trait.items.len(), 1);

        let item = noir_trait.items.remove(0).item;
        let TraitItem::Type { name } = item else {
            panic!("Expected type");
        };
        assert_eq!(name.to_string(), "Elem");
        assert!(!noir_trait.is_alias);
    }

    #[test]
    fn parse_trait_with_constant() {
        let src = "trait Foo { let x: Field = 1; }";
        let mut noir_trait = parse_trait_no_errors(src);
        assert_eq!(noir_trait.items.len(), 1);

        let item = noir_trait.items.remove(0).item;
        let TraitItem::Constant { name, typ, default_value } = item else {
            panic!("Expected constant");
        };
        assert_eq!(name.to_string(), "x");
        assert_eq!(typ.to_string(), "Field");
        assert_eq!(default_value.unwrap().to_string(), "1");
        assert!(!noir_trait.is_alias);
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
    fn parse_trait_inheirtance() {
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

        assert_eq!(noir_trait_impl.trait_name.to_string(), "Foo");
        assert_eq!(noir_trait_impl.impl_generics.len(), 1);
        assert_eq!(noir_trait_impl.trait_generics.ordered_args.len(), 0);
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
}
