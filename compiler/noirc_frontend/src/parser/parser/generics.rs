use noirc_errors::Span;

use crate::{
    ast::{
        GenericTypeArgs, IntegerBitSize, Signedness, UnresolvedGeneric, UnresolvedGenerics,
        UnresolvedType, UnresolvedTypeData,
    },
    parser::ParserErrorReason,
    token::{Keyword, Token, TokenKind},
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_generics(&mut self) -> UnresolvedGenerics {
        let mut generics = Vec::new();

        if !self.eat_less() {
            return generics;
        }

        if self.eat_greater() {
            // TODO: error?
            return generics;
        }

        let mut trailing_comma = false;

        loop {
            let start_span = self.current_token_span;
            let Some(generic) = self.parse_generic() else {
                break;
            };

            if !trailing_comma && !generics.is_empty() {
                self.push_error(ParserErrorReason::MissingCommaSeparatingGenerics, start_span);
            }

            generics.push(generic);

            trailing_comma = self.eat_commas();

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
                self.push_error(
                    ParserErrorReason::MissingTypeForNumericGeneric,
                    self.current_token_span,
                );
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

    pub(super) fn parse_generic_type_args(&mut self) -> GenericTypeArgs {
        let mut generic_type_args = GenericTypeArgs::default();
        if !self.eat_less() {
            return generic_type_args;
        }

        let mut trailing_comma = false;
        loop {
            let start_span = self.current_token_span;
            if let Some(ident) = self.eat_ident() {
                if !trailing_comma && !generic_type_args.is_empty() {
                    self.push_error(ParserErrorReason::MissingCommaSeparatingGenerics, start_span);
                }

                if self.eat_assign() {
                    let typ = self.parse_type();
                    generic_type_args.named_args.push((ident, typ));
                } else {
                    let typ = self.parse_path_type_after_ident(ident);
                    generic_type_args.ordered_args.push(typ);
                }
            } else {
                let typ = self.parse_type();
                if self.current_token_span == start_span {
                    self.eat_greater();
                    break;
                }

                if !trailing_comma && !generic_type_args.is_empty() {
                    println!("1");
                    self.push_error(ParserErrorReason::MissingCommaSeparatingGenerics, start_span);
                }

                generic_type_args.ordered_args.push(typ);
            }

            trailing_comma = self.eat_commas();
        }

        generic_type_args
    }
}

fn type_u32() -> UnresolvedType {
    UnresolvedType {
        typ: UnresolvedTypeData::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo),
        span: Span::default(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{IntegerBitSize, Signedness, UnresolvedGeneric, UnresolvedTypeData},
        parser::Parser,
    };

    #[test]
    fn parses_no_generics() {
        let src = "1";
        let generics = Parser::for_str(src).parse_generics();
        assert!(generics.is_empty());
    }

    #[test]
    fn parses_generics() {
        let src = "<A, let B: u32>";
        let mut generics = Parser::for_str(src).parse_generics();
        assert_eq!(generics.len(), 2);

        let generic = generics.remove(0);
        let UnresolvedGeneric::Variable(ident) = generic else {
            panic!("Expected generic variable");
        };
        assert_eq!("A", ident.to_string());

        let generic = generics.remove(0);
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
    fn parses_no_generic_type_args() {
        let src = "1";
        let generics = Parser::for_str(src).parse_generic_type_args();
        assert!(generics.is_empty());
    }

    #[test]
    fn parses_generic_type_args() {
        let src = "<i32, X = Field>";
        let generics = Parser::for_str(src).parse_generic_type_args();
        assert!(!generics.is_empty());
        assert_eq!(generics.ordered_args.len(), 1);
        assert_eq!(generics.ordered_args[0].to_string(), "i32");
        assert_eq!(generics.named_args.len(), 1);
        assert_eq!(generics.named_args[0].0.to_string(), "X");
        assert_eq!(generics.named_args[0].1.to_string(), "Field");
    }

    #[test]
    fn parses_generic_type_arg_that_is_a_path() {
        let src = "<foo::Bar>";
        let generics = Parser::for_str(src).parse_generic_type_args();
        assert!(!generics.is_empty());
        assert_eq!(generics.ordered_args.len(), 1);
        assert_eq!(generics.ordered_args[0].to_string(), "foo::Bar");
        assert_eq!(generics.named_args.len(), 0);
    }
}
