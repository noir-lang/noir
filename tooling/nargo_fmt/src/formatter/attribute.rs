use noirc_frontend::token::{
    Attribute, Attributes, FunctionAttribute, MetaAttribute, SecondaryAttribute, TestScope, Token,
};

use crate::chunks::ChunkGroup;

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_attributes(&mut self, attributes: Attributes) {
        let mut all_attributes = Vec::new();
        for attribute in attributes.secondary {
            all_attributes.push(Attribute::Secondary(attribute));
        }
        if let Some((function_attribute, index)) = attributes.function {
            all_attributes.insert(index, Attribute::Function(function_attribute));
        }
        for attribute in all_attributes {
            self.format_attribute(attribute);
        }
    }

    pub(super) fn format_secondary_attributes(&mut self, attributes: Vec<SecondaryAttribute>) {
        for attribute in attributes {
            self.format_secondary_attribute(attribute);
        }
    }

    fn format_attribute(&mut self, attribute: Attribute) {
        match attribute {
            Attribute::Function(function_attribute) => {
                self.format_function_attribute(function_attribute);
            }
            Attribute::Secondary(secondary_attribute) => {
                self.format_secondary_attribute(secondary_attribute);
            }
        }
    }

    fn format_function_attribute(&mut self, attribute: FunctionAttribute) {
        self.skip_comments_and_whitespace();
        self.write_indentation();

        if !matches!(self.token, Token::AttributeStart { .. }) {
            panic!("Expected attribute start, got: {:?}", self.token);
        }

        match attribute {
            FunctionAttribute::Foreign(_)
            | FunctionAttribute::Builtin(_)
            | FunctionAttribute::Oracle(_) => self.format_one_arg_attribute(),
            FunctionAttribute::Test(test_scope) => self.format_test_attribute(test_scope),
            FunctionAttribute::Fold
            | FunctionAttribute::NoPredicates
            | FunctionAttribute::InlineAlways => self.format_no_args_attribute(),
        }

        self.write_line();
    }

    pub(super) fn format_secondary_attribute(&mut self, attribute: SecondaryAttribute) {
        self.skip_comments_and_whitespace();
        self.write_indentation();

        if !matches!(self.token, Token::AttributeStart { .. }) {
            panic!("Expected attribute start, got: {:?}", self.token);
        }

        match attribute {
            SecondaryAttribute::Deprecated(message) => {
                self.format_deprecated_attribute(message);
            }
            SecondaryAttribute::ContractLibraryMethod
            | SecondaryAttribute::Export
            | SecondaryAttribute::Varargs
            | SecondaryAttribute::UseCallersScope => {
                self.format_no_args_attribute();
            }
            SecondaryAttribute::Field(_)
            | SecondaryAttribute::Abi(_)
            | SecondaryAttribute::Allow(_) => {
                self.format_one_arg_attribute();
            }
            SecondaryAttribute::Tag(custom_attribute) => {
                self.write_and_skip_span_without_formatting(custom_attribute.span);
            }
            SecondaryAttribute::Meta(meta_attribute) => {
                self.format_meta_attribute(meta_attribute);
            }
        }

        self.write_line();
    }

    fn format_deprecated_attribute(&mut self, message: Option<String>) {
        self.write_current_token_and_bump(); // #[
        self.skip_comments_and_whitespace();
        if message.is_some() {
            self.write_current_token_and_bump(); // deprecated
            self.write_left_paren(); // (
            self.skip_comments_and_whitespace(); // message
            self.write_current_token_and_bump(); // )
            self.write_right_paren();
        } else {
            self.write_current_token_and_bump();
        }
        self.write_right_bracket(); // ]
    }

    fn format_test_attribute(&mut self, test_scope: TestScope) {
        self.write_current_token_and_bump(); // #[
        self.skip_comments_and_whitespace();
        self.write_current_token_and_bump(); // test

        match test_scope {
            TestScope::None => (),
            TestScope::ShouldFailWith { reason: None } => {
                self.write_left_paren(); // (
                self.skip_comments_and_whitespace();
                self.write_current_token_and_bump(); // should_fail
                self.write_right_paren(); // )
            }
            TestScope::ShouldFailWith { reason: Some(..) } => {
                self.write_left_paren(); // (
                self.skip_comments_and_whitespace();
                self.write_current_token_and_bump(); // should_fail_with
                self.write_space();
                self.write_token(Token::Assign);
                self.write_space();
                self.skip_comments_and_whitespace();
                self.write_current_token_and_bump(); // "reason"
                self.write_right_paren(); // )
            }
        }

        self.write_right_bracket(); // ]
    }

    fn format_meta_attribute(&mut self, meta_attribute: MetaAttribute) {
        self.write_current_token_and_bump(); // #[
        self.skip_comments_and_whitespace();
        self.format_path(meta_attribute.name);
        self.skip_comments_and_whitespace();
        if self.is_at(Token::LeftParen) {
            let comments_count_before_arguments = self.written_comments_count;
            let has_arguments = !meta_attribute.arguments.is_empty();

            let mut chunk_formatter = self.chunk_formatter();
            let mut group = ChunkGroup::new();
            group.text(chunk_formatter.chunk(|formatter| {
                formatter.write_left_paren();
            }));
            chunk_formatter.format_expressions_separated_by_comma(
                meta_attribute.arguments,
                false, // force trailing comma
                &mut group,
            );
            group.text(chunk_formatter.chunk(|formatter| {
                formatter.write_right_paren();
            }));
            if has_arguments || self.written_comments_count > comments_count_before_arguments {
                self.format_chunk_group(group);
            }
        }
        self.write_right_bracket();
    }

    fn format_no_args_attribute(&mut self) {
        self.write_current_token_and_bump(); // #[
        self.skip_comments_and_whitespace();
        self.write_current_token_and_bump(); // name
        self.write_right_bracket(); // ]
    }

    fn format_one_arg_attribute(&mut self) {
        self.write_current_token_and_bump(); // #[
        self.skip_comments_and_whitespace();
        self.write_current_token_and_bump(); // name
        self.write_left_paren(); // (
        loop {
            self.skip_comments_and_whitespace();
            if self.is_at(Token::RightParen) {
                self.write_right_paren();
                break;
            } else {
                self.write_current_token_and_bump();
            }
        }
        self.write_right_bracket(); // ]
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_format;

    fn assert_format_attribute(src: &str, expected: &str) {
        let src = format!("  {src} fn foo() {{}}");
        let expected = format!("{expected}\nfn foo() {{}}\n");
        assert_format(&src, &expected);
    }

    #[test]
    fn format_inner_tag_attribute() {
        let src = "  #!['foo] ";
        let expected = "#!['foo]\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_deprecated_attribute() {
        let src = "  #[ deprecated ] ";
        let expected = "#[deprecated]";
        assert_format_attribute(src, expected);
    }

    #[test]
    fn format_deprecated_attribute_with_message() {
        let src = "  #[ deprecated ( \"use something else\" ) ] ";
        let expected = "#[deprecated(\"use something else\")]";
        assert_format_attribute(src, expected);
    }

    #[test]
    fn format_contract_library_method() {
        let src = "  #[ contract_library_method ] ";
        let expected = "#[contract_library_method]";
        assert_format_attribute(src, expected);
    }

    #[test]
    fn format_export() {
        let src = "  #[ export ] ";
        let expected = "#[export]";
        assert_format_attribute(src, expected);
    }

    #[test]
    fn format_varargs() {
        let src = "  #[ varargs ] ";
        let expected = "#[varargs]";
        assert_format_attribute(src, expected);
    }

    #[test]
    fn format_use_callers_scope() {
        let src = "  #[ use_callers_scope ] ";
        let expected = "#[use_callers_scope]";
        assert_format_attribute(src, expected);
    }

    #[test]
    fn format_field_attribute() {
        let src = "  #[ field ( bn256 ) ] ";
        let expected = "#[field(bn256)]";
        assert_format_attribute(src, expected);
    }

    #[test]
    fn format_abi_attribute() {
        let src = "  #[ abi ( foo ) ] ";
        let expected = "#[abi(foo)]";
        assert_format_attribute(src, expected);
    }

    #[test]
    fn format_allow_attribute() {
        let src = "  #[ allow ( unused_vars ) ] ";
        let expected = "#[allow(unused_vars)]";
        assert_format_attribute(src, expected);
    }

    #[test]
    fn format_meta_attribute_without_arguments() {
        let src = "  #[ custom  ] ";
        let expected = "#[custom]";
        assert_format_attribute(src, expected);
    }

    #[test]
    fn format_meta_attribute_without_arguments_removes_parentheses() {
        let src = "  #[ custom (  ) ] ";
        let expected = "#[custom]";
        assert_format_attribute(src, expected);
    }

    #[test]
    fn format_meta_attribute_without_arguments_but_comment() {
        let src = "  #[ custom ( /* nothing */ ) ] ";
        let expected = "#[custom( /* nothing */ )]";
        assert_format_attribute(src, expected);
    }

    #[test]
    fn format_meta_attribute_with_arguments() {
        let src = "  #[ custom ( 1 , 2,  [ 3, 4 ],  ) ] ";
        let expected = "#[custom(1, 2, [3, 4])]";
        assert_format_attribute(src, expected);
    }

    #[test]
    fn format_foreign_attribute() {
        let src = "  #[ foreign ( foo ) ] ";
        let expected = "#[foreign(foo)]";
        assert_format_attribute(src, expected);
    }

    #[test]
    fn format_recursive_attribute() {
        let src = "  #[ recursive ] ";
        let expected = "#[recursive]";
        assert_format_attribute(src, expected);
    }

    #[test]
    fn format_test_attribute() {
        let src = "  #[ test ] ";
        let expected = "#[test]";
        assert_format_attribute(src, expected);
    }

    #[test]
    fn format_test_should_fail_attribute() {
        let src = "  #[ test ( should_fail )] ";
        let expected = "#[test(should_fail)]";
        assert_format_attribute(src, expected);
    }

    #[test]
    fn format_test_should_fail_with_reason_attribute() {
        let src = "  #[ test ( should_fail_with=\"reason\" )] ";
        let expected = "#[test(should_fail_with = \"reason\")]";
        assert_format_attribute(src, expected);
    }

    #[test]
    fn format_multiple_function_attributes() {
        let src = " #[foo] #[test] #[bar]  ";
        let expected = "#[foo]\n#[test]\n#[bar]";
        assert_format_attribute(src, expected);
    }
}
