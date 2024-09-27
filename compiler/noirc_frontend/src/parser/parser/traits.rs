use noirc_errors::Span;

use crate::{
    ast::{Documented, Ident, ItemVisibility, NoirTrait, TraitItem},
    token::{Attribute, SecondaryAttribute},
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

        self.eat_right_brace();

        items
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
    use crate::parser::{parser::parse_program, ItemKind};

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
}
