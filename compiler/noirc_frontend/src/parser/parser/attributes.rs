use noirc_errors::Location;

use crate::ast::{Expression, ExpressionKind, Ident, Literal, Path};
use crate::lexer::errors::LexerErrorKind;
use crate::parser::ParserErrorReason;
use crate::parser::labels::ParsingRuleLabel;
use crate::token::{
    Attribute, FunctionAttribute, FunctionAttributeKind, FuzzingScope, MetaAttribute,
    MetaAttributeName, SecondaryAttribute, SecondaryAttributeKind, TestScope, Token,
};

use super::Parser;
use super::parse_many::without_separator;

impl Parser<'_> {
    /// InnerAttribute = '#![' SecondaryAttribute ']'
    pub(super) fn parse_inner_attribute(&mut self) -> Option<SecondaryAttribute> {
        let start_location = self.current_token_location;
        let is_tag = self.eat_inner_attribute_start()?;
        let attribute = if is_tag {
            self.parse_tag_attribute(start_location)
        } else {
            self.parse_non_tag_attribute(start_location)
        };

        match attribute {
            Attribute::Function(function_attribute) => {
                self.errors.push(
                    LexerErrorKind::InvalidInnerAttribute {
                        location: self.location_since(start_location),
                        found: function_attribute.to_string(),
                    }
                    .into(),
                );
                None
            }
            Attribute::Secondary(secondary_attribute) => Some(secondary_attribute),
        }
    }

    /// Attributes = Attribute*
    pub(super) fn parse_attributes(&mut self) -> Vec<(Attribute, Location)> {
        self.parse_many("attributes", without_separator(), Self::parse_attribute)
    }

    /// Attribute = '#[' (FunctionAttribute | SecondaryAttribute) ']'
    ///
    /// FunctionAttribute
    ///     = 'builtin' '(' AttributeValue ')'
    ///     | 'fold'
    ///     | 'foreign' '(' AttributeValue ')'
    ///     | 'inline_always'
    ///     | 'inline_never'
    ///     | 'no_predicates'
    ///     | 'oracle' '(' AttributeValue ')'
    ///     | 'recursive'
    ///     | 'test'
    ///     | 'test' '(' 'should_fail' ')'
    ///     | 'test' '(' 'should_fail_with' '=' string ')'
    ///     | 'fuzz'
    ///     | 'fuzz' '(' 'only_fail_with' '=' string ')'
    ///     | 'fuzz' '(' 'should_fail' ')'
    ///     | 'fuzz' '(' 'should_fail_with' '=' string ')'
    ///
    /// SecondaryAttribute
    ///     = 'abi' '(' AttributeValue ')'
    ///     | 'allow' '(' AttributeValue ')'
    ///     | 'deprecated'
    ///     | 'deprecated' '(' string ')'
    ///     | 'contract_library_method'
    ///     | 'export'
    ///     | 'field' '(' AttributeValue ')'
    ///     | 'use_callers_scope'
    ///     | 'varargs'
    ///     | MetaAttribute
    ///
    /// MetaAttribute
    ///     = Path Arguments?
    ///
    /// AttributeValue
    ///     = Path
    ///     | integer
    pub(crate) fn parse_attribute(&mut self) -> Option<(Attribute, Location)> {
        let start_location = self.current_token_location;
        let is_tag = self.eat_attribute_start()?;
        let attribute = if is_tag {
            self.parse_tag_attribute(start_location)
        } else {
            self.parse_non_tag_attribute(start_location)
        };
        Some((attribute, self.location_since(start_location)))
    }

    pub(super) fn validate_secondary_attributes(
        &mut self,
        attributes: Vec<(Attribute, Location)>,
    ) -> Vec<SecondaryAttribute> {
        attributes
            .into_iter()
            .filter_map(|(attribute, location)| match attribute {
                Attribute::Function(..) => {
                    self.push_error(ParserErrorReason::NoFunctionAttributesAllowedOnType, location);
                    None
                }
                Attribute::Secondary(attr) => Some(attr),
            })
            .collect()
    }

    fn parse_tag_attribute(&mut self, start_location: Location) -> Attribute {
        let mut contents = String::new();

        let mut brackets_count = 1; // 1 because of the starting `#[`
        // Note: Keep trailing whitespace tokens.
        // If we skip them, only non-whitespace tokens are parsed.
        // When converting those tokens into a `String` for the tag attribute,
        // the result will lose whitespace and no longer match the original content.
        self.set_lexer_skip_whitespaces_flag(false);

        while !self.at_eof() {
            if self.at(Token::LeftBracket) {
                brackets_count += 1;
            } else if self.at(Token::RightBracket) {
                brackets_count -= 1;
                if brackets_count == 0 {
                    self.bump();
                    break;
                }
            }

            contents.push_str(&self.token.to_string());
            self.bump();
        }

        self.set_lexer_skip_whitespaces_flag(true);
        while self.at_whitespace() {
            self.bump();
        }

        let location = self.location_since(start_location);
        let kind = SecondaryAttributeKind::Tag(contents);
        let attr = SecondaryAttribute { kind, location };
        Attribute::Secondary(attr)
    }

    fn parse_non_tag_attribute(&mut self, start_location: Location) -> Attribute {
        if let Some(path) = self.parse_path_no_turbofish() {
            if let Some(ident) = path.as_ident() {
                if ident.as_str() == "test" {
                    // The test attribute is the only secondary attribute that has `a = b` in its syntax
                    // (`should_fail_with = "..."``) so we parse it differently.
                    self.parse_test_attribute(start_location)
                } else if ident.as_str() == "fuzz" {
                    // The fuzz attribute is a secondary attribute that has `a = b` in its syntax
                    // (`only_fail_with = "..."``) or (`should_fail_with = "..."``) so we parse it differently.
                    self.parse_fuzz_attribute(start_location)
                } else if ident.as_str() == "must_use" {
                    // The muse_use attribute is a secondary attribute that has the syntax `must_use = string` in its syntax (to match rust)
                    self.parse_must_use_attribute(start_location)
                } else {
                    // Every other attribute has the form `name(arg1, arg2, .., argN)`
                    self.parse_ident_attribute_other_than_test_and_fuzz(ident, start_location)
                }
            } else {
                // This is a Meta attribute with the syntax `path(arg1, arg2, .., argN)`
                let name = MetaAttributeName::Path(path);
                self.parse_meta_attribute(name, start_location)
            }
        } else if let Some(expr_id) = self.eat_unquote_marker() {
            // This is a Meta attribute with the syntax `$expr(arg1, arg2, .., argN)`
            let name = MetaAttributeName::Resolved(expr_id);
            self.parse_meta_attribute(name, start_location)
        } else {
            self.expected_label(ParsingRuleLabel::Path);
            self.parse_tag_attribute(start_location)
        }
    }

    fn parse_meta_attribute(
        &mut self,
        name: MetaAttributeName,
        start_location: Location,
    ) -> Attribute {
        let arguments = self.parse_arguments().unwrap_or_default();
        self.skip_until_right_bracket(true);
        let location = self.location_since(start_location);
        let kind = SecondaryAttributeKind::Meta(MetaAttribute { name, arguments });
        let attr = SecondaryAttribute { kind, location };
        Attribute::Secondary(attr)
    }

    fn parse_ident_attribute_other_than_test_and_fuzz(
        &mut self,
        ident: &Ident,
        start_location: Location,
    ) -> Attribute {
        let arguments = self.parse_arguments().unwrap_or_default();
        self.skip_until_right_bracket(true);
        let location = self.location_since(start_location);
        match ident.as_str() {
            "abi" => self.parse_single_name_attribute(ident, arguments, start_location, |name| {
                let kind = SecondaryAttributeKind::Abi(name);
                let attr = SecondaryAttribute { kind, location };
                Attribute::Secondary(attr)
            }),
            "allow" => self.parse_single_name_attribute(ident, arguments, start_location, |name| {
                let kind = SecondaryAttributeKind::Allow(name);
                let attr = SecondaryAttribute { kind, location };
                Attribute::Secondary(attr)
            }),
            "builtin" => {
                self.parse_single_name_attribute(ident, arguments, start_location, |name| {
                    let kind = FunctionAttributeKind::Builtin(name);
                    let attr = FunctionAttribute { kind, location };
                    Attribute::Function(attr)
                })
            }
            "deprecated" => {
                let kind = self.parse_deprecated_attribute(ident, arguments);
                let attr = SecondaryAttribute { kind, location };
                Attribute::Secondary(attr)
            }
            "contract_library_method" => {
                let kind = SecondaryAttributeKind::ContractLibraryMethod;
                let attr = SecondaryAttribute { kind, location };
                let attr = Attribute::Secondary(attr);
                self.parse_no_args_attribute(ident, arguments, attr)
            }
            "export" => {
                let kind = SecondaryAttributeKind::Export;
                let attr = SecondaryAttribute { kind, location };
                let attr = Attribute::Secondary(attr);
                self.parse_no_args_attribute(ident, arguments, attr)
            }
            "field" => self.parse_single_name_attribute(ident, arguments, start_location, |name| {
                let kind = SecondaryAttributeKind::Field(name);
                let attr = SecondaryAttribute { kind, location };
                Attribute::Secondary(attr)
            }),
            "fold" => {
                let kind = FunctionAttributeKind::Fold;
                let attr = FunctionAttribute { kind, location };
                let attr = Attribute::Function(attr);
                self.parse_no_args_attribute(ident, arguments, attr)
            }
            "foreign" => {
                self.parse_single_name_attribute(ident, arguments, start_location, |name| {
                    let kind = FunctionAttributeKind::Foreign(name);
                    let attr = FunctionAttribute { kind, location };
                    Attribute::Function(attr)
                })
            }
            "inline_always" => {
                let kind = FunctionAttributeKind::InlineAlways;
                let attr = FunctionAttribute { kind, location };
                let attr = Attribute::Function(attr);
                self.parse_no_args_attribute(ident, arguments, attr)
            }
            "inline_never" => {
                let kind = FunctionAttributeKind::InlineNever;
                let attr = FunctionAttribute { kind, location };
                let attr = Attribute::Function(attr);
                self.parse_no_args_attribute(ident, arguments, attr)
            }
            "no_predicates" => {
                let kind = FunctionAttributeKind::NoPredicates;
                let attr = FunctionAttribute { kind, location };
                let attr = Attribute::Function(attr);
                self.parse_no_args_attribute(ident, arguments, attr)
            }
            "oracle" => {
                self.parse_single_name_attribute(ident, arguments, start_location, |name| {
                    let kind = FunctionAttributeKind::Oracle(name);
                    let attr = FunctionAttribute { kind, location };
                    Attribute::Function(attr)
                })
            }
            "use_callers_scope" => {
                let kind = SecondaryAttributeKind::UseCallersScope;
                let attr = SecondaryAttribute { kind, location };
                let attr = Attribute::Secondary(attr);
                self.parse_no_args_attribute(ident, arguments, attr)
            }
            "varargs" => {
                let kind = SecondaryAttributeKind::Varargs;
                let attr = SecondaryAttribute { kind, location };
                let attr = Attribute::Secondary(attr);
                self.parse_no_args_attribute(ident, arguments, attr)
            }
            _ => {
                let kind = SecondaryAttributeKind::Meta(MetaAttribute {
                    name: MetaAttributeName::Path(Path::from_ident(ident.clone())),
                    arguments,
                });
                let attr = SecondaryAttribute { kind, location };
                Attribute::Secondary(attr)
            }
        }
    }

    fn parse_deprecated_attribute(
        &mut self,
        ident: &Ident,
        mut arguments: Vec<Expression>,
    ) -> SecondaryAttributeKind {
        if arguments.is_empty() {
            return SecondaryAttributeKind::Deprecated(None);
        }

        if arguments.len() > 1 {
            self.push_error(
                ParserErrorReason::WrongNumberOfAttributeArguments {
                    name: ident.to_string(),
                    min: 0,
                    max: 1,
                    found: arguments.len(),
                },
                ident.location(),
            );
            return SecondaryAttributeKind::Deprecated(None);
        }

        let argument = arguments.remove(0);
        let ExpressionKind::Literal(Literal::Str(message)) = argument.kind else {
            self.push_error(
                ParserErrorReason::DeprecatedAttributeExpectsAStringArgument,
                argument.location,
            );
            return SecondaryAttributeKind::Deprecated(None);
        };

        SecondaryAttributeKind::Deprecated(Some(message))
    }

    fn parse_test_attribute(&mut self, start_location: Location) -> Attribute {
        let scope = if self.eat_left_paren() {
            let scope = if let Some(ident) = self.eat_ident() {
                match ident.as_str() {
                    "should_fail" => Some(TestScope::ShouldFailWith { reason: None }),
                    "should_fail_with" => {
                        self.eat_or_error(Token::Assign);
                        if let Some(reason) = self.eat_str() {
                            Some(TestScope::ShouldFailWith { reason: Some(reason) })
                        } else {
                            Some(TestScope::ShouldFailWith { reason: None })
                        }
                    }
                    "only_fail_with" => {
                        self.eat_or_error(Token::Assign);
                        if let Some(reason) = self.eat_str() {
                            Some(TestScope::OnlyFailWith { reason })
                        } else {
                            self.expected_string();
                            None
                        }
                    }
                    _ => None,
                }
            } else {
                None
            };
            self.eat_or_error(Token::RightParen);
            scope
        } else {
            Some(TestScope::None)
        };

        self.skip_until_right_bracket(true);

        let scope = if let Some(scope) = scope {
            scope
        } else {
            self.errors.push(
                LexerErrorKind::MalformedTestAttribute {
                    location: self.location_since(start_location),
                }
                .into(),
            );
            TestScope::None
        };

        let location = self.location_since(start_location);
        let kind = FunctionAttributeKind::Test(scope);
        let attr = FunctionAttribute { kind, location };
        Attribute::Function(attr)
    }

    fn parse_fuzz_attribute(&mut self, start_location: Location) -> Attribute {
        let scope = if self.eat_left_paren() {
            let scope = if let Some(ident) = self.eat_ident() {
                match ident.as_str() {
                    "should_fail" => Some(FuzzingScope::ShouldFailWith { reason: None }),
                    "should_fail_with" => {
                        self.eat_or_error(Token::Assign);
                        if let Some(reason) = self.eat_str() {
                            Some(FuzzingScope::ShouldFailWith { reason: Some(reason) })
                        } else {
                            Some(FuzzingScope::ShouldFailWith { reason: None })
                        }
                    }
                    "only_fail_with" => {
                        self.eat_or_error(Token::Assign);
                        self.eat_str().map(|reason| FuzzingScope::OnlyFailWith { reason })
                    }
                    _ => None,
                }
            } else {
                None
            };
            self.eat_or_error(Token::RightParen);
            scope
        } else {
            Some(FuzzingScope::None)
        };

        self.skip_until_right_bracket(true);

        let scope = if let Some(scope) = scope {
            scope
        } else {
            self.errors.push(
                LexerErrorKind::MalformedFuzzAttribute {
                    location: self.location_since(start_location),
                }
                .into(),
            );
            FuzzingScope::None
        };

        let location = self.location_since(start_location);
        let kind = FunctionAttributeKind::FuzzingHarness(scope);
        let attr = FunctionAttribute { kind, location };
        Attribute::Function(attr)
    }

    fn parse_must_use_attribute(&mut self, start_location: Location) -> Attribute {
        let location_after_name = self.current_token_location;

        let message = if self.eat_assign() {
            let message = self.eat_str();
            if message.is_none() {
                let location = self.location_since(start_location);
                let error = LexerErrorKind::MalformedMustUseAttribute { location };
                self.errors.push(error.into());
            }
            self.skip_until_right_bracket(false);
            message
        } else {
            if self.at(Token::RightBracket) {
                self.skip_until_right_bracket(false);
            } else {
                let location = self.location_since(location_after_name);
                let error = LexerErrorKind::MalformedMustUseAttribute { location };
                self.errors.push(error.into());
            }
            None
        };

        let location = self.location_since(start_location);
        let kind = SecondaryAttributeKind::MustUse(message);
        Attribute::Secondary(SecondaryAttribute { kind, location })
    }

    fn parse_single_name_attribute<F>(
        &mut self,
        ident: &Ident,
        mut arguments: Vec<Expression>,
        start_location: Location,
        f: F,
    ) -> Attribute
    where
        F: FnOnce(String) -> Attribute,
    {
        if arguments.len() != 1 {
            self.push_error(
                ParserErrorReason::WrongNumberOfAttributeArguments {
                    name: ident.to_string(),
                    min: 1,
                    max: 1,
                    found: arguments.len(),
                },
                self.current_token_location,
            );
            return f(String::new());
        }

        let argument = arguments.remove(0);
        match argument.kind {
            ExpressionKind::Variable(..) | ExpressionKind::Literal(Literal::Integer(..)) => {
                f(argument.to_string())
            }
            _ => {
                let location = self.location_since(start_location);
                self.errors.push(
                    LexerErrorKind::MalformedFuncAttribute {
                        location,
                        found: argument.to_string(),
                    }
                    .into(),
                );
                f(String::new())
            }
        }
    }

    fn parse_no_args_attribute(
        &mut self,
        ident: &Ident,
        arguments: Vec<Expression>,
        attribute: Attribute,
    ) -> Attribute {
        if !arguments.is_empty() {
            self.push_error(
                ParserErrorReason::WrongNumberOfAttributeArguments {
                    name: ident.to_string(),
                    min: 0,
                    max: 0,
                    found: arguments.len(),
                },
                ident.location(),
            );
        }

        attribute
    }

    fn skip_until_right_bracket(&mut self, mut issue_error: bool) {
        let mut brackets_count = 1; // 1 because of the starting `#[`

        while !self.at_eof() {
            if self.at(Token::LeftBracket) {
                brackets_count += 1;
            } else if self.at(Token::RightBracket) {
                brackets_count -= 1;
                if brackets_count == 0 {
                    self.bump();
                    break;
                }
            }

            if issue_error {
                issue_error = false;
                self.expected_token(Token::RightBracket);
            }
            self.bump();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        parser::{Parser, parser::tests::expect_no_errors},
        token::{Attribute, FunctionAttributeKind, SecondaryAttributeKind, TestScope},
    };

    fn parse_inner_secondary_attribute_no_errors(src: &str, expected: SecondaryAttributeKind) {
        let mut parser = Parser::for_str_with_dummy_file(src);
        let attribute = parser.parse_inner_attribute();
        expect_no_errors(&parser.errors);
        assert_eq!(attribute.unwrap().kind, expected);
    }

    fn parse_function_attribute_no_errors(src: &str, expected: FunctionAttributeKind) {
        let mut parser = Parser::for_str_with_dummy_file(src);
        let (attribute, _span) = parser.parse_attribute().unwrap();
        expect_no_errors(&parser.errors);
        let Attribute::Function(attribute) = attribute else {
            panic!("Expected function attribute");
        };
        assert_eq!(attribute.kind, expected);
    }

    fn parse_secondary_attribute_no_errors(src: &str, expected: SecondaryAttributeKind) {
        let mut parser = Parser::for_str_with_dummy_file(src);
        let (attribute, _span) = parser.parse_attribute().unwrap();
        expect_no_errors(&parser.errors);
        let Attribute::Secondary(attribute) = attribute else {
            panic!("Expected secondary attribute");
        };
        assert_eq!(attribute.kind, expected);
    }

    #[test]
    fn parses_inner_attribute_as_tag() {
        let src = "#!['hello]";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let SecondaryAttributeKind::Tag(contents) = parser.parse_inner_attribute().unwrap().kind
        else {
            panic!("Expected inner tag attribute");
        };
        expect_no_errors(&parser.errors);
        assert_eq!(contents, "hello");
    }

    #[test]
    fn parses_inner_attribute_as_tag_with_nested_brackets() {
        let src = "#!['hello[1]]";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let SecondaryAttributeKind::Tag(contents) = parser.parse_inner_attribute().unwrap().kind
        else {
            panic!("Expected inner tag attribute");
        };
        expect_no_errors(&parser.errors);
        assert_eq!(contents, "hello[1]");
    }

    #[test]
    fn parses_inner_attribute_deprecated() {
        let src = "#![deprecated]";
        let expected = SecondaryAttributeKind::Deprecated(None);
        parse_inner_secondary_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_inner_attribute_deprecated_with_message() {
        let src = "#![deprecated(\"use something else\")]";
        let expected = SecondaryAttributeKind::Deprecated(Some("use something else".to_string()));
        parse_inner_secondary_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_inner_attribute_contract_library_method() {
        let src = "#![contract_library_method]";
        let expected = SecondaryAttributeKind::ContractLibraryMethod;
        parse_inner_secondary_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_inner_attribute_export() {
        let src = "#![export]";
        let expected = SecondaryAttributeKind::Export;
        parse_inner_secondary_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_inner_attribute_varargs() {
        let src = "#![varargs]";
        let expected = SecondaryAttributeKind::Varargs;
        parse_inner_secondary_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_inner_attribute_use_callers_scope() {
        let src = "#![use_callers_scope]";
        let expected = SecondaryAttributeKind::UseCallersScope;
        parse_inner_secondary_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_abi() {
        let src = "#[abi(foo)]";
        let expected = SecondaryAttributeKind::Abi("foo".to_string());
        parse_secondary_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_foreign() {
        let src = "#[foreign(foo)]";
        let expected = FunctionAttributeKind::Foreign("foo".to_string());
        parse_function_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_builtin() {
        let src = "#[builtin(foo)]";
        let expected = FunctionAttributeKind::Builtin("foo".to_string());
        parse_function_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_oracle() {
        let src = "#[oracle(foo)]";
        let expected = FunctionAttributeKind::Oracle("foo".to_string());
        parse_function_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_fold() {
        let src = "#[fold]";
        let expected = FunctionAttributeKind::Fold;
        parse_function_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_no_predicates() {
        let src = "#[no_predicates]";
        let expected = FunctionAttributeKind::NoPredicates;
        parse_function_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_inline_always() {
        let src = "#[inline_always]";
        let expected = FunctionAttributeKind::InlineAlways;
        parse_function_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_inline_never() {
        let src = "#[inline_never]";
        let expected = FunctionAttributeKind::InlineNever;
        parse_function_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_field() {
        let src = "#[field(bn254)]";
        let expected = SecondaryAttributeKind::Field("bn254".to_string());
        parse_secondary_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_field_with_integer() {
        let src = "#[field(23)]";
        let expected = SecondaryAttributeKind::Field("23".to_string());
        parse_secondary_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_allow() {
        let src = "#[allow(unused_vars)]";
        let expected = SecondaryAttributeKind::Allow("unused_vars".to_string());
        parse_secondary_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_test_no_scope() {
        let src = "#[test]";
        let expected = FunctionAttributeKind::Test(TestScope::None);
        parse_function_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_test_should_fail() {
        let src = "#[test(should_fail)]";
        let expected = FunctionAttributeKind::Test(TestScope::ShouldFailWith { reason: None });
        parse_function_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_test_should_fail_with() {
        let src = "#[test(should_fail_with = \"reason\")]";
        let reason = Some("reason".to_string());
        let expected = FunctionAttributeKind::Test(TestScope::ShouldFailWith { reason });
        parse_function_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_test_only_fail_with() {
        let src = "#[test(only_fail_with = \"reason\")]";
        let reason = "reason".to_string();
        let expected = FunctionAttributeKind::Test(TestScope::OnlyFailWith { reason });
        parse_function_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_meta_attribute_single_identifier_no_arguments() {
        let src = "#[foo]";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let (attribute, _span) = parser.parse_attribute().unwrap();
        expect_no_errors(&parser.errors);
        let Attribute::Secondary(attribute) = attribute else {
            panic!("Expected secondary attribute");
        };
        let SecondaryAttributeKind::Meta(meta) = attribute.kind else {
            panic!("Expected meta attribute");
        };
        assert_eq!(meta.name.to_string(), "foo");
        assert!(meta.arguments.is_empty());
    }

    #[test]
    fn parses_meta_attribute_single_identifier_with_arguments() {
        let src = "#[foo(1, 2, 3)]";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let (attribute, _span) = parser.parse_attribute().unwrap();
        expect_no_errors(&parser.errors);
        let Attribute::Secondary(attribute) = attribute else {
            panic!("Expected secondary attribute");
        };
        let SecondaryAttributeKind::Meta(meta) = attribute.kind else {
            panic!("Expected meta attribute");
        };
        assert_eq!(meta.name.to_string(), "foo");
        assert_eq!(meta.arguments.len(), 3);
        assert_eq!(meta.arguments[0].to_string(), "1");
    }

    #[test]
    fn parses_meta_attribute_path_with_arguments() {
        let src = "#[foo::bar(1, 2, 3)]";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let (attribute, _span) = parser.parse_attribute().unwrap();
        expect_no_errors(&parser.errors);
        let Attribute::Secondary(attribute) = attribute else {
            panic!("Expected secondary attribute");
        };
        let SecondaryAttributeKind::Meta(meta) = attribute.kind else {
            panic!("Expected meta attribute");
        };
        assert_eq!(meta.name.to_string(), "foo::bar");
        assert_eq!(meta.arguments.len(), 3);
        assert_eq!(meta.arguments[0].to_string(), "1");
    }

    #[test]
    fn parses_attributes() {
        let src = "#[test] #[deprecated]";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let mut attributes = parser.parse_attributes();
        expect_no_errors(&parser.errors);
        assert_eq!(attributes.len(), 2);

        let (attr, _) = attributes.remove(0);
        let Attribute::Function(attr) = attr else {
            panic!("Expected function attribute");
        };
        assert!(matches!(attr.kind, FunctionAttributeKind::Test(TestScope::None)));

        let (attr, _) = attributes.remove(0);
        let Attribute::Secondary(attr) = attr else {
            panic!("Expected secondary attribute");
        };
        assert!(matches!(attr.kind, SecondaryAttributeKind::Deprecated(None)));
    }

    #[test]
    fn parses_inner_tag_attribute_with_whitespace() {
        let src = "#!['hello world]";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let SecondaryAttributeKind::Tag(contents) = parser.parse_inner_attribute().unwrap().kind
        else {
            panic!("Expected inner tag attribute");
        };
        expect_no_errors(&parser.errors);
        assert_eq!(contents, "hello world");
    }

    #[test]
    fn parses_inner_tag_attribute_with_multiple_whitespaces() {
        let src = "#!['x as u32]";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let SecondaryAttributeKind::Tag(contents) = parser.parse_inner_attribute().unwrap().kind
        else {
            panic!("Expected inner tag attribute");
        };
        expect_no_errors(&parser.errors);
        assert_eq!(contents, "x as u32");
    }
    #[test]
    fn parses_tag_attribute_with_multiple_whitespaces() {
        let src = "#['y as i16]";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let (attribute, _span) = parser.parse_attribute().unwrap();
        expect_no_errors(&parser.errors);
        let Attribute::Secondary(attribute) = attribute else {
            panic!("Expected secondary attribute");
        };
        let SecondaryAttributeKind::Tag(contents) = attribute.kind else {
            panic!("Expected meta attribute");
        };
        assert_eq!(contents, "y as i16");
    }
    #[test]
    fn parses_tag_attribute_with_whitespace() {
        let src = "#['foo bar]";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let (attribute, _span) = parser.parse_attribute().unwrap();
        expect_no_errors(&parser.errors);
        let Attribute::Secondary(attribute) = attribute else {
            panic!("Expected secondary attribute");
        };
        let SecondaryAttributeKind::Tag(contents) = attribute.kind else {
            panic!("Expected meta attribute");
        };
        assert_eq!(contents, "foo bar");
    }
}
