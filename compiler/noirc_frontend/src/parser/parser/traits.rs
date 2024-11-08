use noirc_errors::Span;

use crate::ast::{Documented, ItemVisibility, NoirTrait, Pattern, TraitItem, UnresolvedType};
use crate::{
    ast::{Ident, UnresolvedTypeData},
    parser::{labels::ParsingRuleLabel, ParserErrorReason},
    token::{Attribute, Keyword, SecondaryAttribute, Token},
};

use super::parse_many::without_separator;
use super::Parser;

impl<'a> Parser<'a> {
    /// Trait = 'trait' identifier Generics ( ':' TraitBounds )? WhereClause TraitBody
    pub(crate) fn parse_trait(
        &mut self,
        attributes: Vec<(Attribute, Span)>,
        visibility: ItemVisibility,
        start_span: Span,
    ) -> NoirTrait {
        let attributes = self.validate_secondary_attributes(attributes);

        let Some(name) = self.eat_ident() else {
            self.expected_identifier();
            return empty_trait(attributes, visibility, self.span_since(start_span));
        };

        let generics = self.parse_generics();
        let bounds = if self.eat_colon() { self.parse_trait_bounds() } else { Vec::new() };
        let where_clause = self.parse_where_clause();
        let items = self.parse_trait_body();

        NoirTrait {
            name,
            generics,
            bounds,
            where_clause,
            span: self.span_since(start_span),
            items,
            attributes,
            visibility,
        }
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
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{NoirTrait, TraitItem},
        parser::{
            parser::{parse_program, tests::expect_no_errors},
            ItemKind,
        },
    };

    fn parse_trait_no_errors(src: &str) -> NoirTrait {
        let (mut module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Trait(noir_trait) = item.kind else {
            panic!("Expected trait");
        };
        noir_trait
    }

    #[test]
    fn parse_empty_trait() {
        let src = "trait Foo {}";
        let noir_trait = parse_trait_no_errors(src);
        assert_eq!(noir_trait.name.to_string(), "Foo");
        assert!(noir_trait.generics.is_empty());
        assert!(noir_trait.where_clause.is_empty());
        assert!(noir_trait.items.is_empty());
    }

    #[test]
    fn parse_trait_with_generics() {
        let src = "trait Foo<A, B> {}";
        let noir_trait = parse_trait_no_errors(src);
        assert_eq!(noir_trait.name.to_string(), "Foo");
        assert_eq!(noir_trait.generics.len(), 2);
        assert!(noir_trait.where_clause.is_empty());
        assert!(noir_trait.items.is_empty());
    }

    #[test]
    fn parse_trait_with_where_clause() {
        let src = "trait Foo<A, B> where A: Z {}";
        let noir_trait = parse_trait_no_errors(src);
        assert_eq!(noir_trait.name.to_string(), "Foo");
        assert_eq!(noir_trait.generics.len(), 2);
        assert_eq!(noir_trait.where_clause.len(), 1);
        assert!(noir_trait.items.is_empty());
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
    }

    #[test]
    fn parse_trait_inheirtance() {
        let src = "trait Foo: Bar + Baz {}";
        let noir_trait = parse_trait_no_errors(src);
        assert_eq!(noir_trait.bounds.len(), 2);

        assert_eq!(noir_trait.bounds[0].to_string(), "Bar");
        assert_eq!(noir_trait.bounds[1].to_string(), "Baz");

        assert_eq!(noir_trait.to_string(), "trait Foo: Bar + Baz {\n}");
    }
}
