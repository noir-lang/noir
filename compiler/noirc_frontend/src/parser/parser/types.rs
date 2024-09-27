use noirc_errors::Span;

use crate::{
    ast::{Ident, UnresolvedType, UnresolvedTypeData},
    token::{Keyword, Token},
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_type(&mut self) -> UnresolvedType {
        let start_span = self.current_token_span;

        let typ = self.parse_unresolved_type_data();
        let span = self.span_since(start_span);

        UnresolvedType { typ, span }
    }

    fn parse_unresolved_type_data(&mut self) -> UnresolvedTypeData {
        if let Some(typ) = self.parse_parentheses_type() {
            return typ;
        }

        if let Some(typ) = self.parse_bool_type() {
            return typ;
        }

        if let Some(typ) = self.parse_field_type() {
            return typ;
        }

        if let Some(typ) = self.parse_int_type() {
            return typ;
        }

        if let Some(typ) = self.parses_mutable_reference_type() {
            return typ;
        }

        let path = self.parse_path_no_turbofish();
        if !path.is_empty() {
            let generics = self.parse_generic_type_args();
            return UnresolvedTypeData::Named(path, generics, false);
        }

        // TODO: parse more types

        UnresolvedTypeData::Error
    }

    fn parse_bool_type(&mut self) -> Option<UnresolvedTypeData> {
        if self.eat_keyword(Keyword::Bool) {
            return Some(UnresolvedTypeData::Bool);
        }

        None
    }

    fn parse_field_type(&mut self) -> Option<UnresolvedTypeData> {
        if self.eat_keyword(Keyword::Field) {
            return Some(UnresolvedTypeData::FieldElement);
        }

        None
    }

    fn parse_int_type(&mut self) -> Option<UnresolvedTypeData> {
        if let Some(int_type) = self.eat_int_type() {
            return Some(match UnresolvedTypeData::from_int_token(int_type) {
                Ok(typ) => typ,
                Err(_) => {
                    // TODO: error
                    UnresolvedTypeData::Error
                }
            });
        }

        None
    }

    fn parses_mutable_reference_type(&mut self) -> Option<UnresolvedTypeData> {
        if self.eat(Token::Ampersand) {
            if !self.eat_keyword(Keyword::Mut) {
                // TODO: error
            }
            return Some(UnresolvedTypeData::MutableReference(Box::new(self.parse_type())));
        };

        None
    }

    fn parse_parentheses_type(&mut self) -> Option<UnresolvedTypeData> {
        if !self.eat_left_paren() {
            return None;
        }

        if self.eat_right_paren() {
            return Some(UnresolvedTypeData::Unit);
        }

        let mut types = Vec::new();
        let mut trailing_comma = false;
        loop {
            let start_span = self.current_token_span;
            let typ = self.parse_type();
            if self.current_token_span == start_span {
                // TODO: error
                self.eat_right_paren();
                break;
            }

            types.push(typ);

            trailing_comma = self.eat_commas();
            // TODO: error if no comma between types

            if self.eat_right_paren() {
                break;
            }
        }

        Some(if types.len() == 1 && !trailing_comma {
            UnresolvedTypeData::Parenthesized(Box::new(types.remove(0)))
        } else {
            UnresolvedTypeData::Tuple(types)
        })
    }

    pub(super) fn parse_path_type_after_ident(&mut self, ident: Ident) -> UnresolvedType {
        let start_span = ident.span();
        let path = self.parse_path_no_turbofish_after_ident(ident);
        let generics = self.parse_generic_type_args();
        let typ = UnresolvedTypeData::Named(path, generics, false);
        UnresolvedType { typ, span: self.span_since(start_span) }
    }

    pub(super) fn parse_optional_type_annotation(&mut self) -> UnresolvedType {
        if self.eat_colon() {
            self.parse_type()
        } else {
            UnresolvedType { typ: UnresolvedTypeData::Unspecified, span: Span::default() }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{IntegerBitSize, Signedness, UnresolvedTypeData},
        parser::Parser,
    };

    #[test]
    fn parses_unit_type() {
        let src = "()";
        let typ = Parser::for_str(src).parse_type();
        assert!(matches!(typ.typ, UnresolvedTypeData::Unit));
    }

    #[test]
    fn parses_bool_type() {
        let src = "bool";
        let typ = Parser::for_str(src).parse_type();
        assert!(matches!(typ.typ, UnresolvedTypeData::Bool));
    }

    #[test]
    fn parses_int_type() {
        let src = "u32";
        let typ = Parser::for_str(src).parse_type();
        assert!(matches!(
            typ.typ,
            UnresolvedTypeData::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo)
        ));
    }

    #[test]
    fn parses_field_type() {
        let src = "Field";
        let typ = Parser::for_str(src).parse_type();
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parses_tuple_type() {
        let src = "(Field, bool)";
        let typ = Parser::for_str(src).parse_type();
        let UnresolvedTypeData::Tuple(mut types) = typ.typ else { panic!("Expected a tuple type") };
        assert_eq!(types.len(), 2);

        let typ = types.remove(0);
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));

        let typ = types.remove(0);
        assert!(matches!(typ.typ, UnresolvedTypeData::Bool));
    }

    #[test]
    fn parses_tuple_type_one_element() {
        let src = "(Field,)";
        let typ = Parser::for_str(src).parse_type();
        let UnresolvedTypeData::Tuple(mut types) = typ.typ else { panic!("Expected a tuple type") };
        assert_eq!(types.len(), 1);

        let typ = types.remove(0);
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parses_parenthesized_type() {
        let src = "(Field)";
        let typ = Parser::for_str(src).parse_type();
        let UnresolvedTypeData::Parenthesized(typ) = typ.typ else {
            panic!("Expected a parenthesized type")
        };
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parses_unclosed_parentheses_type() {
        let src = "(Field";
        let mut parser = Parser::for_str(src);
        assert!(parser.errors.is_empty()); // TODO: there should be an error here
        let typ = parser.parse_type();
        let UnresolvedTypeData::Parenthesized(typ) = typ.typ else {
            panic!("Expected a parenthesized type")
        };
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parses_mutable_reference_type() {
        let src = "&mut Field";
        let typ = Parser::for_str(src).parse_type();
        let UnresolvedTypeData::MutableReference(typ) = typ.typ else {
            panic!("Expected a mutable reference type")
        };
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parses_named_type_no_generics() {
        let src = "foo::Bar";
        let typ = Parser::for_str(src).parse_type();
        let UnresolvedTypeData::Named(path, generics, _) = typ.typ else {
            panic!("Expected a named type")
        };
        assert_eq!(path.to_string(), "foo::Bar");
        assert!(generics.is_empty());
    }
}
