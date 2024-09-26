use noirc_errors::Span;

use crate::{
    ast::{UnresolvedType, UnresolvedTypeData},
    token::Keyword,
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_type(&mut self) -> UnresolvedType {
        let start_span = self.current_token_span;

        let typ = self.parse_unresolved_type_data();
        let span = if self.current_token_span == start_span {
            start_span
        } else {
            self.span_since(start_span)
        };

        UnresolvedType { typ, span }
    }

    fn parse_unresolved_type_data(&mut self) -> UnresolvedTypeData {
        if self.eat_keyword(Keyword::Field) {
            return UnresolvedTypeData::FieldElement;
        }

        if let Some(int_type) = self.eat_int_type() {
            return match UnresolvedTypeData::from_int_token(int_type) {
                Ok(typ) => typ,
                Err(_) => {
                    // TODO: error
                    UnresolvedTypeData::Error
                }
            };
        }

        // TODO: parse more types

        UnresolvedTypeData::Error
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
}
