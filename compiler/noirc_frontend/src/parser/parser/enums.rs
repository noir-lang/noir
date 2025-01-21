use noirc_errors::Span;

use crate::{
    ast::{Documented, EnumVariant, Ident, ItemVisibility, NoirEnumeration, UnresolvedGenerics},
    parser::ParserErrorReason,
    token::{Attribute, SecondaryAttribute, Token},
};

use super::{
    parse_many::{separated_by_comma_until_right_brace, separated_by_comma_until_right_paren},
    Parser,
};

impl<'a> Parser<'a> {
    /// Enum = 'enum' identifier Generics '{' EnumVariant* '}'
    ///
    /// EnumField = OuterDocComments identifier ':' Type
    pub(crate) fn parse_enum(
        &mut self,
        attributes: Vec<(Attribute, Span)>,
        visibility: ItemVisibility,
        start_span: Span,
    ) -> NoirEnumeration {
        let attributes = self.validate_secondary_attributes(attributes);

        self.push_error(ParserErrorReason::ExperimentalFeature("Enums"), start_span);

        let Some(name) = self.eat_ident() else {
            self.expected_identifier();
            return self.empty_enum(
                Ident::default(),
                attributes,
                visibility,
                Vec::new(),
                start_span,
            );
        };

        let generics = self.parse_generics();

        if !self.eat_left_brace() {
            self.expected_token(Token::LeftBrace);
            return self.empty_enum(name, attributes, visibility, generics, start_span);
        }

        let comma_separated = separated_by_comma_until_right_brace();
        let variants = self.parse_many("enum variants", comma_separated, Self::parse_enum_variant);

        NoirEnumeration {
            name,
            attributes,
            visibility,
            generics,
            variants,
            span: self.span_since(start_span),
        }
    }

    fn parse_enum_variant(&mut self) -> Option<Documented<EnumVariant>> {
        let mut doc_comments;
        let name;

        // Loop until we find an identifier, skipping anything that's not one
        loop {
            let doc_comments_start_span = self.current_token_span;
            doc_comments = self.parse_outer_doc_comments();

            if let Some(ident) = self.eat_ident() {
                name = ident;
                break;
            }

            if !doc_comments.is_empty() {
                self.push_error(
                    ParserErrorReason::DocCommentDoesNotDocumentAnything,
                    self.span_since(doc_comments_start_span),
                );
            }

            // Though we do have to stop at EOF
            if self.at_eof() {
                self.expected_token(Token::RightBrace);
                return None;
            }

            // Or if we find a right brace
            if self.at(Token::RightBrace) {
                return None;
            }

            self.expected_identifier();
            self.bump();
        }

        let mut parameters = Vec::new();

        if self.eat_left_paren() {
            let comma_separated = separated_by_comma_until_right_paren();
            parameters = self.parse_many("variant parameters", comma_separated, Self::parse_type);
        }

        Some(Documented::new(EnumVariant { name, parameters }, doc_comments))
    }

    fn empty_enum(
        &self,
        name: Ident,
        attributes: Vec<SecondaryAttribute>,
        visibility: ItemVisibility,
        generics: UnresolvedGenerics,
        start_span: Span,
    ) -> NoirEnumeration {
        NoirEnumeration {
            name,
            attributes,
            visibility,
            generics,
            variants: Vec::new(),
            span: self.span_since(start_span),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{IntegerBitSize, NoirEnumeration, Signedness, UnresolvedGeneric, UnresolvedTypeData},
        parser::{
            parser::{
                parse_program,
                tests::{expect_no_errors, get_source_with_error_span},
            },
            ItemKind, ParserErrorReason,
        },
    };

    fn parse_enum_no_errors(src: &str) -> NoirEnumeration {
        let (mut module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Enum(noir_enum) = item.kind else {
            panic!("Expected enum");
        };
        noir_enum
    }

    #[test]
    fn parse_empty_enum() {
        let src = "enum Foo {}";
        let noir_enum = parse_enum_no_errors(src);
        assert_eq!("Foo", noir_enum.name.to_string());
        assert!(noir_enum.variants.is_empty());
        assert!(noir_enum.generics.is_empty());
    }

    #[test]
    fn parse_empty_enum_with_generics() {
        let src = "enum Foo<A, let B: u32> {}";
        let mut noir_enum = parse_enum_no_errors(src);
        assert_eq!("Foo", noir_enum.name.to_string());
        assert!(noir_enum.variants.is_empty());
        assert_eq!(noir_enum.generics.len(), 2);

        let generic = noir_enum.generics.remove(0);
        let UnresolvedGeneric::Variable(ident) = generic else {
            panic!("Expected generic variable");
        };
        assert_eq!("A", ident.to_string());

        let generic = noir_enum.generics.remove(0);
        let UnresolvedGeneric::Numeric { ident, typ } = generic else {
            panic!("Expected generic numeric");
        };
        assert_eq!("B", ident.to_string());
        assert_eq!(
            typ.typ,
            UnresolvedTypeData::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo)
        );
    }

    #[test]
    fn parse_enum_with_variants() {
        let src = "enum Foo { X(i32), y(Field, u32), Z }";
        let mut noir_enum = parse_enum_no_errors(src);
        assert_eq!("Foo", noir_enum.name.to_string());
        assert_eq!(noir_enum.variants.len(), 3);

        let variant = noir_enum.variants.remove(0).item;
        assert_eq!("X", variant.name.to_string());
        assert!(matches!(
            variant.parameters[0].typ,
            UnresolvedTypeData::Integer(Signedness::Signed, IntegerBitSize::ThirtyTwo)
        ));

        let variant = noir_enum.variants.remove(0).item;
        assert_eq!("y", variant.name.to_string());
        assert!(matches!(variant.parameters[0].typ, UnresolvedTypeData::FieldElement));
        assert!(matches!(variant.parameters[1].typ, UnresolvedTypeData::Integer(..)));

        let variant = noir_enum.variants.remove(0).item;
        assert_eq!("Z", variant.name.to_string());
        assert_eq!(variant.parameters.len(), 0);
    }

    #[test]
    fn parse_empty_enum_with_doc_comments() {
        let src = "/// Hello\nenum Foo {}";
        let (module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        assert_eq!(item.doc_comments.len(), 1);
        let ItemKind::Enum(noir_enum) = &item.kind else {
            panic!("Expected enum");
        };
        assert_eq!("Foo", noir_enum.name.to_string());
    }

    #[test]
    fn parse_unclosed_enum() {
        let src = "enum Foo {";
        let (module, errors) = parse_program(src);
        assert_eq!(errors.len(), 2);
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Enum(noir_enum) = &item.kind else {
            panic!("Expected enum");
        };
        assert_eq!("Foo", noir_enum.name.to_string());
    }

    #[test]
    fn parse_error_no_function_attributes_allowed_on_enum() {
        let src = "
        #[test] enum Foo {}
        ^^^^^^^
        ";
        let (src, _) = get_source_with_error_span(src);
        let (_, errors) = parse_program(&src);
        let reason = errors[0].reason().unwrap();
        assert!(matches!(reason, ParserErrorReason::NoFunctionAttributesAllowedOnType));
    }

    #[test]
    fn recovers_on_non_field() {
        let src = "
        enum Foo { 42 X(i32) }
                   ^^
        ";
        let (src, _) = get_source_with_error_span(src);
        let (module, errors) = parse_program(&src);

        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Enum(noir_enum) = &item.kind else {
            panic!("Expected enum");
        };
        assert_eq!("Foo", noir_enum.name.to_string());
        assert_eq!(noir_enum.variants.len(), 1);

        let error = &errors[1];
        assert_eq!(error.to_string(), "Expected an identifier but found '42'");
    }
}
