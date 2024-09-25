use crate::{
    ast::{UnresolvedType, UnresolvedTypeData},
    token::Keyword,
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_type(&mut self) -> UnresolvedType {
        let start_span = self.current_token_span;

        // TODO: parse more types

        let typ = if self.eat_keyword(Keyword::Field) {
            UnresolvedTypeData::FieldElement
        } else if let Some(int_type) = self.eat_int_type() {
            match UnresolvedTypeData::from_int_token(int_type) {
                Ok(typ) => typ,
                Err(_) => {
                    // TODO: error
                    UnresolvedTypeData::Error
                }
            }
        } else {
            return UnresolvedType { typ: UnresolvedTypeData::Error, span: start_span };
        };

        UnresolvedType { typ, span: self.span_since(start_span) }
    }
}
