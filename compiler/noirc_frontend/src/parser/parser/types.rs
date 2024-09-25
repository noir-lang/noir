use crate::ast::{UnresolvedType, UnresolvedTypeData};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_type(&mut self) -> UnresolvedType {
        let start_span = self.current_token_span;

        if let Some(int_type) = self.eat_int_type() {
            let typ = match UnresolvedTypeData::from_int_token(int_type) {
                Ok(typ) => typ,
                Err(_) => {
                    // TODO: error
                    UnresolvedTypeData::Error
                }
            };
            return UnresolvedType { typ, span: self.span_since(start_span) };
        }

        // TODO: parse more types

        UnresolvedType { typ: UnresolvedTypeData::Error, span: start_span }
    }
}
