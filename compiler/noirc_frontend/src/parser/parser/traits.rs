use noirc_errors::Span;

use crate::{
    ast::{
        Documented, Ident, ItemVisibility, NoirTrait, Pattern, TraitItem, UnresolvedType,
        UnresolvedTypeData,
    },
    token::{Attribute, Keyword, SecondaryAttribute},
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_trait(
        &mut self,
        attributes: Vec<(Attribute, Span)>,
        visibility: ItemVisibility,
        start_span: Span,
    ) -> NoirTrait {
        let attributes = self.validate_secondary_attributes(attributes);

        let Some(name) = self.eat_ident() else {
            // TODO: error
            return empty_trait(attributes, visibility, self.span_since(start_span));
        };

        let generics = self.parse_generics();
        let where_clause = self.parse_where_clause();
        let items = self.parse_trait_items();

        NoirTrait {
            name,
            generics,
            where_clause,
            span: self.span_since(start_span),
            items,
            attributes,
            visibility,
        }
    }

    fn parse_trait_items(&mut self) -> Vec<Documented<TraitItem>> {
        let mut items = Vec::new();

        if !self.eat_left_brace() {
            // TODO: error
            return items;
        }

        loop {
            let doc_comments = self.parse_outer_doc_comments();

            if let Some(item) = self.parse_trait_item() {
                items.push(Documented::new(item, doc_comments));

                if self.eat_right_brace() {
                    break;
                }
            } else {
                // TODO: error
                if self.is_eof() || self.eat_right_brace() {
                    break;
                } else {
                    // Keep going
                    self.next_token();
                }
            }
        }

        items
    }

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

    fn parse_trait_type(&mut self) -> Option<TraitItem> {
        if !self.eat_keyword(Keyword::Type) {
            return None;
        }

        let name = match self.eat_ident() {
            Some(name) => name,
            None => {
                // TODO: error
                Ident::default()
            }
        };

        self.eat_semicolons();

        Some(TraitItem::Type { name })
    }

    fn parse_trait_constant(&mut self) -> Option<TraitItem> {
        if !self.eat_keyword(Keyword::Let) {
            return None;
        }

        let name = match self.eat_ident() {
            Some(name) => name,
            None => {
                // TODO: error
                Ident::default()
            }
        };

        let typ = if self.eat_colon() {
            self.parse_type()
        } else {
            // TODO: error
            UnresolvedType { typ: UnresolvedTypeData::Unspecified, span: Span::default() }
        };

        let default_value = if self.eat_assign() { Some(self.parse_expression()) } else { None };

        self.eat_semicolons();

        Some(TraitItem::Constant { name, typ, default_value })
    }

    fn parse_trait_function(&mut self) -> Option<TraitItem> {
        let is_unconstrained = self.eat_keyword(Keyword::Unconstrained);
        let visibility = self.parse_item_visibility();
        let is_comptime = self.eat_keyword(Keyword::Comptime);

        if !self.eat_keyword(Keyword::Fn) {
            // TODO: error if unconstrained, visibility or comptime
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
                    // TODO: error
                    None
                }
            })
            .collect();

        Some(TraitItem::Function {
            is_unconstrained,
            visibility,
            is_comptime,
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
    return NoirTrait {
        name: Ident::default(),
        generics: Vec::new(),
        where_clause: Vec::new(),
        span,
        items: Vec::new(),
        attributes,
        visibility,
    };
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::TraitItem,
        parser::{parser::parse_program, ItemKind},
    };

    #[test]
    fn parse_empty_trait() {
        let src = "trait Foo {}";
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Trait(noir_trait) = &item.kind else {
            panic!("Expected trait");
        };
        assert_eq!(noir_trait.name.to_string(), "Foo");
        assert!(noir_trait.generics.is_empty());
        assert!(noir_trait.where_clause.is_empty());
        assert!(noir_trait.items.is_empty());
    }

    #[test]
    fn parse_trait_with_generics() {
        let src = "trait Foo<A, B> {}";
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Trait(noir_trait) = &item.kind else {
            panic!("Expected trait");
        };
        assert_eq!(noir_trait.name.to_string(), "Foo");
        assert_eq!(noir_trait.generics.len(), 2);
        assert!(noir_trait.where_clause.is_empty());
        assert!(noir_trait.items.is_empty());
    }

    #[test]
    fn parse_trait_with_where_clause() {
        let src = "trait Foo<A, B> where A: Z {}";
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Trait(noir_trait) = &item.kind else {
            panic!("Expected trait");
        };
        assert_eq!(noir_trait.name.to_string(), "Foo");
        assert_eq!(noir_trait.generics.len(), 2);
        assert_eq!(noir_trait.where_clause.len(), 1);
        assert!(noir_trait.items.is_empty());
    }

    #[test]
    fn parse_trait_with_type() {
        let src = "trait Foo { type Elem; }";
        let (mut module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Trait(mut noir_trait) = item.kind else {
            panic!("Expected trait");
        };
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
        let (mut module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Trait(mut noir_trait) = item.kind else {
            panic!("Expected trait");
        };
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
        let (mut module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Trait(mut noir_trait) = item.kind else {
            panic!("Expected trait");
        };
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
        let (mut module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Trait(mut noir_trait) = item.kind else {
            panic!("Expected trait");
        };
        assert_eq!(noir_trait.items.len(), 1);

        let item = noir_trait.items.remove(0).item;
        let TraitItem::Function { body, .. } = item else {
            panic!("Expected function");
        };
        assert!(body.is_some());
    }
}
