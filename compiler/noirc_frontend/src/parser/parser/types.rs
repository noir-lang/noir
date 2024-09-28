use noirc_errors::Span;

use crate::{
    ast::{Ident, UnresolvedType, UnresolvedTypeData, UnresolvedTypeExpression},
    parser::ParserErrorReason,
    token::{Keyword, Token, TokenKind},
    QuotedType,
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
        if let Some(typ) = self.parse_primitive_type() {
            return typ;
        }

        if let Some(typ) = self.parse_parentheses_type() {
            return typ;
        }

        if let Some(typ) = self.parse_array_or_slice_type() {
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

    fn parse_primitive_type(&mut self) -> Option<UnresolvedTypeData> {
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
                Err(_) => {
                    // TODO: error
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
            self.push_error(ParserErrorReason::ExpectedStringTypeLength, self.current_token_span);
            let expr = UnresolvedTypeExpression::Constant(0, self.current_token_span);
            return Some(UnresolvedTypeData::String(expr));
        }

        let expr = match self.parse_type_expression() {
            Ok(expr) => expr,
            Err(error) => {
                self.errors.push(error);
                UnresolvedTypeExpression::Constant(0, self.current_token_span)
            }
        };

        if !self.eat_greater() {
            self.push_error(
                ParserErrorReason::ExpectedGreaterAfterStringTypeLength,
                self.current_token_span,
            );
        }

        Some(UnresolvedTypeData::String(expr))
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

    fn parse_interned_type(&mut self) -> Option<UnresolvedTypeData> {
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
                // TODO: error
            }
            return Some(UnresolvedTypeData::MutableReference(Box::new(self.parse_type())));
        };

        None
    }

    fn parse_array_or_slice_type(&mut self) -> Option<UnresolvedTypeData> {
        if !self.eat_left_bracket() {
            return None;
        }

        let typ = self.parse_type();

        if self.eat_semicolon() {
            match self.parse_type_expression() {
                Ok(expr) => {
                    if !self.eat_right_bracket() {
                        self.push_error(ParserErrorReason::ExpectedBracketAfterArrayType, typ.span);
                    }
                    Some(UnresolvedTypeData::Array(expr, Box::new(typ)))
                }
                Err(error) => {
                    self.errors.push(error);
                    if !self.eat_right_bracket() {
                        self.push_error(ParserErrorReason::ExpectedBracketAfterArrayType, typ.span);
                    }

                    Some(UnresolvedTypeData::Slice(Box::new(typ)))
                }
            }
        } else {
            if !self.eat_right_bracket() {
                self.push_error(ParserErrorReason::ExpectedBracketAfterSliceType, typ.span);
            }

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
    use strum::IntoEnumIterator;

    use crate::{
        ast::{IntegerBitSize, Signedness, UnresolvedTypeData},
        parser::{
            parser::tests::{get_single_error, get_source_with_error_span},
            Parser, ParserErrorReason,
        },
        QuotedType,
    };

    #[test]
    fn parses_unit_type() {
        let src = "()";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type();
        assert!(parser.errors.is_empty());
        assert!(matches!(typ.typ, UnresolvedTypeData::Unit));
    }

    #[test]
    fn parses_bool_type() {
        let src = "bool";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type();
        assert!(parser.errors.is_empty());
        assert!(matches!(typ.typ, UnresolvedTypeData::Bool));
    }

    #[test]
    fn parses_int_type() {
        let src = "u32";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type();
        assert!(parser.errors.is_empty());
        assert!(matches!(
            typ.typ,
            UnresolvedTypeData::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo)
        ));
    }

    #[test]
    fn parses_field_type() {
        let src = "Field";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type();
        assert!(parser.errors.is_empty());
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parses_str_type() {
        let src = "str<10>";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeData::String(expr) = typ.typ else { panic!("Expected a string type") };
        assert_eq!(expr.to_string(), "10");
    }

    #[test]
    fn parses_comptime_types() {
        for quoted_type in QuotedType::iter() {
            let src = quoted_type.to_string();
            let mut parser = Parser::for_str(&src);
            let typ = parser.parse_type();
            assert!(parser.errors.is_empty());
            let UnresolvedTypeData::Quoted(parsed_qouted_type) = typ.typ else {
                panic!("Expected a quoted type for {}", quoted_type.to_string())
            };
            assert_eq!(parsed_qouted_type, quoted_type);
        }
    }

    #[test]
    fn parses_tuple_type() {
        let src = "(Field, bool)";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type();
        assert!(parser.errors.is_empty());
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
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeData::Tuple(mut types) = typ.typ else { panic!("Expected a tuple type") };
        assert_eq!(types.len(), 1);

        let typ = types.remove(0);
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parses_parenthesized_type() {
        let src = "(Field)";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeData::Parenthesized(typ) = typ.typ else {
            panic!("Expected a parenthesized type")
        };
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parses_unclosed_parentheses_type() {
        let src = "(Field";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type();
        assert!(parser.errors.is_empty()); // TODO: there should be an error here
        let UnresolvedTypeData::Parenthesized(typ) = typ.typ else {
            panic!("Expected a parenthesized type")
        };
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parses_mutable_reference_type() {
        let src = "&mut Field";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeData::MutableReference(typ) = typ.typ else {
            panic!("Expected a mutable reference type")
        };
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parses_named_type_no_generics() {
        let src = "foo::Bar";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeData::Named(path, generics, _) = typ.typ else {
            panic!("Expected a named type")
        };
        assert_eq!(path.to_string(), "foo::Bar");
        assert!(generics.is_empty());
    }

    #[test]
    fn parses_slice_type() {
        let src = "[Field]";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeData::Slice(typ) = typ.typ else { panic!("Expected a slice type") };
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn errors_if_missing_right_bracket_after_slice_type() {
        let src = "
        [Field
         ^^^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        parser.parse_type();
        let reason = get_single_error(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::ExpectedBracketAfterSliceType));
    }

    #[test]
    fn parses_array_type() {
        let src = "[Field; 10]";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_type();
        assert!(parser.errors.is_empty());
        let UnresolvedTypeData::Array(expr, typ) = typ.typ else {
            panic!("Expected an array type")
        };
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
        assert_eq!(expr.to_string(), "10");
    }
}
