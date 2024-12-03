use acvm::{AcirField, FieldElement};

use crate::{
    ast::{UnresolvedType, UnresolvedTypeData, UnresolvedTypeExpression},
    parser::{labels::ParsingRuleLabel, ParserErrorReason},
    token::{Keyword, Token, TokenKind},
    QuotedType,
};

use super::{parse_many::separated_by_comma_until_right_paren, Parser};

impl<'a> Parser<'a> {
    pub(crate) fn parse_type_or_error(&mut self) -> UnresolvedType {
        if let Some(typ) = self.parse_type() {
            typ
        } else {
            self.expected_label(ParsingRuleLabel::Type);
            self.unspecified_type_at_previous_token_end()
        }
    }

    pub(crate) fn parse_type(&mut self) -> Option<UnresolvedType> {
        let start_span = self.current_token_span;
        let typ = self.parse_unresolved_type_data()?;
        let span = self.span_since(start_span);
        Some(UnresolvedType { typ, span })
    }

    fn parse_unresolved_type_data(&mut self) -> Option<UnresolvedTypeData> {
        if let Some(typ) = self.parse_primitive_type() {
            return Some(typ);
        }

        if let Some(typ) = self.parse_parentheses_type() {
            return Some(typ);
        }

        if let Some(typ) = self.parse_array_or_slice_type() {
            return Some(typ);
        }

        if let Some(typ) = self.parses_mutable_reference_type() {
            return Some(typ);
        }

        if let Some(typ) = self.parse_function_type() {
            return Some(typ);
        }

        if let Some(typ) = self.parse_trait_as_type() {
            return Some(typ);
        }

        if let Some(typ) = self.parse_as_trait_path_type() {
            return Some(typ);
        }

        if let Some(path) = self.parse_path_no_turbofish() {
            let generics = self.parse_generic_type_args();
            return Some(UnresolvedTypeData::Named(path, generics, false));
        }

        None
    }

    pub(super) fn parse_primitive_type(&mut self) -> Option<UnresolvedTypeData> {
        if let Some(typ) = self.parse_field_type() {
            return Some(typ);
        }

        if let Some(typ) = self.parse_int_type() {
            return Some(typ);
        }

        if let Some(typ) = self.parse_bool_type() {
            return Some(typ);
        }

        if let Some(typ) = self.parse_str_type() {
            return Some(typ);
        }

        if let Some(typ) = self.parse_fmtstr_type() {
            return Some(typ);
        }

        if let Some(typ) = self.parse_comptime_type() {
            return Some(typ);
        }

        if let Some(typ) = self.parse_resolved_type() {
            return Some(typ);
        }

        if let Some(typ) = self.parse_interned_type() {
            return Some(typ);
        }

        None
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
                Err(err) => {
                    self.push_error(
                        ParserErrorReason::InvalidBitSize(err.0),
                        self.previous_token_span,
                    );
                    UnresolvedTypeData::Error
                }
            });
        }

        None
    }

    fn parse_str_type(&mut self) -> Option<UnresolvedTypeData> {
        if !self.eat_keyword(Keyword::String) {
            return None;
        }

        if !self.eat_less() {
            self.expected_token(Token::Less);
            let expr =
                UnresolvedTypeExpression::Constant(FieldElement::zero(), self.current_token_span);
            return Some(UnresolvedTypeData::String(expr));
        }

        let expr = match self.parse_type_expression() {
            Ok(expr) => expr,
            Err(error) => {
                self.errors.push(error);
                UnresolvedTypeExpression::Constant(FieldElement::zero(), self.current_token_span)
            }
        };

        self.eat_or_error(Token::Greater);

        Some(UnresolvedTypeData::String(expr))
    }

    fn parse_fmtstr_type(&mut self) -> Option<UnresolvedTypeData> {
        if !self.eat_keyword(Keyword::FormatString) {
            return None;
        }

        if !self.eat_less() {
            self.expected_token(Token::Less);
            let expr =
                UnresolvedTypeExpression::Constant(FieldElement::zero(), self.current_token_span);
            let typ = UnresolvedTypeData::Error.with_span(self.span_at_previous_token_end());
            return Some(UnresolvedTypeData::FormatString(expr, Box::new(typ)));
        }

        let expr = match self.parse_type_expression() {
            Ok(expr) => expr,
            Err(error) => {
                self.errors.push(error);
                UnresolvedTypeExpression::Constant(FieldElement::zero(), self.current_token_span)
            }
        };

        if !self.eat_commas() {
            self.expected_token(Token::Comma);
        }

        let typ = self.parse_type_or_error();

        self.eat_or_error(Token::Greater);

        Some(UnresolvedTypeData::FormatString(expr, Box::new(typ)))
    }

    fn parse_comptime_type(&mut self) -> Option<UnresolvedTypeData> {
        if self.eat_keyword(Keyword::Expr) {
            return Some(UnresolvedTypeData::Quoted(QuotedType::Expr));
        }
        if self.eat_keyword(Keyword::Quoted) {
            return Some(UnresolvedTypeData::Quoted(QuotedType::Quoted));
        }
        if self.eat_keyword(Keyword::TopLevelItem) {
            return Some(UnresolvedTypeData::Quoted(QuotedType::TopLevelItem));
        }
        if self.eat_keyword(Keyword::TypeType) {
            return Some(UnresolvedTypeData::Quoted(QuotedType::Type));
        }
        if self.eat_keyword(Keyword::TypedExpr) {
            return Some(UnresolvedTypeData::Quoted(QuotedType::TypedExpr));
        }
        if self.eat_keyword(Keyword::StructDefinition) {
            return Some(UnresolvedTypeData::Quoted(QuotedType::StructDefinition));
        }
        if self.eat_keyword(Keyword::TraitConstraint) {
            return Some(UnresolvedTypeData::Quoted(QuotedType::TraitConstraint));
        }
        if self.eat_keyword(Keyword::TraitDefinition) {
            return Some(UnresolvedTypeData::Quoted(QuotedType::TraitDefinition));
        }
        if self.eat_keyword(Keyword::TraitImpl) {
            return Some(UnresolvedTypeData::Quoted(QuotedType::TraitImpl));
        }
        if self.eat_keyword(Keyword::UnresolvedType) {
            return Some(UnresolvedTypeData::Quoted(QuotedType::UnresolvedType));
        }
        if self.eat_keyword(Keyword::FunctionDefinition) {
            return Some(UnresolvedTypeData::Quoted(QuotedType::FunctionDefinition));
        }
        if self.eat_keyword(Keyword::Module) {
            return Some(UnresolvedTypeData::Quoted(QuotedType::Module));
        }
        if self.eat_keyword(Keyword::CtString) {
            return Some(UnresolvedTypeData::Quoted(QuotedType::CtString));
        }
        None
    }

    fn parse_function_type(&mut self) -> Option<UnresolvedTypeData> {
        let unconstrained = self.eat_keyword(Keyword::Unconstrained);

        if !self.eat_keyword(Keyword::Fn) {
            if unconstrained {
                self.expected_token(Token::Keyword(Keyword::Fn));
                return Some(UnresolvedTypeData::Function(
                    Vec::new(),
                    Box::new(self.unspecified_type_at_previous_token_end()),
                    Box::new(self.unspecified_type_at_previous_token_end()),
                    unconstrained,
                ));
            }

            return None;
        }

        let env = if self.eat_left_bracket() {
            let typ = self.parse_type_or_error();
            self.eat_or_error(Token::RightBracket);
            typ
        } else {
            UnresolvedTypeData::Unit.with_span(self.span_at_previous_token_end())
        };

        if !self.eat_left_paren() {
            self.expected_token(Token::LeftParen);

            return Some(UnresolvedTypeData::Function(
                Vec::new(),
                Box::new(self.unspecified_type_at_previous_token_end()),
                Box::new(self.unspecified_type_at_previous_token_end()),
                unconstrained,
            ));
        }

        let args = self.parse_many(
            "parameters",
            separated_by_comma_until_right_paren(),
            Self::parse_parameter,
        );

        let ret = if self.eat(Token::Arrow) {
            self.parse_type_or_error()
        } else {
            self.expected_token(Token::Arrow);
            UnresolvedTypeData::Unit.with_span(self.span_at_previous_token_end())
        };

        Some(UnresolvedTypeData::Function(args, Box::new(ret), Box::new(env), unconstrained))
    }

    fn parse_parameter(&mut self) -> Option<UnresolvedType> {
        let typ = self.parse_type_or_error();
        if let UnresolvedTypeData::Error = typ.typ {
            None
        } else {
            Some(typ)
        }
    }

    fn parse_trait_as_type(&mut self) -> Option<UnresolvedTypeData> {
        if !self.eat_keyword(Keyword::Impl) {
            return None;
        }

        let Some(path) = self.parse_path_no_turbofish() else {
            self.expected_label(ParsingRuleLabel::Path);
            return None;
        };

        let generics = self.parse_generic_type_args();

        Some(UnresolvedTypeData::TraitAsType(path, generics))
    }

    fn parse_as_trait_path_type(&mut self) -> Option<UnresolvedTypeData> {
        let as_trait_path = self.parse_as_trait_path()?;
        Some(UnresolvedTypeData::AsTraitPath(Box::new(as_trait_path)))
    }

    fn parse_resolved_type(&mut self) -> Option<UnresolvedTypeData> {
        if let Some(token) = self.eat_kind(TokenKind::QuotedType) {
            match token.into_token() {
                Token::QuotedType(id) => {
                    return Some(UnresolvedTypeData::Resolved(id));
                }
                _ => unreachable!(),
            }
        }

        None
    }

    pub(super) fn parse_interned_type(&mut self) -> Option<UnresolvedTypeData> {
        if let Some(token) = self.eat_kind(TokenKind::InternedUnresolvedTypeData) {
            match token.into_token() {
                Token::InternedUnresolvedTypeData(id) => {
                    return Some(UnresolvedTypeData::Interned(id));
                }
                _ => unreachable!(),
            }
        }

        None
    }

    fn parses_mutable_reference_type(&mut self) -> Option<UnresolvedTypeData> {
        if self.eat(Token::Ampersand) {
            if !self.eat_keyword(Keyword::Mut) {
                self.expected_mut_after_ampersand();
            }

            return Some(UnresolvedTypeData::MutableReference(Box::new(
                self.parse_type_or_error(),
            )));
        };

        None
    }

    fn parse_array_or_slice_type(&mut self) -> Option<UnresolvedTypeData> {
        if !self.eat_left_bracket() {
            return None;
        }

        let typ = self.parse_type_or_error();

        if self.eat_semicolon() {
            match self.parse_type_expression() {
                Ok(expr) => {
                    self.eat_or_error(Token::RightBracket);
                    Some(UnresolvedTypeData::Array(expr, Box::new(typ)))
                }
                Err(error) => {
                    self.errors.push(error);
                    self.eat_or_error(Token::RightBracket);
                    Some(UnresolvedTypeData::Slice(Box::new(typ)))
                }
            }
        } else {
            self.eat_or_error(Token::RightBracket);
            Some(UnresolvedTypeData::Slice(Box::new(typ)))
        }
    }

    fn parse_parentheses_type(&mut self) -> Option<UnresolvedTypeData> {
        if !self.eat_left_paren() {
            return None;
        }

        if self.eat_right_paren() {
            return Some(UnresolvedTypeData::Unit);
        }

        let (mut types, trailing_comma) = self.parse_many_return_trailing_separator_if_any(
            "tuple elements",
            separated_by_comma_until_right_paren(),
            Self::parse_type_in_list,
        );

        Some(if types.len() == 1 && !trailing_comma {
            UnresolvedTypeData::Parenthesized(Box::new(types.remove(0)))
        } else {
            UnresolvedTypeData::Tuple(types)
        })
    }

    pub(super) fn parse_type_in_list(&mut self) -> Option<UnresolvedType> {
        if let Some(typ) = self.parse_type() {
            Some(typ)
        } else {
            self.expected_label(ParsingRuleLabel::Type);
            None
        }
    }

    /// OptionalTypeAnnotation = ( ':' Type )?
    pub(super) fn parse_optional_type_annotation(&mut self) -> UnresolvedType {
        if self.eat_colon() {
            self.parse_type_or_error()
        } else {
            self.unspecified_type_at_previous_token_end()
        }
    }

    pub(super) fn unspecified_type_at_previous_token_end(&self) -> UnresolvedType {
        UnresolvedTypeData::Unspecified.with_span(self.span_at_previous_token_end())
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use crate::{
        ast::{IntegerBitSize, Signedness, UnresolvedType, UnresolvedTypeData},
        parser::{
            parser::tests::{expect_no_errors, get_single_error, get_source_with_error_span},
            Parser,
        },
        QuotedType,
    };

    fn parse_type_no_errors(src: &str) -> UnresolvedType {
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type_or_error();
        expect_no_errors(&parser.errors);
        typ
    }

    #[test]
    fn parses_unit_type() {
        let src = "()";
        let typ = parse_type_no_errors(src);
        assert!(matches!(typ.typ, UnresolvedTypeData::Unit));
    }

    #[test]
    fn parses_bool_type() {
        let src = "bool";
        let typ = parse_type_no_errors(src);
        assert!(matches!(typ.typ, UnresolvedTypeData::Bool));
    }

    #[test]
    fn parses_int_type() {
        let src = "u32";
        let typ = parse_type_no_errors(src);
        assert!(matches!(
            typ.typ,
            UnresolvedTypeData::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo)
        ));
    }

    #[test]
    fn parses_field_type() {
        let src = "Field";
        let typ = parse_type_no_errors(src);
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parses_str_type() {
        let src = "str<10>";
        let typ = parse_type_no_errors(src);
        let UnresolvedTypeData::String(expr) = typ.typ else { panic!("Expected a string type") };
        assert_eq!(expr.to_string(), "10");
    }

    #[test]
    fn parses_fmtstr_type() {
        let src = "fmtstr<10, T>";
        let typ = parse_type_no_errors(src);
        let UnresolvedTypeData::FormatString(expr, typ) = typ.typ else {
            panic!("Expected a format string type")
        };
        assert_eq!(expr.to_string(), "10");
        assert_eq!(typ.to_string(), "T");
    }

    #[test]
    fn parses_comptime_types() {
        for quoted_type in QuotedType::iter() {
            let src = quoted_type.to_string();
            let typ = parse_type_no_errors(&src);
            let UnresolvedTypeData::Quoted(parsed_qouted_type) = typ.typ else {
                panic!("Expected a quoted type for {}", quoted_type)
            };
            assert_eq!(parsed_qouted_type, quoted_type);
        }
    }

    #[test]
    fn parses_tuple_type() {
        let src = "(Field, bool)";
        let typ = parse_type_no_errors(src);
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
        let typ = parse_type_no_errors(src);
        let UnresolvedTypeData::Tuple(mut types) = typ.typ else { panic!("Expected a tuple type") };
        assert_eq!(types.len(), 1);

        let typ = types.remove(0);
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parses_parenthesized_type() {
        let src = "(Field)";
        let typ = parse_type_no_errors(src);
        let UnresolvedTypeData::Parenthesized(typ) = typ.typ else {
            panic!("Expected a parenthesized type")
        };
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parses_unclosed_parentheses_type() {
        let src = "(Field";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type_or_error();
        assert_eq!(parser.errors.len(), 1);
        let UnresolvedTypeData::Parenthesized(typ) = typ.typ else {
            panic!("Expected a parenthesized type")
        };
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parses_mutable_reference_type() {
        let src = "&mut Field";
        let typ = parse_type_no_errors(src);
        let UnresolvedTypeData::MutableReference(typ) = typ.typ else {
            panic!("Expected a mutable reference type")
        };
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parses_named_type_no_generics() {
        let src = "foo::Bar";
        let typ = parse_type_no_errors(src);
        let UnresolvedTypeData::Named(path, generics, _) = typ.typ else {
            panic!("Expected a named type")
        };
        assert_eq!(path.to_string(), "foo::Bar");
        assert!(generics.is_empty());
    }

    #[test]
    fn parses_slice_type() {
        let src = "[Field]";
        let typ = parse_type_no_errors(src);
        let UnresolvedTypeData::Slice(typ) = typ.typ else { panic!("Expected a slice type") };
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn errors_if_missing_right_bracket_after_slice_type() {
        let src = "
        [Field 
              ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        parser.parse_type();
        let error = get_single_error(&parser.errors, span);
        assert_eq!(error.to_string(), "Expected a ']' but found end of input");
    }

    #[test]
    fn parses_array_type() {
        let src = "[Field; 10]";
        let typ = parse_type_no_errors(src);
        let UnresolvedTypeData::Array(expr, typ) = typ.typ else {
            panic!("Expected an array type")
        };
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
        assert_eq!(expr.to_string(), "10");
    }

    #[test]
    fn parses_empty_function_type() {
        let src = "fn() -> Field";
        let typ = parse_type_no_errors(src);
        let UnresolvedTypeData::Function(args, ret, env, unconstrained) = typ.typ else {
            panic!("Expected a function type")
        };
        assert!(args.is_empty());
        assert_eq!(ret.typ.to_string(), "Field");
        assert!(matches!(env.typ, UnresolvedTypeData::Unit));
        assert!(!unconstrained);
    }

    #[test]
    fn parses_function_type_with_arguments() {
        let src = "fn(Field, bool) -> Field";
        let typ = parse_type_no_errors(src);
        let UnresolvedTypeData::Function(args, _ret, _env, _unconstrained) = typ.typ else {
            panic!("Expected a function type")
        };
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].typ.to_string(), "Field");
        assert_eq!(args[1].typ.to_string(), "bool");
    }

    #[test]
    fn parses_function_type_with_return_type() {
        let src = "fn() -> Field";
        let typ = parse_type_no_errors(src);
        let UnresolvedTypeData::Function(_args, ret, _env, _unconstrained) = typ.typ else {
            panic!("Expected a function type")
        };
        assert_eq!(ret.typ.to_string(), "Field");
    }

    #[test]
    fn parses_function_type_with_env() {
        let src = "fn[Field]() -> Field";
        let typ = parse_type_no_errors(src);
        let UnresolvedTypeData::Function(_args, _ret, env, _unconstrained) = typ.typ else {
            panic!("Expected a function type")
        };
        assert_eq!(env.typ.to_string(), "Field");
    }

    #[test]
    fn parses_unconstrained_function_type() {
        let src = "unconstrained fn() -> Field";
        let typ = parse_type_no_errors(src);
        let UnresolvedTypeData::Function(_args, _ret, _env, unconstrained) = typ.typ else {
            panic!("Expected a function type")
        };
        assert!(unconstrained);
    }

    #[test]
    fn parses_trait_as_type_no_generics() {
        let src = "impl foo::Bar";
        let typ = parse_type_no_errors(src);
        let UnresolvedTypeData::TraitAsType(path, generics) = typ.typ else {
            panic!("Expected trait as type")
        };
        assert_eq!(path.to_string(), "foo::Bar");
        assert!(generics.is_empty());
    }

    #[test]
    fn parses_as_trait_path() {
        let src = "<Field as foo::Bar>::baz";
        let typ = parse_type_no_errors(src);
        let UnresolvedTypeData::AsTraitPath(as_trait_path) = typ.typ else {
            panic!("Expected as_trait_path")
        };
        assert_eq!(as_trait_path.typ.typ.to_string(), "Field");
        assert_eq!(as_trait_path.trait_path.to_string(), "foo::Bar");
        assert!(as_trait_path.trait_generics.is_empty());
        assert_eq!(as_trait_path.impl_item.to_string(), "baz");
    }
}
