use noirc_errors::Span;

use crate::{
    ast::{Ident, NoirTypeAlias, UnresolvedType, UnresolvedTypeData},
    token::Token,
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_type_alias(&mut self, start_span: Span) -> NoirTypeAlias {
        let Some(name) = self.eat_ident() else {
            self.expected_identifier();
            return NoirTypeAlias {
                name: Ident::default(),
                generics: Vec::new(),
                typ: UnresolvedType { typ: UnresolvedTypeData::Error, span: Span::default() },
                span: start_span,
            };
        };

        let generics = self.parse_generics();

        if !self.eat_assign() {
            self.expected_token(Token::Assign);

            let span = self.span_since(start_span);
            self.eat_semicolons();

            return NoirTypeAlias {
                name,
                generics,
                typ: UnresolvedType { typ: UnresolvedTypeData::Error, span: Span::default() },
                span,
            };
        }

        let typ = self.parse_type_or_error();
        let span = self.span_since(start_span);
        if !self.eat_semicolons() {
            self.expected_token(Token::Semicolon);
        }

        NoirTypeAlias { name, generics, typ, span }
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
