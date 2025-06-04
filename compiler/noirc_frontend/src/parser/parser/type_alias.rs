use noirc_errors::Location;

use crate::{
    ast::{ItemVisibility, NoirTypeAlias, UnresolvedType, UnresolvedTypeData},
    token::Token,
};

use super::Parser;

impl Parser<'_> {
    /// TypeAlias = 'type' identifier Generics '=' Type ';'
    pub(crate) fn parse_type_alias(
        &mut self,
        visibility: ItemVisibility,
        start_location: Location,
    ) -> NoirTypeAlias {
        let Some(name) = self.eat_ident() else {
            self.expected_identifier();
            let location = self.location_at_previous_token_end();
            return NoirTypeAlias {
                visibility,
                name: self.unknown_ident_at_previous_token_end(),
                generics: Vec::new(),
                typ: UnresolvedType { typ: UnresolvedTypeData::Error, location },
                location: start_location,
            };
        };

        let generics = self.parse_generics_disallowing_trait_bounds();

        if !self.eat_assign() {
            self.expected_token(Token::Assign);

            let location = self.location_since(start_location);
            self.eat_semicolons();

            return NoirTypeAlias {
                visibility,
                name,
                generics,
                typ: UnresolvedType {
                    typ: UnresolvedTypeData::Error,
                    location: self.location_at_previous_token_end(),
                },
                location,
            };
        }

        let typ = self.parse_type_or_error();
        let location = self.location_since(start_location);
        self.eat_semicolon_or_error();

        NoirTypeAlias { visibility, name, generics, typ, location }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::NoirTypeAlias,
        parse_program_with_dummy_file,
        parser::{ItemKind, parser::tests::expect_no_errors},
    };

    fn parse_type_alias_no_errors(src: &str) -> NoirTypeAlias {
        let (mut module, errors) = parse_program_with_dummy_file(src);
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
        assert_eq!(alias.typ.typ.to_string(), "Field");
    }

    #[test]
    fn parse_type_alias_with_generics() {
        let src = "type Foo<A> = Field;";
        let alias = parse_type_alias_no_errors(src);
        assert_eq!("Foo", alias.name.to_string());
        assert_eq!(alias.generics.len(), 1);
    }
}
