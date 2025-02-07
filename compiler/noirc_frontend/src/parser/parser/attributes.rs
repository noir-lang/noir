use noirc_errors::Location;

use crate::ast::{Expression, ExpressionKind, Ident, Literal, Path};
use crate::lexer::errors::LexerErrorKind;
use crate::parser::labels::ParsingRuleLabel;
use crate::parser::ParserErrorReason;
use crate::token::{Attribute, FunctionAttribute, MetaAttribute, TestScope, Token};
use crate::token::{CustomAttribute, SecondaryAttribute};

use super::parse_many::without_separator;
use super::Parser;

impl<'a> Parser<'a> {
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
                        span: self.location_since(start_location).span,
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
    ///     | 'no_predicates'
    ///     | 'oracle' '(' AttributeValue ')'
    ///     | 'recursive'
    ///     | 'test'
    ///     | 'test' '(' 'should_fail' ')'
    ///     | 'test' '(' 'should_fail_with' '=' string ')'
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
        let contents_start_location = self.current_token_location;
        let mut contents_location = contents_start_location;
        let mut contents = String::new();

        let mut brackets_count = 1; // 1 because of the starting `#[`

        while !self.at_eof() {
            if self.at(Token::LeftBracket) {
                brackets_count += 1;
            } else if self.at(Token::RightBracket) {
                brackets_count -= 1;
                if brackets_count == 0 {
                    contents_location = self.location_since(contents_start_location);
                    self.bump();
                    break;
                }
            }

            contents.push_str(&self.token.to_string());
            self.bump();
        }

        Attribute::Secondary(SecondaryAttribute::Tag(CustomAttribute {
            contents,
            span: self.location_since(start_location).span,
            contents_span: contents_location.span,
        }))
    }

    fn parse_non_tag_attribute(&mut self, start_location: Location) -> Attribute {
        if matches!(&self.token.token(), Token::Keyword(..))
            && (self.next_is(Token::LeftParen) || self.next_is(Token::RightBracket))
        {
            // This is a Meta attribute with the syntax `keyword(arg1, arg2, .., argN)`
            let path = Path::from_single(self.token.to_string(), self.current_token_location);
            self.bump();
            self.parse_meta_attribute(path, start_location)
        } else if let Some(path) = self.parse_path_no_turbofish() {
            if let Some(ident) = path.as_ident() {
                if ident.0.contents == "test" {
                    // The test attribute is the only secondary attribute that has `a = b` in its syntax
                    // (`should_fail_with = "..."``) so we parse it differently.
                    self.parse_test_attribute(start_location)
                } else {
                    // Every other attribute has the form `name(arg1, arg2, .., argN)`
                    self.parse_ident_attribute_other_than_test(ident, start_location)
                }
            } else {
                // This is a Meta attribute with the syntax `path(arg1, arg2, .., argN)`
                self.parse_meta_attribute(path, start_location)
            }
        } else {
            self.expected_label(ParsingRuleLabel::Path);
            self.parse_tag_attribute(start_location)
        }
    }

    fn parse_meta_attribute(&mut self, name: Path, start_location: Location) -> Attribute {
        let arguments = self.parse_arguments().unwrap_or_default();
        self.skip_until_right_bracket();
        Attribute::Secondary(SecondaryAttribute::Meta(MetaAttribute {
            name,
            arguments,
            span: self.location_since(start_location).span,
        }))
    }

    fn parse_ident_attribute_other_than_test(
        &mut self,
        ident: &Ident,
        start_location: Location,
    ) -> Attribute {
        let arguments = self.parse_arguments().unwrap_or_default();
        self.skip_until_right_bracket();
        match ident.0.contents.as_str() {
            "abi" => self.parse_single_name_attribute(ident, arguments, start_location, |name| {
                Attribute::Secondary(SecondaryAttribute::Abi(name))
            }),
            "allow" => self.parse_single_name_attribute(ident, arguments, start_location, |name| {
                Attribute::Secondary(SecondaryAttribute::Allow(name))
            }),
            "builtin" => {
                self.parse_single_name_attribute(ident, arguments, start_location, |name| {
                    Attribute::Function(FunctionAttribute::Builtin(name))
                })
            }
            "deprecated" => self.parse_deprecated_attribute(ident, arguments),
            "contract_library_method" => {
                let attr = Attribute::Secondary(SecondaryAttribute::ContractLibraryMethod);
                self.parse_no_args_attribute(ident, arguments, attr)
            }
            "export" => {
                let attr = Attribute::Secondary(SecondaryAttribute::Export);
                self.parse_no_args_attribute(ident, arguments, attr)
            }
            "field" => self.parse_single_name_attribute(ident, arguments, start_location, |name| {
                Attribute::Secondary(SecondaryAttribute::Field(name))
            }),
            "fold" => {
                let attr = Attribute::Function(FunctionAttribute::Fold);
                self.parse_no_args_attribute(ident, arguments, attr)
            }
            "foreign" => {
                self.parse_single_name_attribute(ident, arguments, start_location, |name| {
                    Attribute::Function(FunctionAttribute::Foreign(name))
                })
            }
            "inline_always" => {
                let attr = Attribute::Function(FunctionAttribute::InlineAlways);
                self.parse_no_args_attribute(ident, arguments, attr)
            }
            "no_predicates" => {
                let attr = Attribute::Function(FunctionAttribute::NoPredicates);
                self.parse_no_args_attribute(ident, arguments, attr)
            }
            "oracle" => {
                self.parse_single_name_attribute(ident, arguments, start_location, |name| {
                    Attribute::Function(FunctionAttribute::Oracle(name))
                })
            }
            "use_callers_scope" => {
                let attr = Attribute::Secondary(SecondaryAttribute::UseCallersScope);
                self.parse_no_args_attribute(ident, arguments, attr)
            }
            "varargs" => {
                let attr = Attribute::Secondary(SecondaryAttribute::Varargs);
                self.parse_no_args_attribute(ident, arguments, attr)
            }
            _ => Attribute::Secondary(SecondaryAttribute::Meta(MetaAttribute {
                name: Path::from_ident(ident.clone()),
                arguments,
                span: self.location_since(start_location).span,
            })),
        }
    }

    fn parse_deprecated_attribute(
        &mut self,
        ident: &Ident,
        mut arguments: Vec<Expression>,
    ) -> Attribute {
        if arguments.is_empty() {
            return Attribute::Secondary(SecondaryAttribute::Deprecated(None));
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
            return Attribute::Secondary(SecondaryAttribute::Deprecated(None));
        }

        let argument = arguments.remove(0);
        let ExpressionKind::Literal(Literal::Str(message)) = argument.kind else {
            self.push_error(
                ParserErrorReason::DeprecatedAttributeExpectsAStringArgument,
                argument.location,
            );
            return Attribute::Secondary(SecondaryAttribute::Deprecated(None));
        };

        Attribute::Secondary(SecondaryAttribute::Deprecated(Some(message)))
    }

    fn parse_test_attribute(&mut self, start_location: Location) -> Attribute {
        let scope = if self.eat_left_paren() {
            let scope = if let Some(ident) = self.eat_ident() {
                match ident.0.contents.as_str() {
                    "should_fail" => Some(TestScope::ShouldFailWith { reason: None }),
                    "should_fail_with" => {
                        self.eat_or_error(Token::Assign);
                        if let Some(reason) = self.eat_str() {
                            Some(TestScope::ShouldFailWith { reason: Some(reason) })
                        } else {
                            Some(TestScope::ShouldFailWith { reason: None })
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

        self.skip_until_right_bracket();

        let scope = if let Some(scope) = scope {
            scope
        } else {
            self.errors.push(
                LexerErrorKind::MalformedTestAttribute {
                    span: self.location_since(start_location).span,
                }
                .into(),
            );
            TestScope::None
        };

        Attribute::Function(FunctionAttribute::Test(scope))
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
                let span = self.location_since(start_location).span;
                self.errors.push(
                    LexerErrorKind::MalformedFuncAttribute { span, found: argument.to_string() }
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

    fn skip_until_right_bracket(&mut self) {
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

            self.expected_token(Token::RightBracket);
            self.bump();
        }
    }
}

#[cfg(test)]
mod tests {
    use noirc_errors::Span;

    use crate::{
        parser::{parser::tests::expect_no_errors, Parser},
        token::{Attribute, FunctionAttribute, SecondaryAttribute, TestScope},
    };

    fn parse_inner_secondary_attribute_no_errors(src: &str, expected: SecondaryAttribute) {
        let mut parser = Parser::for_str_with_dummy_file(src);
        let attribute = parser.parse_inner_attribute();
        expect_no_errors(&parser.errors);
        assert_eq!(attribute.unwrap(), expected);
    }

    fn parse_attribute_no_errors(src: &str, expected: Attribute) {
        let mut parser = Parser::for_str_with_dummy_file(src);
        let (attribute, _span) = parser.parse_attribute().unwrap();
        expect_no_errors(&parser.errors);
        assert_eq!(attribute, expected);
    }

    #[test]
    fn parses_inner_attribute_as_tag() {
        let src = "#!['hello]";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let Some(SecondaryAttribute::Tag(custom)) = parser.parse_inner_attribute() else {
            panic!("Expected inner tag attribute");
        };
        expect_no_errors(&parser.errors);
        assert_eq!(custom.contents, "hello");
        assert_eq!(custom.span, Span::from(0..src.len() as u32));
        assert_eq!(custom.contents_span, Span::from(4..src.len() as u32 - 1));
    }

    #[test]
    fn parses_inner_attribute_as_tag_with_nested_brackets() {
        let src = "#!['hello[1]]";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let Some(SecondaryAttribute::Tag(custom)) = parser.parse_inner_attribute() else {
            panic!("Expected inner tag attribute");
        };
        expect_no_errors(&parser.errors);
        assert_eq!(custom.contents, "hello[1]");
    }

    #[test]
    fn parses_inner_attribute_deprecated() {
        let src = "#![deprecated]";
        let expected = SecondaryAttribute::Deprecated(None);
        parse_inner_secondary_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_inner_attribute_deprecated_with_message() {
        let src = "#![deprecated(\"use something else\")]";
        let expected = SecondaryAttribute::Deprecated(Some("use something else".to_string()));
        parse_inner_secondary_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_inner_attribute_contract_library_method() {
        let src = "#![contract_library_method]";
        let expected = SecondaryAttribute::ContractLibraryMethod;
        parse_inner_secondary_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_inner_attribute_export() {
        let src = "#![export]";
        let expected = SecondaryAttribute::Export;
        parse_inner_secondary_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_inner_attribute_varargs() {
        let src = "#![varargs]";
        let expected = SecondaryAttribute::Varargs;
        parse_inner_secondary_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_inner_attribute_use_callers_scope() {
        let src = "#![use_callers_scope]";
        let expected = SecondaryAttribute::UseCallersScope;
        parse_inner_secondary_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_abi() {
        let src = "#[abi(foo)]";
        let expected = Attribute::Secondary(SecondaryAttribute::Abi("foo".to_string()));
        parse_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_foreign() {
        let src = "#[foreign(foo)]";
        let expected = Attribute::Function(FunctionAttribute::Foreign("foo".to_string()));
        parse_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_builtin() {
        let src = "#[builtin(foo)]";
        let expected = Attribute::Function(FunctionAttribute::Builtin("foo".to_string()));
        parse_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_oracle() {
        let src = "#[oracle(foo)]";
        let expected = Attribute::Function(FunctionAttribute::Oracle("foo".to_string()));
        parse_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_fold() {
        let src = "#[fold]";
        let expected = Attribute::Function(FunctionAttribute::Fold);
        parse_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_no_predicates() {
        let src = "#[no_predicates]";
        let expected = Attribute::Function(FunctionAttribute::NoPredicates);
        parse_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_inline_always() {
        let src = "#[inline_always]";
        let expected = Attribute::Function(FunctionAttribute::InlineAlways);
        parse_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_field() {
        let src = "#[field(bn254)]";
        let expected = Attribute::Secondary(SecondaryAttribute::Field("bn254".to_string()));
        parse_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_field_with_integer() {
        let src = "#[field(23)]";
        let expected = Attribute::Secondary(SecondaryAttribute::Field("23".to_string()));
        parse_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_allow() {
        let src = "#[allow(unused_vars)]";
        let expected = Attribute::Secondary(SecondaryAttribute::Allow("unused_vars".to_string()));
        parse_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_test_no_scope() {
        let src = "#[test]";
        let expected = Attribute::Function(FunctionAttribute::Test(TestScope::None));
        parse_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_test_should_fail() {
        let src = "#[test(should_fail)]";
        let expected = Attribute::Function(FunctionAttribute::Test(TestScope::ShouldFailWith {
            reason: None,
        }));
        parse_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_attribute_test_should_fail_with() {
        let src = "#[test(should_fail_with = \"reason\")]";
        let expected = Attribute::Function(FunctionAttribute::Test(TestScope::ShouldFailWith {
            reason: Some("reason".to_string()),
        }));
        parse_attribute_no_errors(src, expected);
    }

    #[test]
    fn parses_meta_attribute_single_identifier_no_arguments() {
        let src = "#[foo]";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let (attribute, _span) = parser.parse_attribute().unwrap();
        expect_no_errors(&parser.errors);
        let Attribute::Secondary(SecondaryAttribute::Meta(meta)) = attribute else {
            panic!("Expected meta attribute");
        };
        assert_eq!(meta.name.to_string(), "foo");
        assert!(meta.arguments.is_empty());
    }

    #[test]
    fn parses_meta_attribute_single_identifier_as_keyword() {
        let src = "#[dep]";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let (attribute, _span) = parser.parse_attribute().unwrap();
        expect_no_errors(&parser.errors);
        let Attribute::Secondary(SecondaryAttribute::Meta(meta)) = attribute else {
            panic!("Expected meta attribute");
        };
        assert_eq!(meta.name.to_string(), "dep");
        assert!(meta.arguments.is_empty());
    }

    #[test]
    fn parses_meta_attribute_single_identifier_with_arguments() {
        let src = "#[foo(1, 2, 3)]";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let (attribute, _span) = parser.parse_attribute().unwrap();
        expect_no_errors(&parser.errors);
        let Attribute::Secondary(SecondaryAttribute::Meta(meta)) = attribute else {
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
        let Attribute::Secondary(SecondaryAttribute::Meta(meta)) = attribute else {
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
        assert!(matches!(attr, Attribute::Function(FunctionAttribute::Test(TestScope::None))));

        let (attr, _) = attributes.remove(0);
        assert!(matches!(attr, Attribute::Secondary(SecondaryAttribute::Deprecated(None))));
    }
}
