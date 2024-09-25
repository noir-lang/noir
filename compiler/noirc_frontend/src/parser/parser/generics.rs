use noirc_errors::Span;

use crate::{
    ast::{
        IntegerBitSize, Signedness, UnresolvedGeneric, UnresolvedGenerics, UnresolvedType,
        UnresolvedTypeData,
    },
    token::{Keyword, Token, TokenKind},
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_generics(&mut self) -> UnresolvedGenerics {
        let mut generics = Vec::new();

        if !self.eat_less() {
            return generics;
        }

        while let Some(generic) = self.parse_generic() {
            generics.push(generic);

            self.eat_commas();
            if self.eat_greater() {
                break;
            }
        }

        generics
    }

    fn parse_generic(&mut self) -> Option<UnresolvedGeneric> {
        // Check `T`
        if let Some(ident) = self.eat_ident() {
            return Some(UnresolvedGeneric::Variable(ident));
        }

        // Check `let N: u32`
        if self.eat_keyword(Keyword::Let) {
            let Some(ident) = self.eat_ident() else {
                return None;
            };

            if !self.eat_colon() {
                // TODO: error
                return Some(UnresolvedGeneric::Numeric { ident, typ: type_u32() });
            }

            let typ = self.parse_type();

            // TODO: error if typ isn't an integer type

            return Some(UnresolvedGeneric::Numeric { ident, typ });
        }

        // Check resolved generics
        if let Some(token) = self.eat_kind(TokenKind::QuotedType) {
            match token.into_token() {
                Token::QuotedType(id) => {
                    return Some(UnresolvedGeneric::Resolved(id, self.previous_token_span));
                }
                _ => unreachable!(),
            }
        }

        None
    }
}

fn type_u32() -> UnresolvedType {
    UnresolvedType {
        typ: UnresolvedTypeData::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo),
        span: Span::default(),
    }
}
