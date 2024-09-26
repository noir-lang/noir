use noirc_errors::Span;

use crate::{
    ast::{Documented, Ident, ItemVisibility, NoirStruct, StructField, UnresolvedGenerics},
    token::{Attribute, SecondaryAttribute},
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_struct(
        &mut self,
        attributes: Vec<Attribute>,
        visibility: ItemVisibility,
        start_span: Span,
    ) -> NoirStruct {
        let attributes = self.validate_secondary_attributes(attributes);

        let Some(name) = self.eat_ident() else {
            // TODO: error
            return self.empty_struct(
                Ident::default(),
                attributes,
                visibility,
                Vec::new(),
                start_span,
            );
        };

        let generics = self.parse_generics();

        if !self.eat_left_brace() {
            // TODO: error
            return self.empty_struct(name, attributes, visibility, generics, start_span);
        }

        let mut fields = Vec::new();

        loop {
            let doc_comments = self.parse_outer_doc_comments();

            let Some(name) = self.eat_ident() else {
                // TODO: error if there are doc comments
                break;
            };

            if !self.eat_colon() {
                // TODO: error
            }

            let typ = self.parse_type();

            fields.push(Documented::new(StructField { name, typ }, doc_comments));

            self.eat_commas();
            // TODO: error if no comma between fields
        }

        if !self.eat_right_brace() {
            // TODO: error
        }

        NoirStruct {
            name,
            attributes,
            visibility,
            generics,
            fields,
            span: self.span_since(start_span),
        }
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
        ast::{IntegerBitSize, Signedness, UnresolvedGeneric, UnresolvedTypeData},
        parser::{parser::parse_program, ItemKind},
    };

    #[test]
    fn parse_empty_struct() {
        let src = "struct Foo {}";
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Struct(noir_struct) = &item.kind else {
            panic!("Expected struct");
        };
        assert_eq!("Foo", noir_struct.name.to_string());
        assert!(noir_struct.fields.is_empty());
        assert!(noir_struct.generics.is_empty());
    }

    #[test]
    fn parse_empty_struct_with_generics() {
        let src = "struct Foo<A, let B: u32> {}";
        let (mut module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Struct(mut noir_struct) = item.kind else {
            panic!("Expected struct");
        };
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
        )
    }

    #[test]
    fn parse_struct_with_fields() {
        let src = "struct Foo { x: i32, y: Field }";
        let (mut module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Struct(mut noir_struct) = item.kind else {
            panic!("Expected struct");
        };
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
        assert!(errors.is_empty());
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
        assert!(errors.is_empty()); // TODO: there should be an error here
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Struct(noir_struct) = &item.kind else {
            panic!("Expected struct");
        };
        assert_eq!("Foo", noir_struct.name.to_string());
    }
}
