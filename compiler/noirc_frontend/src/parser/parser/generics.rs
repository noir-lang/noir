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
            return generics;
        }

        let mut trailing_comma = false;

        loop {
            let start_span = self.current_token_span;
            let Some(generic) = self.parse_generic() else {
                break;
            };

            if !trailing_comma && !generics.is_empty() {
                self.expected_token_separating_items(",", "generic parameters", start_span);
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
        if let Some(generic) = self.parse_variable_generic() {
            return Some(generic);
        }

        if let Some(generic) = self.parse_numeric_generic() {
            return Some(generic);
        }

        if let Some(generic) = self.parse_resolved_generic() {
            return Some(generic);
        }

        None
    }

    fn parse_variable_generic(&mut self) -> Option<UnresolvedGeneric> {
        if let Some(ident) = self.eat_ident() {
            return Some(UnresolvedGeneric::Variable(ident));
        }

        None
    }

    fn parse_numeric_generic(&mut self) -> Option<UnresolvedGeneric> {
        if !self.eat_keyword(Keyword::Let) {
            return None;
        }

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

        let typ = self.parse_type_or_error();
        if let UnresolvedTypeData::Integer(signedness, bit_size) = &typ.typ {
            if matches!(signedness, Signedness::Signed)
                || matches!(bit_size, IntegerBitSize::SixtyFour)
            {
                self.push_error(ParserErrorReason::ForbiddenNumericGenericType, typ.span);
            }
        }

        Some(UnresolvedGeneric::Numeric { ident, typ })
    }

    fn parse_resolved_generic(&mut self) -> Option<UnresolvedGeneric> {
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

            if matches!(self.token.token(), Token::Ident(..))
                && self.next_token.token() == &Token::Assign
            {
                let ident = self.eat_ident().unwrap();

                if !trailing_comma && !generic_type_args.is_empty() {
                    self.expected_token_separating_items(",", "generic parameters", start_span);
                }

                self.eat_assign();

                let typ = self.parse_type_or_error();
                generic_type_args.named_args.push((ident, typ));
            } else {
                let typ = self.parse_type_or_type_expression();
                let Some(typ) = typ else {
                    // TODO: error? (not sure if this is `<>` so test that)
                    self.eat_greater();
                    break;
                };

                if !trailing_comma && !generic_type_args.is_empty() {
                    self.expected_token_separating_items(",", "generic parameters", start_span);
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
        parser::{
            parser::tests::{get_single_error, get_source_with_error_span},
            Parser, ParserErrorReason,
        },
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
        let mut parser = Parser::for_str(src);
        let mut generics = parser.parse_generics();
        assert!(parser.errors.is_empty());
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
        );
    }

    #[test]
    fn parses_no_generic_type_args() {
        let src = "1";
        let mut parser = Parser::for_str(src);
        let generics = parser.parse_generic_type_args();
        assert!(parser.errors.is_empty());
        assert!(generics.is_empty());
    }

    #[test]
    fn parses_generic_type_args() {
        let src = "<i32, X = Field>";
        let mut parser = Parser::for_str(src);
        let generics = parser.parse_generic_type_args();
        assert!(parser.errors.is_empty());
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
        let mut parser = Parser::for_str(src);
        let generics = parser.parse_generic_type_args();
        assert!(parser.errors.is_empty());
        assert!(!generics.is_empty());
        assert_eq!(generics.ordered_args.len(), 1);
        assert_eq!(generics.ordered_args[0].to_string(), "foo::Bar");
        assert_eq!(generics.named_args.len(), 0);
    }

    #[test]
    fn parses_generic_type_arg_that_is_an_int() {
        let src = "<1>";
        let mut parser = Parser::for_str(src);
        let generics = parser.parse_generic_type_args();
        assert!(parser.errors.is_empty());
        assert!(!generics.is_empty());
        assert_eq!(generics.ordered_args.len(), 1);
        assert_eq!(generics.ordered_args[0].to_string(), "1");
    }

    #[test]
    fn parse_numeric_generic_error_if_invalid_integer() {
        let src = "
        <let N: u64>
                ^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        parser.parse_generics();
        let reason = get_single_error(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::ForbiddenNumericGenericType));
    }

    #[test]
    fn parse_arithmetic_generic_on_variable() {
        let src = "<N - 1>";
        let mut parser = Parser::for_str(src);
        let generics = parser.parse_generic_type_args();
        assert_eq!(generics.ordered_args[0].to_string(), "(N - 1)");
    }

    #[test]
    fn parse_var_with_turbofish_in_generic() {
        let src = "<N<1>>";
        let mut parser = Parser::for_str(src);
        let generics = parser.parse_generic_type_args();
        assert_eq!(generics.ordered_args[0].to_string(), "N<1>");
    }
}
