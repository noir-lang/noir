use noirc_frontend::{
    ast::{UnresolvedType, UnresolvedTypeData},
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
                self.write_current_token();
                self.bump();
            }
            UnresolvedTypeData::Array(type_expr, typ) => {
                self.write_left_bracket();
                self.format_type(*typ);
                self.write_semicolon();
                self.write_space();
                self.format_type_expression(type_expr);
                self.write_right_bracket();
            }
            UnresolvedTypeData::Slice(_unresolved_type) => todo!("Format slice type"),
            UnresolvedTypeData::Expression(_unresolved_type_expression) => {
                todo!("Format expression type")
            }
            UnresolvedTypeData::String(_unresolved_type_expression) => todo!("Format string type"),
            UnresolvedTypeData::FormatString(_unresolved_type_expression, _unresolved_type) => {
                todo!("Format format string type")
            }
            UnresolvedTypeData::Parenthesized(_unresolved_type) => {
                todo!("Format parenthesized type")
            }
            UnresolvedTypeData::Named(path, generic_type_args, _) => {
                self.format_path(path);
                if !generic_type_args.is_empty() {
                    todo!("Format named type generics");
                }
            }
            UnresolvedTypeData::TraitAsType(_path, _generic_type_args) => {
                todo!("Format trait as type")
            }
            UnresolvedTypeData::MutableReference(typ) => {
                self.write_token(Token::Ampersand);
                self.write_keyword(Keyword::Mut);
                self.write_space();
                self.format_type(*typ);
            }
            UnresolvedTypeData::Tuple(_vec) => todo!("Format tuple type"),
            UnresolvedTypeData::Function(_vec, _unresolved_type, _unresolved_type1, _) => {
                todo!("Format function type")
            }
            UnresolvedTypeData::Quoted(_quoted_type) => todo!("Format quoted type"),
            UnresolvedTypeData::AsTraitPath(_as_trait_path) => todo!("Format as trait path"),
            UnresolvedTypeData::Resolved(..)
            | UnresolvedTypeData::Interned(..)
            | UnresolvedTypeData::Error => unreachable!("Should not be present in the AST"),
            UnresolvedTypeData::Unspecified => panic!("Unspecified type should have been handled"),
        }
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
    fn format_mutable_reference_type() {
        let src = " &  mut  Field ";
        let expected = "&mut Field";
        assert_format_type(src, expected);
    }
}
