use noirc_errors::Location;

use crate::{
    ast::{ItemVisibility, TypeAlias, UnresolvedType, UnresolvedTypeData},
    token::Token,
};

use super::Parser;

impl Parser<'_> {
    /// TypeAlias = 'type' identifier Generics '=' Type ';'
    pub(crate) fn parse_type_alias(
        &mut self,
        visibility: ItemVisibility,
        start_location: Location,
    ) -> TypeAlias {
        let location = self.location_at_previous_token_end();
        let Some(name) = self.eat_non_underscore_ident() else {
            self.expected_identifier();
            return TypeAlias {
                visibility,
                name: self.unknown_ident_at_previous_token_end(),
                generics: Vec::new(),
                typ: UnresolvedType { typ: UnresolvedTypeData::Error, location },
                location: start_location,
                numeric_type: None,
                numeric_location: Location::dummy(),
            };
        };
        // Optional numeric type for alias over numeric generics
        let mut num_typ = None;
        let generics = self.parse_generics_disallowing_trait_bounds();
        if self.eat_colon() {
            // To specify a type alias on a numeric generic expression, we need to specify the type of the expression
            // It must be a numeric type
            num_typ = Some(self.parse_type_or_error());
        }
        let mut expr_location = self.current_token_location;
        let location;
        let typ = if !self.eat_assign() {
            self.expected_token(Token::Assign);
            location = self.location_since(start_location);
            self.eat_semicolons();
            UnresolvedType {
                typ: UnresolvedTypeData::Error,
                location: self.location_at_previous_token_end(),
            }
        } else {
            expr_location = self.current_token_location;
            let typ = self.parse_type_or_type_expression().unwrap_or(UnresolvedType {
                typ: UnresolvedTypeData::Error,
                location: expr_location,
            });
            location = self.location_since(start_location);
            self.eat_semicolon_or_error();
            typ
        };
        let numeric_location = self.location_since(expr_location);

        TypeAlias {
            visibility,
            name,
            generics,
            typ,
            location,
            numeric_type: num_typ,
            numeric_location,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{TypeAlias, UnresolvedType, UnresolvedTypeData},
        parse_program_with_dummy_file,
        parser::{ItemKind, parser::tests::expect_no_errors},
    };

    fn parse_type_alias_no_errors(src: &str) -> TypeAlias {
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

    #[test]
    fn parse_numeric_generic_type_alias() {
        let src = "type Double<let N: u32>: u32 = N * 2;";
        let alias = parse_type_alias_no_errors(src);
        assert_eq!("Double", alias.name.to_string());
        assert_eq!(alias.generics.len(), 1);
    }

    #[test]
    fn parse_incomplete_type_alias() {
        let src = "type Foo = ";
        let (mut module, errors) = parse_program_with_dummy_file(src);
        assert!(!errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::TypeAlias(alias) = item.kind else {
            panic!("Expected global");
        };
        assert_eq!(alias.name.to_string(), "Foo");
        assert!(matches!(alias.typ, UnresolvedType { typ: UnresolvedTypeData::Error, .. }));
    }
}
