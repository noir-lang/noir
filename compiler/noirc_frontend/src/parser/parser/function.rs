use super::{
    attributes::{all_attributes, split_attributes_in_two, validate_attributes},
    block, fresh_statement, ident, keyword, maybe_comp_time, nothing, parameter_name_recovery,
    parameter_recovery, parenthesized, parse_type, pattern,
    primitives::token_kind,
    self_parameter,
    visibility::{item_visibility, visibility},
    where_clause, NoirParser,
};
use crate::token::{Keyword, Token, TokenKind};
use crate::{
    ast::{BlockExpression, IntegerBitSize},
    parser::spanned,
};
use crate::{
    ast::{
        FunctionDefinition, FunctionReturnType, ItemVisibility, NoirFunction, Param, Visibility,
    },
    macros_api::UnresolvedTypeData,
    parser::{ParserError, ParserErrorReason},
};
use crate::{
    ast::{Signedness, UnresolvedGeneric, UnresolvedGenerics},
    parser::labels::ParsingRuleLabel,
};

use chumsky::prelude::*;
use noirc_errors::Span;

/// function_definition: attribute function_modifiers 'fn' ident generics '(' function_parameters ')' function_return_type block
///                      function_modifiers 'fn' ident generics '(' function_parameters ')' function_return_type block
pub(super) fn function_definition(allow_self: bool) -> impl NoirParser<NoirFunction> {
    let body_or_error =
        spanned(block(fresh_statement()).or_not()).validate(|(body, body_span), span, emit| {
            if let Some(body) = body {
                (body, body_span)
            } else {
                emit(ParserError::with_reason(
                    ParserErrorReason::ExpectedLeftBraceOrArrowAfterFunctionParameters,
                    span,
                ));
                (BlockExpression { statements: vec![] }, Span::from(span.end()..span.end()))
            }
        });

    all_attributes()
        .then(function_modifiers())
        .then_ignore(keyword(Keyword::Fn))
        .then(ident())
        .then(generics())
        .then(
            parenthesized(function_parameters(allow_self))
                .then(function_return_type())
                .then(where_clause())
                .then(body_or_error)
                // Allow parsing just `fn foo` for recovery and LSP autocompletion
                .or_not(),
        )
        .validate(|args, span, emit| {
            let (
                (((all_attributes, (is_unconstrained, visibility, is_comptime)), name), generics),
                params_and_others,
            ) = args;

            let (fv_attributes, attributes) = split_attributes_in_two(all_attributes);

            // Validate collected attributes, filtering them into function and secondary variants
            let attributes = validate_attributes(attributes, fv_attributes, span, emit);
            let function_definition = if let Some(params_and_others) = params_and_others {
                let (
                    ((parameters, (return_visibility, return_type)), where_clause),
                    (body, body_span),
                ) = params_and_others;

                FunctionDefinition {
                    span: body_span,
                    name,
                    attributes,
                    is_unconstrained,
                    visibility,
                    is_comptime,
                    generics,
                    parameters,
                    body,
                    where_clause,
                    return_type,
                    return_visibility,
                }
            } else {
                emit(ParserError::with_reason(
                    ParserErrorReason::ExpectedLeftParenOrLeftBracketAfterFunctionName,
                    span,
                ));

                let empty_span = Span::from(span.end()..span.end());
                FunctionDefinition {
                    span: empty_span,
                    name,
                    attributes,
                    is_unconstrained,
                    visibility,
                    is_comptime,
                    generics,
                    parameters: Vec::new(),
                    body: BlockExpression { statements: vec![] },
                    where_clause: Vec::new(),
                    return_type: FunctionReturnType::Default(empty_span),
                    return_visibility: Visibility::Private,
                }
            };
            function_definition.into()
        })
}

/// function_modifiers: 'unconstrained'? (visibility)?
///
/// returns (is_unconstrained, visibility) for whether each keyword was present
pub(super) fn function_modifiers() -> impl NoirParser<(bool, ItemVisibility, bool)> {
    keyword(Keyword::Unconstrained).or_not().then(item_visibility()).then(maybe_comp_time()).map(
        |((unconstrained, visibility), comptime)| (unconstrained.is_some(), visibility, comptime),
    )
}

pub(super) fn numeric_generic() -> impl NoirParser<UnresolvedGeneric> {
    keyword(Keyword::Let)
        .ignore_then(ident())
        .then_ignore(just(Token::Colon))
        .then(parse_type())
        .map(|(ident, typ)| UnresolvedGeneric::Numeric { ident, typ })
        .validate(|generic, span, emit| {
            if let UnresolvedGeneric::Numeric { typ, .. } = &generic {
                if let UnresolvedTypeData::Integer(signedness, bit_size) = typ.typ {
                    if matches!(signedness, Signedness::Signed)
                        || matches!(bit_size, IntegerBitSize::SixtyFour)
                    {
                        emit(ParserError::with_reason(
                            ParserErrorReason::ForbiddenNumericGenericType,
                            span,
                        ));
                    }
                }
            }
            generic
        })
}

pub(super) fn generic_type() -> impl NoirParser<UnresolvedGeneric> {
    ident().map(UnresolvedGeneric::Variable)
}

pub(super) fn resolved_generic() -> impl NoirParser<UnresolvedGeneric> {
    token_kind(TokenKind::QuotedType).map_with_span(|token, span| match token {
        Token::QuotedType(id) => UnresolvedGeneric::Resolved(id, span),
        _ => unreachable!("token_kind(QuotedType) guarantees we parse a quoted type"),
    })
}

pub(super) fn generic() -> impl NoirParser<UnresolvedGeneric> {
    generic_type().or(numeric_generic()).or(resolved_generic())
}

/// non_empty_ident_list: ident ',' non_empty_ident_list
///                     | ident
///
/// generics: '<' non_empty_ident_list '>'
///         | %empty
pub(super) fn generics() -> impl NoirParser<UnresolvedGenerics> {
    generic()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .delimited_by(just(Token::Less), just(Token::Greater))
        .or_not()
        .map(|opt| opt.unwrap_or_default())
}

pub(super) fn function_return_type() -> impl NoirParser<(Visibility, FunctionReturnType)> {
    #[allow(deprecated)]
    just(Token::Arrow).ignore_then(visibility()).then(spanned(parse_type())).or_not().map_with_span(
        |ret, span| match ret {
            Some((visibility, (ty, _))) => (visibility, FunctionReturnType::Ty(ty)),
            None => (Visibility::Private, FunctionReturnType::Default(span)),
        },
    )
}

fn function_parameters<'a>(allow_self: bool) -> impl NoirParser<Vec<Param>> + 'a {
    let typ = parse_type().recover_via(parameter_recovery());

    let full_parameter = pattern()
        .recover_via(parameter_name_recovery())
        .then_ignore(just(Token::Colon))
        .then(visibility())
        .then(typ)
        .map_with_span(|((pattern, visibility), typ), span| Param {
            visibility,
            pattern,
            typ,
            span,
        });

    let self_parameter = if allow_self { self_parameter().boxed() } else { nothing().boxed() };

    let parameter = full_parameter.or(self_parameter);

    parameter
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .labelled(ParsingRuleLabel::Parameter)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        parser::parser::test_helpers::*,
        token::{FormalVerificationAttribute, SecondaryAttribute},
    };

    #[test]
    fn regression_skip_comment() {
        parse_all(
            function_definition(false),
            vec![
                "fn main(
                // This comment should be skipped
                x : Field,
                // And this one
                y : Field,
            ) {
            }",
                "fn main(x : Field, y : Field,) {
                foo::bar(
                    // Comment for x argument
                    x,
                    // Comment for y argument
                    y
                )
            }",
            ],
        );
    }

    #[test]
    fn parse_function() {
        parse_all(
            function_definition(false),
            vec![
                "fn func_name() {}",
                "fn f(foo: pub u8, y : pub Field) -> u8 { x + a }",
                "fn f(f: pub Field, y : Field, z : Field) -> u8 { x + a }",
                "fn func_name(f: Field, y : pub Field, z : pub [u8;5],) {}",
                "fn f(f: pub Field, y : Field, z : Field) -> u8 { x + a }",
                "fn f<T>(f: pub Field, y : T, z : Field) -> u8 { x + a }",
                "fn func_name(x: [Field], y : [Field;2],y : pub [Field;2], z : pub [u8;5])  {}",
                "fn main(x: pub u8, y: pub u8) -> pub [u8; 2] { [x, y] }",
                "fn f(f: pub Field, y : Field, z : Field) -> u8 { x + a }",
                "fn f<T>(f: pub Field, y : T, z : Field) -> u8 { x + a }",
                "fn func_name<T>(f: Field, y : T) where T: SomeTrait {}",
                "fn func_name<T>(f: Field, y : T) where T: SomeTrait + SomeTrait2 {}",
                "fn func_name<T>(f: Field, y : T) where T: SomeTrait, T: SomeTrait2 {}",
                "fn func_name<T>(f: Field, y : T) where T: SomeTrait<A> + SomeTrait2 {}",
                "fn func_name<T>(f: Field, y : T) where T: SomeTrait<A, B> + SomeTrait2 {}",
                "fn func_name<T>(f: Field, y : T) where T: SomeTrait<A, B> + SomeTrait2<C> {}",
                "fn func_name<T>(f: Field, y : T) where T: SomeTrait + SomeTrait2<C> {}",
                "fn func_name<T>(f: Field, y : T) where T: SomeTrait + SomeTrait2<C> + TraitY {}",
                "fn func_name<T>(f: Field, y : T, z : U) where SomeStruct<T>: SomeTrait<U> {}",
                // 'where u32: SomeTrait' is allowed in Rust.
                // It will result in compiler error in case SomeTrait isn't implemented for u32.
                "fn func_name<T>(f: Field, y : T) where u32: SomeTrait {}",
                // A trailing plus is allowed by Rust, so we support it as well.
                "fn func_name<T>(f: Field, y : T) where T: SomeTrait + {}",
                // The following should produce compile error on later stage. From the parser's perspective it's fine
                "fn func_name<A>(f: Field, y : Field, z : Field) where T: SomeTrait {}",
                // TODO: this fails with known EOF != EOF error
                // https://github.com/noir-lang/noir/issues/4763
                // fn func_name(x: impl Eq) {} with error Expected an end of input but found end of input
                // "fn func_name(x: impl Eq) {}",
                "fn func_name<T>(x: impl Eq, y : T) where T: SomeTrait + Eq {}",
                "fn func_name<let N: u32>(x: [Field; N]) {}",
            ],
        );

        parse_all_failing(
            function_definition(false),
            vec![
                "fn x2( f: []Field,,) {}",
                "fn ( f: []Field) {}",
                "fn ( f: []Field) {}",
                // TODO: Check for more specific error messages
                "fn func_name<T>(f: Field, y : pub Field, z : pub [u8;5],) where T: {}",
                "fn func_name<T>(f: Field, y : pub Field, z : pub [u8;5],) where SomeTrait {}",
                "fn func_name<T>(f: Field, y : pub Field, z : pub [u8;5],) SomeTrait {}",
                // A leading plus is not allowed.
                "fn func_name<T>(f: Field, y : T) where T: + SomeTrait {}",
                "fn func_name<T>(f: Field, y : T) where T: TraitX + <Y> {}",
                // Test ill-formed numeric generics
                "fn func_name<let T>(y: T) {}",
                "fn func_name<let T:>(y: T) {}",
                "fn func_name<T:>(y: T) {}",
                // Test failure of missing `let`
                "fn func_name<T: u32>(y: T) {}",
                // Test that signed numeric generics are banned
                "fn func_name<let N: i8>() {}",
                // Test that `u64` is banned
                "fn func_name<let N: u64>(x: [Field; N]) {}",
            ],
        );
    }

    #[test]
    fn parse_recover_function_without_body() {
        let src = "fn foo(x: i32)";

        let (noir_function, errors) = parse_recover(function_definition(false), src);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "expected { or -> after function parameters");

        let noir_function = noir_function.unwrap();
        assert_eq!(noir_function.name(), "foo");
        assert_eq!(noir_function.parameters().len(), 1);
        assert!(noir_function.def.body.statements.is_empty());
    }

    #[test]
    fn formal_verification_requires_attribute_parses() {
        let src = r#"#[requires(x > 2)]
                    fn foo(x: i32) {}"#;

        let function_definition = parse_with(function_definition(false), src).unwrap();
        let parsed_attributes = &function_definition.attributes().fv_attributes;

        assert_eq!(parsed_attributes.len(), 1, "Missmatching number of attributes.");
        let FormalVerificationAttribute::Requires(_) = parsed_attributes[0] else {
            panic!("Expected 'requires', but got {:?}.", parsed_attributes[0]);
        };
    }

    #[test]
    fn formal_verification_both_attributes_parse() {
        let src = r#"#[requires(x > 2)]
                    #[ensures(result < 8)]
                    fn foo(x: i32) {}"#;

        let function_definition = parse_with(function_definition(false), src).unwrap();
        let parsed_attributes = &function_definition.attributes().fv_attributes;

        assert_eq!(parsed_attributes.len(), 2, "Missmatching number of attributes.");
        let FormalVerificationAttribute::Requires(_) = parsed_attributes[0] else {
            panic!("Expected 'requires', but got {:?}.", parsed_attributes[0]);
        };
        let FormalVerificationAttribute::Ensures(_) = parsed_attributes[1] else {
            panic!("Expected 'ensures', but got {:?}.", parsed_attributes[1]);
        };
    }

    #[test]
    fn formal_verification_attributes_cooperate() {
        let src = r#"#[requires(x > 2)]
                    #[deprecated]
                    #[ensures(result < 8)]
                    #[requires(x < 5)]
                    fn foo(x: i32) {}"#;

        let function_definition = parse_with(function_definition(false), src).unwrap();
        let parsed_fv_attributes = &function_definition.attributes().fv_attributes;
        let parsed_attributes = &function_definition.attributes().secondary;

        // Check that the formal verification attributes are parsed correctly.
        assert_eq!(parsed_fv_attributes.len(), 3, "Expected 3 formal verification attributes.");
        let FormalVerificationAttribute::Requires(_) = parsed_fv_attributes[0] else {
            panic!("Expected 'requires', but got {:?}.", parsed_fv_attributes[0]);
        };
        let FormalVerificationAttribute::Ensures(_) = parsed_fv_attributes[1] else {
            panic!("Expected 'ensures', but got {:?}.", parsed_fv_attributes[1]);
        };
        let FormalVerificationAttribute::Requires(_) = parsed_fv_attributes[2] else {
            panic!("Expected 'requires', but got {:?}.", parsed_fv_attributes[2]);
        };

        // Check that the other attributes are not botched by this.
        assert_eq!(parsed_attributes.len(), 1, "Expected 1 secondary attributes.");
        let SecondaryAttribute::Deprecated(_) = parsed_attributes[0] else {
            panic!("Expected 'deprecated', but got {:?}.", parsed_attributes[0]);
        };
    }

    #[test]
    fn formal_verification_wrong_attribute_defs() {
        parse_all_failing(
            function_definition(false),
            vec![
                "#[requires(x > 2) fn foo(x: i32) {}",
                "#[ensures(result > 5] fn foo(x: i32) {}",
                "#[ensures result > 5)] fn foo(x: i32) {}",
                "#[ensures result > 5] fn foo(x: i32) {}",
                "#[requires] fn foo(x: i32) {}",
                "#[ensures] fn foo(x: i32) {}",
                "#[ensures()] fn foo(x: i32) {}",
                "#[ensures(result > 4)x] fn foo(x: i32) {}",
            ],
        );
    }
}
