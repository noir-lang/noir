use noirc_errors::Span;

use crate::ast::{Ident, NoirTypeAlias, UnresolvedType, UnresolvedTypeData};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_type_alias(&mut self, start_span: Span) -> NoirTypeAlias {
        let Some(name) = self.eat_ident() else {
            // TODO: error
            return NoirTypeAlias {
                name: Ident::default(),
                generics: Vec::new(),
                typ: UnresolvedType { typ: UnresolvedTypeData::Error, span: Span::default() },
                span: start_span,
            };
        };

        let generics = self.parse_generics();

        if !self.eat_assign() {
            self.eat_semicolons();

            // TODO: error
            return NoirTypeAlias {
                name,
                generics,
                typ: UnresolvedType { typ: UnresolvedTypeData::Error, span: Span::default() },
                span: self.span_since(start_span),
            };
        }

        let typ = self.parse_type_or_error();

        NoirTypeAlias { name, generics, typ, span: self.span_since(start_span) }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::UnresolvedTypeData,
        parser::{parser::parse_program, ItemKind},
    };

    #[test]
    fn parse_type_alias_no_generics() {
        let src = "type Foo = Field;";
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::TypeAlias(alias) = &item.kind else {
            panic!("Expected global");
        };
        assert_eq!("Foo", alias.name.to_string());
        assert!(alias.generics.is_empty());
        assert_eq!(alias.typ.typ, UnresolvedTypeData::FieldElement);
    }

    #[test]
    fn parse_type_alias_with_generics() {
        let src = "type Foo<A> = Field;";
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::TypeAlias(alias) = &item.kind else {
            panic!("Expected type alias");
        };
        assert_eq!("Foo", alias.name.to_string());
        assert_eq!(alias.generics.len(), 1);
    }
}
