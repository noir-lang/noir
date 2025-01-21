use noirc_errors::Span;

use crate::{
    ast::{Documented, Ident, ItemVisibility, NoirStruct, StructField, UnresolvedGenerics},
    parser::ParserErrorReason,
    token::{Attribute, SecondaryAttribute, Token},
};

use super::{parse_many::separated_by_comma_until_right_brace, Parser};

impl<'a> Parser<'a> {
    /// Struct = 'struct' identifier Generics '{' StructField* '}'
    ///
    /// StructField = OuterDocComments identifier ':' Type
    pub(crate) fn parse_struct(
        &mut self,
        attributes: Vec<(Attribute, Span)>,
        visibility: ItemVisibility,
        start_span: Span,
    ) -> NoirStruct {
        let attributes = self.validate_secondary_attributes(attributes);

        let Some(name) = self.eat_ident() else {
            self.expected_identifier();
            return self.empty_struct(
                Ident::default(),
                attributes,
                visibility,
                Vec::new(),
                start_span,
            );
        };

        let generics = self.parse_generics();

        if self.eat_semicolons() {
            return self.empty_struct(name, attributes, visibility, generics, start_span);
        }

        if !self.eat_left_brace() {
            self.expected_token(Token::LeftBrace);
            return self.empty_struct(name, attributes, visibility, generics, start_span);
        }

        let fields = self.parse_many(
            "struct fields",
            separated_by_comma_until_right_brace(),
            Self::parse_struct_field,
        );

        NoirStruct {
            name,
            attributes,
            visibility,
            generics,
            fields,
            span: self.span_since(start_span),
        }
    }

    fn parse_struct_field(&mut self) -> Option<Documented<StructField>> {
        let mut doc_comments;
        let name;
        let mut visibility;

        // Loop until we find an identifier, skipping anything that's not one
        loop {
            let doc_comments_start_span = self.current_token_span;
            doc_comments = self.parse_outer_doc_comments();

            visibility = self.parse_item_visibility();

            if let Some(ident) = self.eat_ident() {
                name = ident;
                break;
            }

            if visibility != ItemVisibility::Private {
                self.expected_identifier();
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

        self.eat_or_error(Token::Colon);

        let typ = self.parse_type_or_error();
        Some(Documented::new(StructField { visibility, name, typ }, doc_comments))
    }

    fn empty_struct(
        &self,
        name: Ident,
        attributes: Vec<SecondaryAttribute>,
        visibility: ItemVisibility,
        generics: UnresolvedGenerics,
        start_span: Span,
    ) -> NoirStruct {
        NoirStruct {
            name,
            attributes,
            visibility,
            generics,
            fields: Vec::new(),
            span: self.span_since(start_span),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{IntegerBitSize, NoirStruct, Signedness, UnresolvedGeneric, UnresolvedTypeData},
        parser::{
            parser::{
                parse_program,
                tests::{
                    expect_no_errors, get_single_error, get_single_error_reason,
                    get_source_with_error_span,
                },
            },
            ItemKind, ParserErrorReason,
        },
    };

    fn parse_struct_no_errors(src: &str) -> NoirStruct {
        let (mut module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Struct(noir_struct) = item.kind else {
            panic!("Expected struct");
        };
        noir_struct
    }

    #[test]
    fn parse_empty_struct() {
        let src = "struct Foo {}";
        let noir_struct = parse_struct_no_errors(src);
        assert_eq!("Foo", noir_struct.name.to_string());
        assert!(noir_struct.fields.is_empty());
        assert!(noir_struct.generics.is_empty());
    }

    #[test]
    fn parse_empty_struct_followed_by_semicolon() {
        let src = "struct Foo;";
        let noir_struct = parse_struct_no_errors(src);
        assert_eq!("Foo", noir_struct.name.to_string());
        assert!(noir_struct.fields.is_empty());
        assert!(noir_struct.generics.is_empty());
    }

    #[test]
    fn parse_empty_struct_with_generics() {
        let src = "struct Foo<A, let B: u32> {}";
        let mut noir_struct = parse_struct_no_errors(src);
        assert_eq!("Foo", noir_struct.name.to_string());
        assert!(noir_struct.fields.is_empty());
        assert_eq!(noir_struct.generics.len(), 2);

        let generic = noir_struct.generics.remove(0);
        let UnresolvedGeneric::Variable(ident) = generic else {
            panic!("Expected generic variable");
        };
        assert_eq!("A", ident.to_string());

        let generic = noir_struct.generics.remove(0);
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
    fn parse_struct_with_fields() {
        let src = "struct Foo { x: i32, y: Field }";
        let mut noir_struct = parse_struct_no_errors(src);
        assert_eq!("Foo", noir_struct.name.to_string());
        assert_eq!(noir_struct.fields.len(), 2);

        let field = noir_struct.fields.remove(0).item;
        assert_eq!("x", field.name.to_string());
        assert!(matches!(
            field.typ.typ,
            UnresolvedTypeData::Integer(Signedness::Signed, IntegerBitSize::ThirtyTwo)
        ));

        let field = noir_struct.fields.remove(0).item;
        assert_eq!("y", field.name.to_string());
        assert!(matches!(field.typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parse_empty_struct_with_doc_comments() {
        let src = "/// Hello\nstruct Foo {}";
        let (module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        assert_eq!(item.doc_comments.len(), 1);
        let ItemKind::Struct(noir_struct) = &item.kind else {
            panic!("Expected struct");
        };
        assert_eq!("Foo", noir_struct.name.to_string());
    }

    #[test]
    fn parse_unclosed_struct() {
        let src = "struct Foo {";
        let (module, errors) = parse_program(src);
        assert_eq!(errors.len(), 1);
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Struct(noir_struct) = &item.kind else {
            panic!("Expected struct");
        };
        assert_eq!("Foo", noir_struct.name.to_string());
    }

    #[test]
    fn parse_error_no_function_attributes_allowed_on_struct() {
        let src = "
        #[test] struct Foo {}
        ^^^^^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let (_, errors) = parse_program(&src);
        let reason = get_single_error_reason(&errors, span);
        assert!(matches!(reason, ParserErrorReason::NoFunctionAttributesAllowedOnType));
    }

    #[test]
    fn recovers_on_non_field() {
        let src = "
        struct Foo { 42 x: i32 }
                     ^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let (module, errors) = parse_program(&src);

        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Struct(noir_struct) = &item.kind else {
            panic!("Expected struct");
        };
        assert_eq!("Foo", noir_struct.name.to_string());
        assert_eq!(noir_struct.fields.len(), 1);

        let error = get_single_error(&errors, span);
        assert_eq!(error.to_string(), "Expected an identifier but found '42'");
    }
}
