use acvm::AcirField;
use noirc_errors::Span;

use crate::{
    ast::{
        BlockExpression, FunctionDefinition, FunctionReturnType, GenericTypeArgs, Ident,
        ItemVisibility, NoirFunction, Param, Path, Pattern, UnresolvedGenerics,
        UnresolvedTraitConstraint, UnresolvedType, UnresolvedTypeData, Visibility,
    },
    parser::ParserErrorReason,
    token::{Attribute, Attributes, Keyword, Token},
};

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
            self.push_error(ParserErrorReason::ExpectedIdentifierAfterFn, self.current_token_span);
            return empty_function(self.previous_token_span);
        };

        let generics = self.parse_generics();
        let parameters = self.parse_function_parameters(allow_self);

        let (return_type, return_visibility) = if self.eat(Token::Arrow) {
            let visibility = self.parse_visibility();
            (FunctionReturnType::Ty(self.parse_type()), visibility)
        } else {
            (
                FunctionReturnType::Default(Span::from(
                    self.previous_token_span.end()..self.previous_token_span.end(),
                )),
                Visibility::Private,
            )
        };

        let where_clause = self.parse_where_clause();

        let body_start_span = self.current_token_span;
        let (body, body_span) = if self.eat_semicolons() {
            if !allow_optional_body {
                // TODO: error
            }

            (None, Span::from(body_start_span.end()..body_start_span.end()))
        } else {
            (
                Some(self.parse_block_expression().unwrap_or_else(empty_body)),
                self.span_since(body_start_span),
            )
        };

        FunctionDefinitionWithOptionalBody {
            name,
            generics,
            parameters,
            body,
            span: body_span,
            where_clause,
            return_type,
            return_visibility,
        }
    }

    fn parse_function_parameters(&mut self, allow_self: bool) -> Vec<Param> {
        let mut parameters = Vec::new();

        if !self.eat_left_paren() {
            return parameters;
        }

        let mut trailing_comma = false;

        loop {
            if self.eat_right_paren() {
                break;
            }

            let start_span = self.current_token_span;
            let pattern_or_self = if allow_self && parameters.is_empty() {
                self.parse_pattern_or_self()
            } else {
                PatternOrSelf::Pattern(self.parse_pattern())
            };
            if self.current_token_span == start_span {
                // An error was already produced by parse_pattern().
                // Let's try with the next token.
                self.next_token();
                if self.is_eof() {
                    break;
                }
                continue;
            }

            if !trailing_comma && !parameters.is_empty() {
                self.push_error(ParserErrorReason::MissingCommaSeparatingParameters, start_span);
            }

            match pattern_or_self {
                PatternOrSelf::Pattern(pattern) => {
                    if self.eat_colon() {
                        let visibility = self.parse_visibility();

                        let typ = self.parse_type();
                        parameters.push(Param {
                            visibility,
                            pattern,
                            typ,
                            span: self.span_since(start_span),
                        });
                    } else {
                        self.push_error(
                            ParserErrorReason::MissingTypeForFunctionParameter,
                            self.previous_token_span,
                        );

                        parameters.push(Param {
                            visibility: Visibility::Private,
                            pattern,
                            typ: UnresolvedType {
                                typ: UnresolvedTypeData::Error,
                                span: Span::default(),
                            },
                            span: self.span_since(start_span),
                        });
                    }
                }
                PatternOrSelf::SelfPattern(self_pattern) => {
                    let span = self.previous_token_span;
                    let ident = Ident::new("self".to_string(), span);
                    let path = Path::from_single("Self".to_owned(), span);
                    let no_args = GenericTypeArgs::default();
                    let mut self_type =
                        UnresolvedTypeData::Named(path, no_args, true).with_span(span);
                    let mut pattern = Pattern::Identifier(ident);

                    if self_pattern.reference {
                        self_type = UnresolvedTypeData::MutableReference(Box::new(self_type))
                            .with_span(self.span_since(start_span));
                    } else if self_pattern.mutable {
                        pattern =
                            Pattern::Mutable(Box::new(pattern), self.span_since(start_span), true);
                    }

                    parameters.push(Param {
                        visibility: Visibility::Private,
                        pattern,
                        typ: self_type,
                        span: self.span_since(start_span),
                    });
                }
            }

            trailing_comma = self.eat_commas();
        }

        parameters
    }

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
                    if !self.eat_right_paren() {
                        self.push_error(
                            ParserErrorReason::ExpectedRightParen,
                            self.current_token_span,
                        );
                    }

                    let id = int.to_u128() as u32;
                    return Visibility::CallData(id);
                } else {
                    self.push_error(ParserErrorReason::ExpectedInteger, self.current_token_span);
                    self.eat_right_paren();
                    return Visibility::CallData(0);
                }
            } else {
                self.push_error(ParserErrorReason::ExpectedLeftParen, self.current_token_span);
                return Visibility::CallData(0);
            }
        }

        Visibility::Private
    }

    fn validate_attributes(&mut self, attributes: Vec<(Attribute, Span)>) -> Attributes {
        let mut primary = None;
        let mut secondary = Vec::new();

        for (attribute, span) in attributes {
            match attribute {
                Attribute::Function(attr) => {
                    if primary.is_some() {
                        self.push_error(ParserErrorReason::MultipleFunctionAttributesFound, span);
                    }
                    primary = Some(attr);
                }
                Attribute::Secondary(attr) => secondary.push(attr),
            }
        }

        Attributes { function: primary, secondary }
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
        ast::{UnresolvedTypeData, Visibility},
        parser::{
            parser::{
                parse_program,
                tests::{get_single_error, get_source_with_error_span},
            },
            ItemKind, ParserErrorReason,
        },
    };

    #[test]
    fn parse_simple_function() {
        let src = "fn foo() {}";
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Function(noir_function) = &item.kind else {
            panic!("Expected function");
        };
        assert_eq!("foo", noir_function.def.name.to_string());
        assert!(noir_function.def.parameters.is_empty());
        assert!(noir_function.def.generics.is_empty());
    }

    #[test]
    fn parse_function_with_generics() {
        let src = "fn foo<A>() {}";
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Function(noir_function) = &item.kind else {
            panic!("Expected function");
        };
        assert_eq!(noir_function.def.generics.len(), 1);
    }

    #[test]
    fn parse_function_with_arguments() {
        let src = "fn foo(x: Field, y: Field) {}";
        let (mut module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Function(mut noir_function) = item.kind else {
            panic!("Expected function");
        };
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
        let (mut module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Function(mut noir_function) = item.kind else {
            panic!("Expected function");
        };
        assert_eq!(noir_function.def.parameters.len(), 1);

        let param = noir_function.def.parameters.remove(0);
        assert_eq!("x", param.pattern.to_string());
        assert_eq!("Field", param.typ.to_string());
        assert_eq!(param.visibility, Visibility::Public);
    }

    #[test]
    fn parse_function_with_argument_return_data_visibility() {
        let src = "fn foo(x: return_data Field) {}";
        let (mut module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Function(mut noir_function) = item.kind else {
            panic!("Expected function");
        };
        assert_eq!(noir_function.def.parameters.len(), 1);

        let param = noir_function.def.parameters.remove(0);
        assert_eq!(param.visibility, Visibility::ReturnData);
    }

    #[test]
    fn parse_function_with_argument_call_data_visibility() {
        let src = "fn foo(x: call_data(42) Field) {}";
        let (mut module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Function(mut noir_function) = item.kind else {
            panic!("Expected function");
        };
        assert_eq!(noir_function.def.parameters.len(), 1);

        let param = noir_function.def.parameters.remove(0);
        assert_eq!(param.visibility, Visibility::CallData(42));
    }

    #[test]
    fn parse_function_return_type() {
        let src = "fn foo() -> Field {}";
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Function(noir_function) = &item.kind else {
            panic!("Expected function");
        };
        assert_eq!(noir_function.def.return_visibility, Visibility::Private);
        assert_eq!(noir_function.return_type().typ, UnresolvedTypeData::FieldElement);
    }

    #[test]
    fn parse_function_return_visibility() {
        let src = "fn foo() -> pub Field {}";
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Function(noir_function) = &item.kind else {
            panic!("Expected function");
        };
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
        let reason = get_single_error(&errors, span);
        assert!(matches!(reason, ParserErrorReason::MultipleFunctionAttributesFound));
    }
}
