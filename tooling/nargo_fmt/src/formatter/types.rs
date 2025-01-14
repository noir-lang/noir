use noirc_frontend::{
    ast::{AsTraitPath, UnresolvedType, UnresolvedTypeData},
    token::{Keyword, Token},
};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_type(&mut self, typ: UnresolvedType) {
        self.skip_comments_and_whitespace();

        match typ.typ {
            UnresolvedTypeData::Unit => {
                self.write_left_paren();
                self.write_right_paren();
            }
            UnresolvedTypeData::Bool => {
                self.write_keyword(Keyword::Bool);
            }
            UnresolvedTypeData::Integer(..) | UnresolvedTypeData::FieldElement => {
                self.write_current_token_and_bump();
            }
            UnresolvedTypeData::Array(type_expr, typ) => {
                self.write_left_bracket();
                self.format_type(*typ);
                self.write_semicolon();
                self.write_space();
                self.format_type_expression(type_expr);
                self.write_right_bracket();
            }
            UnresolvedTypeData::Slice(typ) => {
                self.write_left_bracket();
                self.format_type(*typ);
                self.write_right_bracket();
            }
            UnresolvedTypeData::Expression(type_expr) => {
                self.format_type_expression(type_expr);
            }
            UnresolvedTypeData::String(type_expr) => {
                self.write_keyword(Keyword::String);
                self.write_token(Token::Less);
                self.format_type_expression(type_expr);
                self.write_token(Token::Greater);
            }
            UnresolvedTypeData::FormatString(type_expr, typ) => {
                self.write_keyword(Keyword::FormatString);
                self.write_token(Token::Less);
                self.format_type_expression(type_expr);
                self.write_comma();
                self.write_space();
                self.format_type(*typ);
                self.write_token(Token::Greater);
            }
            UnresolvedTypeData::Parenthesized(typ) => {
                self.write_left_paren();
                self.format_type(*typ);
                self.write_right_paren();
            }
            UnresolvedTypeData::Named(path, generic_type_args, _) => {
                self.format_path(path);
                if !generic_type_args.is_empty() {
                    self.skip_comments_and_whitespace();

                    // Apparently some Named types with generics have `::` before the generics
                    // while others don't, so we have to account for both cases.
                    if self.is_at(Token::DoubleColon) {
                        self.write_token(Token::DoubleColon);
                    }
                    self.format_generic_type_args(generic_type_args);
                }
            }
            UnresolvedTypeData::TraitAsType(path, generic_type_args) => {
                self.write_keyword(Keyword::Impl);
                self.write_space();
                self.format_path(path);
                self.format_generic_type_args(generic_type_args);
            }
            UnresolvedTypeData::MutableReference(typ) => {
                self.write_token(Token::Ampersand);
                self.write_keyword(Keyword::Mut);
                self.write_space();
                self.format_type(*typ);
            }
            UnresolvedTypeData::Tuple(types) => {
                let types_len = types.len();

                self.write_left_paren();
                for (index, typ) in types.into_iter().enumerate() {
                    if index > 0 {
                        self.write_comma();
                        self.write_space();
                    }
                    self.format_type(typ);
                }

                self.skip_comments_and_whitespace();
                if self.is_at(Token::Comma) {
                    if types_len == 1 {
                        self.write_comma();
                    } else {
                        self.bump();
                    }
                }

                self.write_right_paren();
            }
            UnresolvedTypeData::Function(args, return_type, env, unconstrained) => {
                if unconstrained {
                    self.write_keyword(Keyword::Unconstrained);
                    self.write_space();
                }

                self.write_keyword(Keyword::Fn);
                self.skip_comments_and_whitespace();

                if self.is_at(Token::LeftBracket) {
                    self.write_left_bracket();
                    self.format_type(*env);
                    self.write_right_bracket();
                }

                self.write_left_paren();
                for (index, arg) in args.into_iter().enumerate() {
                    if index > 0 {
                        self.write_comma();
                        self.write_space();
                    }
                    self.format_type(arg);
                }

                self.skip_comments_and_whitespace();
                // Remove trailing comma if there's one
                if self.is_at(Token::Comma) {
                    self.bump();
                }

                self.write_right_paren();
                self.skip_comments_and_whitespace();
                if self.is_at(Token::Arrow) {
                    self.write_space();
                    self.write_token(Token::Arrow);
                    self.write_space();
                    self.format_type(*return_type);
                }
            }
            UnresolvedTypeData::Quoted(..) => {
                self.write_current_token_and_bump();
            }
            UnresolvedTypeData::AsTraitPath(as_trait_path) => {
                self.format_as_trait_path(*as_trait_path);
            }
            UnresolvedTypeData::Resolved(..)
            | UnresolvedTypeData::Interned(..)
            | UnresolvedTypeData::Error => unreachable!("Should not be present in the AST"),
            UnresolvedTypeData::Unspecified => panic!("Unspecified type should have been handled"),
        }
    }

    pub(super) fn format_as_trait_path(&mut self, as_trait_path: AsTraitPath) {
        self.write_token(Token::Less);
        self.format_type(as_trait_path.typ);
        self.write_space();
        self.write_keyword(Keyword::As);
        self.write_space();
        self.format_path(as_trait_path.trait_path);
        self.format_generic_type_args(as_trait_path.trait_generics);
        self.write_token(Token::Greater);
        self.write_token(Token::DoubleColon);
        self.write_identifier(as_trait_path.impl_item);
    }
}

#[cfg(test)]
mod tests {
    use noirc_frontend::parser;

    use crate::Config;

    fn assert_format_type(src: &str, expected: &str) {
        let module_src = format!("type X = {};", src);
        let (parsed_module, errors) = parser::parse_program(&module_src);
        if !errors.is_empty() {
            panic!("Expected no errors, got: {:?}", errors);
        }
        let result = crate::format(&module_src, parsed_module, &Config::default());
        let type_result = &result["type X = ".len()..];
        let type_result = &type_result[..type_result.len() - 2];
        similar_asserts::assert_eq!(type_result, expected);

        let (parsed_module, errors) = parser::parse_program(&result);
        if !errors.is_empty() {
            panic!("Expected no errors in idempotent check, got: {:?}", errors);
        }
        let result = crate::format(&result, parsed_module, &Config::default());
        let type_result = &result["type X = ".len()..];
        let type_result = &type_result[..type_result.len() - 2];
        similar_asserts::assert_eq!(type_result, expected);
    }

    #[test]
    fn format_unit_type() {
        let src = " ( ) ";
        let expected = "()";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_bool_type() {
        let src = " bool ";
        let expected = "bool";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_integer_type() {
        let src = " i32 ";
        let expected = "i32";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_named_type() {
        let src = " foo :: bar :: Baz ";
        let expected = "foo::bar::Baz";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_named_type_with_generics() {
        let src = " foo :: bar < A,  B  =  Field , C = i32 , D , >";
        let expected = "foo::bar<A, B = Field, C = i32, D>";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_array_type_with_constant() {
        let src = " [ Field ; 1 ] ";
        let expected = "[Field; 1]";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_array_type_with_var() {
        let src = " [ Field ; LEN ] ";
        let expected = "[Field; LEN]";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_array_type_with_binary() {
        let src = " [ Field ; 1+2 ] ";
        let expected = "[Field; 1 + 2]";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_array_type_with_parenthesized() {
        let src = " [ Field ; ( 1 + 2 ) * ( 3 + 4 )  ] ";
        let expected = "[Field; (1 + 2) * (3 + 4)]";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_slice_type() {
        let src = " [ Field  ] ";
        let expected = "[Field]";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_mutable_reference_type() {
        let src = " &  mut  Field ";
        let expected = "&mut Field";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_parenthesized_type() {
        let src = " ( Field )";
        let expected = "(Field)";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_simple_function_type() {
        let src = " fn ( ) -> Field ";
        let expected = "fn() -> Field";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_function_type_with_args_and_unconstrained() {
        let src = "  unconstrained  fn  (  Field , i32 , ) -> Field ";
        let expected = "unconstrained fn(Field, i32) -> Field";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_function_type_with_env() {
        let src = "  fn  [ Env ] ( ) -> Field ";
        let expected = "fn[Env]() -> Field";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_tuple_type_one_type() {
        let src = " ( Field , )";
        let expected = "(Field,)";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_tuple_type_two_types() {
        let src = " ( Field ,  i32 , )";
        let expected = "(Field, i32)";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_trait_as_type() {
        let src = " impl Foo < Bar > ";
        let expected = "impl Foo<Bar>";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_string_type() {
        let src = "str < 6 >";
        let expected = "str<6>";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_fmtstr_type() {
        let src = "fmtstr < 6, ( ) >";
        let expected = "fmtstr<6, ()>";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_quoted_type() {
        let src = " Expr ";
        let expected = "Expr";
        assert_format_type(src, expected);
    }

    #[test]
    fn format_as_trait_path_type() {
        let src = " < Field as foo :: Bar> :: baz ";
        let expected = "<Field as foo::Bar>::baz";
        assert_format_type(src, expected);
    }
}
