use crate::ast::{
    BlockExpression, GenericTypeArgs, Ident, Path, Pattern, UnresolvedTraitConstraint,
    UnresolvedType,
};
use crate::token::{Attribute, Attributes, Keyword, Token};
use crate::{ast::UnresolvedGenerics, parser::labels::ParsingRuleLabel};
use crate::{
    ast::{
        FunctionDefinition, FunctionReturnType, ItemVisibility, NoirFunction, Param,
        UnresolvedTypeData, Visibility,
    },
    parser::ParserErrorReason,
};
use acvm::AcirField;

use noirc_errors::Span;

use super::parse_many::separated_by_comma_until_right_paren;
use super::pattern::SelfPattern;
use super::{pattern::PatternOrSelf, Parser};

pub(crate) struct FunctionDefinitionWithOptionalBody {
    pub(crate) name: Ident,
    pub(crate) generics: UnresolvedGenerics,
    pub(crate) parameters: Vec<Param>,
    pub(crate) body: Option<BlockExpression>,
    pub(crate) span: Span,
    pub(crate) where_clause: Vec<UnresolvedTraitConstraint>,
    pub(crate) return_type: FunctionReturnType,
    pub(crate) return_visibility: Visibility,
}

impl<'a> Parser<'a> {
    /// Function = 'fn' identifier Generics FunctionParameters ( '->' Visibility Type )? WhereClause ( Block | ';' )
    pub(crate) fn parse_function(
        &mut self,
        attributes: Vec<(Attribute, Span)>,
        visibility: ItemVisibility,
        is_comptime: bool,
        is_unconstrained: bool,
        allow_self: bool,
    ) -> NoirFunction {
        self.parse_function_definition(
            attributes,
            visibility,
            is_comptime,
            is_unconstrained,
            allow_self,
        )
        .into()
    }

    pub(crate) fn parse_function_definition(
        &mut self,
        attributes: Vec<(Attribute, Span)>,
        visibility: ItemVisibility,
        is_comptime: bool,
        is_unconstrained: bool,
        allow_self: bool,
    ) -> FunctionDefinition {
        let attributes = self.validate_attributes(attributes);

        let func = self.parse_function_definition_with_optional_body(
            false, // allow optional body
            allow_self,
        );

        FunctionDefinition {
            name: func.name,
            attributes,
            is_unconstrained,
            is_comptime,
            visibility,
            generics: func.generics,
            parameters: func.parameters,
            body: func.body.unwrap_or_else(empty_body),
            span: func.span,
            where_clause: func.where_clause,
            return_type: func.return_type,
            return_visibility: func.return_visibility,
        }
    }

    pub(super) fn parse_function_definition_with_optional_body(
        &mut self,
        allow_optional_body: bool,
        allow_self: bool,
    ) -> FunctionDefinitionWithOptionalBody {
        let Some(name) = self.eat_ident() else {
            self.expected_identifier();
            return empty_function(self.previous_token_span);
        };

        let generics = self.parse_generics();
        let parameters = self.parse_function_parameters(allow_self);

        let parameters = match parameters {
            Some(parameters) => parameters,
            None => {
                self.push_error(
                    ParserErrorReason::MissingParametersForFunctionDefinition,
                    name.span(),
                );
                Vec::new()
            }
        };

        let (return_type, return_visibility) = if self.eat(Token::Arrow) {
            let visibility = self.parse_visibility();
            (FunctionReturnType::Ty(self.parse_type_or_error()), visibility)
        } else {
            (FunctionReturnType::Default(self.span_at_previous_token_end()), Visibility::Private)
        };

        let where_clause = self.parse_where_clause();

        let body_start_span = self.current_token_span;
        let body = if self.eat_semicolons() {
            if !allow_optional_body {
                self.push_error(ParserErrorReason::ExpectedFunctionBody, body_start_span);
            }

            None
        } else {
            Some(self.parse_block().unwrap_or_else(empty_body))
        };

        FunctionDefinitionWithOptionalBody {
            name,
            generics,
            parameters,
            body,
            span: self.span_since(body_start_span),
            where_clause,
            return_type,
            return_visibility,
        }
    }

    /// FunctionParameters = '(' FunctionParametersList? ')'
    ///
    /// FunctionParametersList = FunctionParameter ( ',' FunctionParameter )* ','?
    ///
    /// FunctionParameter = Visibility PatternOrSelf ':' Type
    fn parse_function_parameters(&mut self, allow_self: bool) -> Option<Vec<Param>> {
        if !self.eat_left_paren() {
            return None;
        }

        Some(self.parse_many("parameters", separated_by_comma_until_right_paren(), |parser| {
            parser.parse_function_parameter(allow_self)
        }))
    }

    fn parse_function_parameter(&mut self, allow_self: bool) -> Option<Param> {
        loop {
            let start_span = self.current_token_span;

            let pattern_or_self = if allow_self {
                self.parse_pattern_or_self()
            } else {
                self.parse_pattern().map(PatternOrSelf::Pattern)
            };

            let Some(pattern_or_self) = pattern_or_self else {
                self.expected_label(ParsingRuleLabel::Pattern);

                // Let's try with the next token
                self.bump();
                if self.at_eof() {
                    return None;
                } else {
                    continue;
                }
            };

            return Some(match pattern_or_self {
                PatternOrSelf::Pattern(pattern) => self.pattern_param(pattern, start_span),
                PatternOrSelf::SelfPattern(self_pattern) => self.self_pattern_param(self_pattern),
            });
        }
    }

    fn pattern_param(&mut self, pattern: Pattern, start_span: Span) -> Param {
        let (visibility, typ) = if !self.eat_colon() {
            self.push_error(
                ParserErrorReason::MissingTypeForFunctionParameter,
                Span::from(pattern.span().start()..self.current_token_span.end()),
            );

            let visibility = Visibility::Private;
            let typ = UnresolvedType { typ: UnresolvedTypeData::Error, span: Span::default() };
            (visibility, typ)
        } else {
            (self.parse_visibility(), self.parse_type_or_error())
        };

        Param { visibility, pattern, typ, span: self.span_since(start_span) }
    }

    fn self_pattern_param(&mut self, self_pattern: SelfPattern) -> Param {
        let ident_span = self.previous_token_span;
        let ident = Ident::new("self".to_string(), ident_span);
        let path = Path::from_single("Self".to_owned(), ident_span);
        let no_args = GenericTypeArgs::default();
        let mut self_type = UnresolvedTypeData::Named(path, no_args, true).with_span(ident_span);
        let mut pattern = Pattern::Identifier(ident);

        if self_pattern.reference {
            self_type =
                UnresolvedTypeData::MutableReference(Box::new(self_type)).with_span(ident_span);
        } else if self_pattern.mutable {
            pattern = Pattern::Mutable(Box::new(pattern), ident_span, true);
        }

        Param {
            visibility: Visibility::Private,
            pattern,
            typ: self_type,
            span: self.span_since(ident_span),
        }
    }

    /// Visibility
    ///     = 'pub'
    ///     | 'return_data'
    ///     | 'call_data' '(' int ')'
    ///     | nothing
    fn parse_visibility(&mut self) -> Visibility {
        if self.eat_keyword(Keyword::Pub) {
            return Visibility::Public;
        }

        if self.eat_keyword(Keyword::ReturnData) {
            return Visibility::ReturnData;
        }

        if self.eat_keyword(Keyword::CallData) {
            if self.eat_left_paren() {
                if let Some(int) = self.eat_int() {
                    self.eat_or_error(Token::RightParen);

                    let id = int.to_u128() as u32;
                    return Visibility::CallData(id);
                } else {
                    self.expected_label(ParsingRuleLabel::Integer);
                    self.eat_right_paren();
                    return Visibility::CallData(0);
                }
            } else {
                self.expected_token(Token::LeftParen);
                return Visibility::CallData(0);
            }
        }

        Visibility::Private
    }

    fn validate_attributes(&mut self, attributes: Vec<(Attribute, Span)>) -> Attributes {
        let mut function = None;
        let mut secondary = Vec::new();

        for (index, (attribute, span)) in attributes.into_iter().enumerate() {
            match attribute {
                Attribute::Function(attr) => {
                    if function.is_none() {
                        function = Some((attr, index));
                    } else {
                        self.push_error(ParserErrorReason::MultipleFunctionAttributesFound, span);
                    }
                }
                Attribute::Secondary(attr) => secondary.push(attr),
            }
        }

        Attributes { function, secondary }
    }
}

fn empty_function(span: Span) -> FunctionDefinitionWithOptionalBody {
    FunctionDefinitionWithOptionalBody {
        name: Ident::default(),
        generics: Vec::new(),
        parameters: Vec::new(),
        body: None,
        span: Span::from(span.end()..span.end()),
        where_clause: Vec::new(),
        return_type: FunctionReturnType::Default(Span::default()),
        return_visibility: Visibility::Private,
    }
}

fn empty_body() -> BlockExpression {
    BlockExpression { statements: Vec::new() }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{ItemVisibility, NoirFunction, UnresolvedTypeData, Visibility},
        parser::{
            parser::{
                parse_program,
                tests::{
                    expect_no_errors, get_single_error, get_single_error_reason,
                    get_source_with_error_span,
                },
            },
            ItemKind, ParserErrorReason,
        },
    };

    fn parse_function_no_error(src: &str) -> NoirFunction {
        let (mut module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Function(noir_function) = item.kind else {
            panic!("Expected function");
        };
        noir_function
    }

    #[test]
    fn parse_simple_function() {
        let src = "fn foo() {}";
        let noir_function = parse_function_no_error(src);
        assert_eq!("foo", noir_function.def.name.to_string());
        assert!(noir_function.def.parameters.is_empty());
        assert!(noir_function.def.generics.is_empty());
    }

    #[test]
    fn parse_function_with_generics() {
        let src = "fn foo<A>() {}";
        let noir_function = parse_function_no_error(src);
        assert_eq!(noir_function.def.generics.len(), 1);
    }

    #[test]
    fn parse_function_with_arguments() {
        let src = "fn foo(x: Field, y: Field) {}";
        let mut noir_function = parse_function_no_error(src);
        assert_eq!(noir_function.def.parameters.len(), 2);

        let param = noir_function.def.parameters.remove(0);
        assert_eq!("x", param.pattern.to_string());
        assert_eq!("Field", param.typ.to_string());
        assert_eq!(param.visibility, Visibility::Private);

        let param = noir_function.def.parameters.remove(0);
        assert_eq!("y", param.pattern.to_string());
        assert_eq!("Field", param.typ.to_string());
        assert_eq!(param.visibility, Visibility::Private);
    }

    #[test]
    fn parse_function_with_argument_pub_visibility() {
        let src = "fn foo(x: pub Field) {}";
        let mut noir_function = parse_function_no_error(src);
        assert_eq!(noir_function.def.parameters.len(), 1);

        let param = noir_function.def.parameters.remove(0);
        assert_eq!("x", param.pattern.to_string());
        assert_eq!("Field", param.typ.to_string());
        assert_eq!(param.visibility, Visibility::Public);
    }

    #[test]
    fn parse_function_with_argument_return_data_visibility() {
        let src = "fn foo(x: return_data Field) {}";
        let mut noir_function = parse_function_no_error(src);
        assert_eq!(noir_function.def.parameters.len(), 1);

        let param = noir_function.def.parameters.remove(0);
        assert_eq!(param.visibility, Visibility::ReturnData);
    }

    #[test]
    fn parse_function_with_argument_call_data_visibility() {
        let src = "fn foo(x: call_data(42) Field) {}";
        let mut noir_function = parse_function_no_error(src);
        assert_eq!(noir_function.def.parameters.len(), 1);

        let param = noir_function.def.parameters.remove(0);
        assert_eq!(param.visibility, Visibility::CallData(42));
    }

    #[test]
    fn parse_function_return_type() {
        let src = "fn foo() -> Field {}";
        let noir_function = parse_function_no_error(src);
        assert_eq!(noir_function.def.return_visibility, Visibility::Private);
        assert_eq!(noir_function.return_type().typ, UnresolvedTypeData::FieldElement);
    }

    #[test]
    fn parse_function_return_visibility() {
        let src = "fn foo() -> pub Field {}";
        let noir_function = parse_function_no_error(src);
        assert_eq!(noir_function.def.return_visibility, Visibility::Public);
        assert_eq!(noir_function.return_type().typ, UnresolvedTypeData::FieldElement);
    }

    #[test]
    fn parse_function_unclosed_parentheses() {
        let src = "fn foo(x: i32,";
        let (module, errors) = parse_program(src);
        assert_eq!(errors.len(), 1);
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Function(noir_function) = &item.kind else {
            panic!("Expected function");
        };
        assert_eq!("foo", noir_function.def.name.to_string());
    }

    #[test]
    fn parse_error_multiple_function_attributes_found() {
        let src = "
        #[foreign(foo)] #[oracle(bar)] fn foo() {}
                        ^^^^^^^^^^^^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let (_, errors) = parse_program(&src);
        let reason = get_single_error_reason(&errors, span);
        assert!(matches!(reason, ParserErrorReason::MultipleFunctionAttributesFound));
    }

    #[test]
    fn parse_function_found_semicolon_instead_of_braces() {
        let src = "
        fn foo();
                ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let (_, errors) = parse_program(&src);
        let reason = get_single_error_reason(&errors, span);
        assert!(matches!(reason, ParserErrorReason::ExpectedFunctionBody));
    }

    #[test]
    fn recovers_on_wrong_parameter_name() {
        let src = "
        fn foo(1 x: i32) {}
               ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let (module, errors) = parse_program(&src);
        assert_eq!(module.items.len(), 1);
        let ItemKind::Function(noir_function) = &module.items[0].kind else {
            panic!("Expected function");
        };
        assert_eq!(noir_function.parameters().len(), 1);

        let error = get_single_error(&errors, span);
        assert_eq!(error.to_string(), "Expected a pattern but found '1'");
    }

    #[test]
    fn recovers_on_missing_colon_after_parameter_name() {
        let src = "
        fn foo(x, y: i32) {}
               ^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let (module, errors) = parse_program(&src);
        assert_eq!(module.items.len(), 1);
        let ItemKind::Function(noir_function) = &module.items[0].kind else {
            panic!("Expected function");
        };
        assert_eq!(noir_function.parameters().len(), 2);

        let error = get_single_error(&errors, span);
        assert!(error.to_string().contains("Missing type for function parameter"));
    }

    #[test]
    fn recovers_on_missing_type_after_parameter_colon() {
        let src = "
        fn foo(x: , y: i32) {}
                  ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let (module, errors) = parse_program(&src);
        assert_eq!(module.items.len(), 1);
        let ItemKind::Function(noir_function) = &module.items[0].kind else {
            panic!("Expected function");
        };
        assert_eq!(noir_function.parameters().len(), 2);

        let error = get_single_error(&errors, span);
        assert_eq!(error.to_string(), "Expected a type but found ','");
    }

    #[test]
    fn parse_function_with_unconstrained_after_visibility() {
        let src = "pub unconstrained fn foo() {}";
        let noir_function = parse_function_no_error(src);
        assert_eq!("foo", noir_function.def.name.to_string());
        assert!(noir_function.def.is_unconstrained);
        assert_eq!(noir_function.def.visibility, ItemVisibility::Public);
    }

    #[test]
    fn parse_function_without_parentheses() {
        let src = "
        fn foo {}
           ^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let (_, errors) = parse_program(&src);
        let reason = get_single_error_reason(&errors, span);
        assert!(matches!(reason, ParserErrorReason::MissingParametersForFunctionDefinition));
    }
}
