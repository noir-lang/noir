use noirc_errors::Span;

use crate::{
    ast::{Ident, ItemVisibility, NoirTypeAlias, UnresolvedType, UnresolvedTypeData},
    token::Token,
};

use super::Parser;

impl<'a> Parser<'a> {
    /// TypeAlias = 'type' identifier Generics '=' Type ';'
    pub(crate) fn parse_type_alias(
        &mut self,
        visibility: ItemVisibility,
        start_span: Span,
    ) -> NoirTypeAlias {
        let Some(name) = self.eat_ident() else {
            self.expected_identifier();
            return NoirTypeAlias {
                visibility,
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
                visibility,
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

        NoirTypeAlias { visibility, name, generics, typ, span }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{NoirTypeAlias, UnresolvedTypeData},
        parser::{
            parser::{parse_program, tests::expect_no_errors},
            ItemKind,
        },
    };

    fn parse_type_alias_no_errors(src: &str) -> NoirTypeAlias {
        let (mut module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::TypeAlias(alias) = item.kind else {
            panic!("Expected global");
        };
        alias
    }

    #[test]
    fn parse_type_alias_no_generics() {
        let src = "type Foo = Field;";
        let alias = parse_type_alias_no_errors(src);
        assert_eq!("Foo", alias.name.to_string());
        assert!(alias.generics.is_empty());
        assert_eq!(alias.typ.typ, UnresolvedTypeData::FieldElement);
    }

    #[test]
    fn parse_type_alias_with_generics() {
        let src = "type Foo<A> = Field;";
        let alias = parse_type_alias_no_errors(src);
        assert_eq!("Foo", alias.name.to_string());
        assert_eq!(alias.generics.len(), 1);
    }
}
