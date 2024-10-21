use crate::{
    ast::{
        GenericTypeArg, GenericTypeArgs, IntegerBitSize, Signedness, UnresolvedGeneric,
        UnresolvedGenerics, UnresolvedType, UnresolvedTypeData,
    },
    parser::{labels::ParsingRuleLabel, ParserErrorReason},
    token::{Keyword, Token, TokenKind},
};

use super::{parse_many::separated_by_comma, Parser};

impl<'a> Parser<'a> {
    /// Generics = ( '<' GenericsList? '>' )?
    ///
    /// GenericsList = Generic ( ',' Generic )* ','?
    pub(super) fn parse_generics(&mut self) -> UnresolvedGenerics {
        if !self.eat_less() {
            return Vec::new();
        }

        self.parse_many(
            "generic parameters",
            separated_by_comma().until(Token::Greater),
            Self::parse_generic_in_list,
        )
    }

    fn parse_generic_in_list(&mut self) -> Option<UnresolvedGeneric> {
        if let Some(generic) = self.parse_generic() {
            Some(generic)
        } else {
            self.expected_label(ParsingRuleLabel::GenericParameter);
            None
        }
    }

    /// Generic
    ///     = VariableGeneric
    ///     | NumericGeneric
    ///     | ResolvedGeneric
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

    /// VariableGeneric = identifier
    fn parse_variable_generic(&mut self) -> Option<UnresolvedGeneric> {
        self.eat_ident().map(UnresolvedGeneric::Variable)
    }

    /// NumericGeneric = 'let' identifier ':' Type
    fn parse_numeric_generic(&mut self) -> Option<UnresolvedGeneric> {
        if !self.eat_keyword(Keyword::Let) {
            return None;
        }

        let ident = self.eat_ident()?;

        if !self.eat_colon() {
            // If we didn't get a type after the colon, error and assume it's u32
            self.push_error(
                ParserErrorReason::MissingTypeForNumericGeneric,
                self.current_token_span,
            );
            let typ = UnresolvedType {
                typ: UnresolvedTypeData::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo),
                span: self.span_at_previous_token_end(),
            };
            return Some(UnresolvedGeneric::Numeric { ident, typ });
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

    /// ResolvedGeneric = quoted_type
    fn parse_resolved_generic(&mut self) -> Option<UnresolvedGeneric> {
        let token = self.eat_kind(TokenKind::QuotedType)?;
        match token.into_token() {
            Token::QuotedType(id) => {
                Some(UnresolvedGeneric::Resolved(id, self.previous_token_span))
            }
            _ => unreachable!(),
        }
    }

    /// GenericTypeArgs = ( '<' GenericTypeArgsList? '>' )
    ///
    /// GenericTypeArgsList = GenericTypeArg ( ',' GenericTypeArg )* ','?
    ///
    /// GenericTypeArg
    ///     = NamedTypeArg
    ///     | OrderedTypeArg
    ///
    /// NamedTypeArg = identifier '=' Type
    ///
    /// OrderedTypeArg = TypeOrTypeExpression
    pub(super) fn parse_generic_type_args(&mut self) -> GenericTypeArgs {
        let mut generic_type_args = GenericTypeArgs::default();
        if !self.eat_less() {
            return generic_type_args;
        }

        let generics = self.parse_many(
            "generic parameters",
            separated_by_comma().until(Token::Greater),
            Self::parse_generic_type_arg,
        );

        for generic in generics {
            match generic {
                GenericTypeArg::Ordered(typ) => {
                    generic_type_args.ordered_args.push(typ);
                    generic_type_args.kinds.push(crate::ast::GenericTypeArgKind::Ordered);
                }
                GenericTypeArg::Named(name, typ) => {
                    generic_type_args.named_args.push((name, typ));
                    generic_type_args.kinds.push(crate::ast::GenericTypeArgKind::Named);
                }
            }
        }

        generic_type_args
    }

    fn parse_generic_type_arg(&mut self) -> Option<GenericTypeArg> {
        if matches!(self.token.token(), Token::Ident(..)) && self.next_is(Token::Assign) {
            let ident = self.eat_ident().unwrap();

            self.eat_assign();

            let typ = self.parse_type_or_error();
            return Some(GenericTypeArg::Named(ident, typ));
        }

        // Otherwise
        let Some(typ) = self.parse_type_or_type_expression() else {
            self.expected_label(ParsingRuleLabel::TypeOrTypeExpression);
            return None;
        };

        Some(GenericTypeArg::Ordered(typ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{GenericTypeArgs, IntegerBitSize, Signedness, UnresolvedGeneric, UnresolvedTypeData},
        parser::{
            parser::tests::{
                expect_no_errors, get_single_error_reason, get_source_with_error_span,
            },
            Parser, ParserErrorReason,
        },
    };

    fn parse_generics_no_errors(src: &str) -> Vec<UnresolvedGeneric> {
        let mut parser = Parser::for_str(src);
        let generics = parser.parse_generics();
        expect_no_errors(&parser.errors);
        generics
    }

    fn parse_generic_type_args_no_errors(src: &str) -> GenericTypeArgs {
        let mut parser = Parser::for_str(src);
        let generics = parser.parse_generic_type_args();
        expect_no_errors(&parser.errors);
        generics
    }

    #[test]
    fn parses_no_generics() {
        let src = "1";
        let generics = parse_generics_no_errors(src);
        assert!(generics.is_empty());
    }

    #[test]
    fn parses_generics() {
        let src = "<A, let B: u32>";
        let mut generics = parse_generics_no_errors(src);
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
        let generics = parse_generic_type_args_no_errors(src);
        assert!(generics.is_empty());
    }

    #[test]
    fn parses_generic_type_args() {
        let src = "<i32, X = Field>";
        let generics = parse_generic_type_args_no_errors(src);
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
        let generics = parse_generic_type_args_no_errors(src);
        assert!(!generics.is_empty());
        assert_eq!(generics.ordered_args.len(), 1);
        assert_eq!(generics.ordered_args[0].to_string(), "foo::Bar");
        assert_eq!(generics.named_args.len(), 0);
    }

    #[test]
    fn parses_generic_type_arg_that_is_an_int() {
        let src = "<1>";
        let generics = parse_generic_type_args_no_errors(src);
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
        let reason = get_single_error_reason(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::ForbiddenNumericGenericType));
    }

    #[test]
    fn parse_arithmetic_generic_on_variable() {
        let src = "<N - 1>";
        let generics = parse_generic_type_args_no_errors(src);
        assert_eq!(generics.ordered_args[0].to_string(), "(N - 1)");
    }

    #[test]
    fn parse_var_with_turbofish_in_generic() {
        let src = "<N<1>>";
        let generics = parse_generic_type_args_no_errors(src);
        assert_eq!(generics.ordered_args[0].to_string(), "N<1>");
    }
}
