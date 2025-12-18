use crate::{
    ast::{
        GenericTypeArg, GenericTypeArgs, IdentOrQuotedType, IntegerBitSize, Path,
        UnresolvedGeneric, UnresolvedGenerics, UnresolvedType, UnresolvedTypeData,
    },
    parser::{ParserErrorReason, labels::ParsingRuleLabel},
    shared::Signedness,
    token::{Keyword, Token, TokenKind},
};

use super::{Parser, parse_many::separated_by_comma};

impl Parser<'_> {
    pub(super) fn parse_generics_disallowing_trait_bounds(&mut self) -> UnresolvedGenerics {
        self.parse_generics(false)
    }

    pub(super) fn parse_generics_allowing_trait_bounds(&mut self) -> UnresolvedGenerics {
        self.parse_generics(true)
    }

    /// Generics = ( '<' GenericsVector? '>' )?
    ///
    /// GenericsVector = Generic ( ',' Generic )* ','?
    fn parse_generics(&mut self, allow_trait_bounds: bool) -> UnresolvedGenerics {
        if !self.eat_less() {
            return Vec::new();
        }

        self.parse_many(
            "generic parameters",
            separated_by_comma().until(Token::Greater),
            |parser| parser.parse_generic_in_vector(allow_trait_bounds),
        )
    }

    fn parse_generic_in_vector(&mut self, allow_trait_bounds: bool) -> Option<UnresolvedGeneric> {
        if let Some(generic) = self.parse_generic(allow_trait_bounds) {
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
    fn parse_generic(&mut self, allow_trait_bounds: bool) -> Option<UnresolvedGeneric> {
        if let Some(generic) = self.parse_variable_generic(allow_trait_bounds) {
            return Some(generic);
        }

        self.parse_numeric_generic()
    }

    /// VariableGeneric = identifier ( ':' TraitBounds ) ?
    fn parse_variable_generic(&mut self, allow_trait_bounds: bool) -> Option<UnresolvedGeneric> {
        let ident = self.parse_ident_or_quoted()?;

        let trait_bounds = if self.eat_colon() {
            if !allow_trait_bounds {
                self.push_error(
                    ParserErrorReason::TraitBoundsNotAllowedHere,
                    self.previous_token_location,
                );
            }

            self.parse_trait_bounds()
        } else {
            Vec::new()
        };
        Some(UnresolvedGeneric::Variable(ident, trait_bounds))
    }

    fn parse_ident_or_quoted(&mut self) -> Option<IdentOrQuotedType> {
        if let Some(ident) = self.eat_ident() {
            return Some(IdentOrQuotedType::Ident(ident));
        }

        let token = self.eat_kind(TokenKind::QuotedType)?;
        match token.into_token() {
            Token::QuotedType(id) => {
                Some(IdentOrQuotedType::Quoted(id, self.previous_token_location))
            }
            _ => unreachable!(),
        }
    }

    /// NumericGeneric = 'let' identifier ':' Type
    fn parse_numeric_generic(&mut self) -> Option<UnresolvedGeneric> {
        if !self.eat_keyword(Keyword::Let) {
            return None;
        }

        let ident = self.parse_ident_or_quoted()?;

        if !self.eat_colon() {
            // If we didn't get a type after the colon, error and assume it's u32
            self.push_error(
                ParserErrorReason::MissingTypeForNumericGeneric,
                self.current_token_location,
            );
            let location = self.location_at_previous_token_end();
            let typ = UnresolvedType {
                typ: UnresolvedTypeData::integer(
                    Signedness::Unsigned,
                    IntegerBitSize::ThirtyTwo,
                    location,
                ),
                location,
            };
            return Some(UnresolvedGeneric::Numeric { ident, typ });
        }

        let mut typ = self.parse_type_or_error();

        // If we failed to parse a type, default to u32 instead of Type::Error
        // to prevent more type errors down the line
        if typ.typ == UnresolvedTypeData::Error {
            let path = Path::from_single("u32".to_string(), self.location_at_previous_token_end());
            typ.typ = UnresolvedTypeData::Named(path, GenericTypeArgs::default(), true);
        }

        Some(UnresolvedGeneric::Numeric { ident, typ })
    }

    /// GenericTypeArgs = ( '<' GenericTypeArgsVector? '>' )
    ///
    /// GenericTypeArgsVector = GenericTypeArg ( ',' GenericTypeArg )* ','?
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

            let Some(typ) = self.parse_type_or_type_expression() else {
                self.expected_label(ParsingRuleLabel::TypeOrTypeExpression);
                return None;
            };
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
        ast::{GenericTypeArgs, UnresolvedGeneric},
        parser::{
            Parser, ParserErrorReason,
            parser::tests::{
                expect_no_errors, get_single_error_reason, get_source_with_error_span,
            },
        },
    };

    fn parse_generics_no_errors(src: &str) -> Vec<UnresolvedGeneric> {
        let mut parser = Parser::for_str_with_dummy_file(src);
        let generics = parser.parse_generics(true /* allow trait bounds */);
        expect_no_errors(&parser.errors);
        generics
    }

    fn parse_generic_type_args_no_errors(src: &str) -> GenericTypeArgs {
        let mut parser = Parser::for_str_with_dummy_file(src);
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
        let src = "<A, let B: u32, C: X + Y>";
        let mut generics = parse_generics_no_errors(src);
        assert_eq!(generics.len(), 3);

        let generic = generics.remove(0);
        let UnresolvedGeneric::Variable(ident, trait_bounds) = generic else {
            panic!("Expected generic variable");
        };
        assert_eq!("A", ident.to_string());
        assert!(trait_bounds.is_empty());

        let generic = generics.remove(0);
        let UnresolvedGeneric::Numeric { ident, typ } = generic else {
            panic!("Expected generic numeric");
        };
        assert_eq!("B", ident.to_string());
        assert_eq!(typ.typ.to_string(), "u32",);

        let generic = generics.remove(0);
        let UnresolvedGeneric::Variable(ident, trait_bounds) = generic else {
            panic!("Expected generic variable");
        };
        assert_eq!("C", ident.to_string());
        assert_eq!(trait_bounds.len(), 2);

        assert_eq!(trait_bounds[0].to_string(), "X");
        assert_eq!(trait_bounds[1].to_string(), "Y");
    }

    #[test]
    fn parses_no_generic_type_args() {
        let src = "1";
        let generics = parse_generic_type_args_no_errors(src);
        assert!(generics.is_empty());
    }

    #[test]
    fn parses_generic_type_args() {
        let src = "<i32, X = Field, Y = 1>";
        let generics = parse_generic_type_args_no_errors(src);
        assert!(!generics.is_empty());
        assert_eq!(generics.ordered_args.len(), 1);
        assert_eq!(generics.ordered_args[0].to_string(), "i32");
        assert_eq!(generics.named_args.len(), 2);
        assert_eq!(generics.named_args[0].0.to_string(), "X");
        assert_eq!(generics.named_args[0].1.to_string(), "Field");
        assert_eq!(generics.named_args[1].0.to_string(), "Y");
        assert_eq!(generics.named_args[1].1.to_string(), "1");
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

    #[test]
    fn parse_generic_trait_bound_not_allowed() {
        let src = "
        N: Trait
         ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str_with_dummy_file(&src);
        parser.parse_generic(false);
        let reason = get_single_error_reason(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::TraitBoundsNotAllowedHere));
    }
}
