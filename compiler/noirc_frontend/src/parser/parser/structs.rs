use noirc_errors::Span;

use crate::{
    ast::{Ident, ItemVisibility, NoirStruct, UnresolvedGenerics},
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
            return self.empty_struct(
                Ident::default(),
                attributes,
                visibility,
                generics,
                start_span,
            );
        }

        // TODO: fields

        if !self.eat_right_brace() {
            // TODO: error
        }

        self.empty_struct(name, attributes, visibility, generics, start_span)
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
            panic!("Expected import");
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
            panic!("Expected import");
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
}
