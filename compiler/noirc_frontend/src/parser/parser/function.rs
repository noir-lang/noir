use super::{
    attributes::{attributes, validate_attributes},
    block, fresh_statement, ident, keyword, nothing, optional_distinctness, optional_visibility,
    parameter_name_recovery, parameter_recovery, parenthesized, parse_type, pattern,
    self_parameter, where_clause, NoirParser,
};
use crate::parser::labels::ParsingRuleLabel;
use crate::parser::spanned;
use crate::token::{Keyword, Token};
use crate::{
    Distinctness, FunctionDefinition, FunctionReturnType, FunctionVisibility, Ident, NoirFunction,
    Param, Visibility,
};

use chumsky::prelude::*;

/// function_definition: attribute function_modifiers 'fn' ident generics '(' function_parameters ')' function_return_type block
///                      function_modifiers 'fn' ident generics '(' function_parameters ')' function_return_type block
pub(super) fn function_definition(allow_self: bool) -> impl NoirParser<NoirFunction> {
    attributes()
        .then(function_modifiers())
        .then_ignore(keyword(Keyword::Fn))
        .then(ident())
        .then(generics())
        .then(parenthesized(function_parameters(allow_self)))
        .then(function_return_type())
        .then(where_clause())
        .then(spanned(block(fresh_statement())))
        .validate(|(((args, ret), where_clause), (body, body_span)), span, emit| {
            let ((((attributes, modifiers), name), generics), parameters) = args;

            // Validate collected attributes, filtering them into function and secondary variants
            let attributes = validate_attributes(attributes, span, emit);
            FunctionDefinition {
                span: body_span,
                name,
                attributes,
                is_unconstrained: modifiers.0,
                is_open: modifiers.2,
                is_internal: modifiers.3,
                visibility: if modifiers.1 {
                    FunctionVisibility::PublicCrate
                } else if modifiers.4 {
                    FunctionVisibility::Public
                } else {
                    FunctionVisibility::Private
                },
                generics,
                parameters,
                body,
                where_clause,
                return_type: ret.1,
                return_visibility: ret.0 .1,
                return_distinctness: ret.0 .0,
            }
            .into()
        })
}

/// function_modifiers: 'unconstrained'? 'pub(crate)'? 'pub'? 'open'? 'internal'?
///
/// returns (is_unconstrained, is_pub_crate, is_open, is_internal, is_pub) for whether each keyword was present
fn function_modifiers() -> impl NoirParser<(bool, bool, bool, bool, bool)> {
    keyword(Keyword::Unconstrained)
        .or_not()
        .then(is_pub_crate())
        .then(keyword(Keyword::Pub).or_not())
        .then(keyword(Keyword::Open).or_not())
        .then(keyword(Keyword::Internal).or_not())
        .map(|((((unconstrained, pub_crate), public), open), internal)| {
            (
                unconstrained.is_some(),
                pub_crate,
                open.is_some(),
                internal.is_some(),
                public.is_some(),
            )
        })
}

fn is_pub_crate() -> impl NoirParser<bool> {
    (keyword(Keyword::Pub)
        .then_ignore(just(Token::LeftParen))
        .then_ignore(keyword(Keyword::Crate))
        .then_ignore(just(Token::RightParen)))
    .or_not()
    .map(|a| a.is_some())
}

/// non_empty_ident_list: ident ',' non_empty_ident_list
///                     | ident
///
/// generics: '<' non_empty_ident_list '>'
///         | %empty
pub(super) fn generics() -> impl NoirParser<Vec<Ident>> {
    ident()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .at_least(1)
        .delimited_by(just(Token::Less), just(Token::Greater))
        .or_not()
        .map(|opt| opt.unwrap_or_default())
}

fn function_return_type() -> impl NoirParser<((Distinctness, Visibility), FunctionReturnType)> {
    just(Token::Arrow)
        .ignore_then(optional_distinctness())
        .then(optional_visibility())
        .then(spanned(parse_type()))
        .or_not()
        .map_with_span(|ret, span| match ret {
            Some((head, (ty, _))) => (head, FunctionReturnType::Ty(ty)),
            None => (
                (Distinctness::DuplicationAllowed, Visibility::Private),
                FunctionReturnType::Default(span),
            ),
        })
}

fn function_parameters<'a>(allow_self: bool) -> impl NoirParser<Vec<Param>> + 'a {
    let typ = parse_type().recover_via(parameter_recovery());

    let full_parameter = pattern()
        .recover_via(parameter_name_recovery())
        .then_ignore(just(Token::Colon))
        .then(optional_visibility())
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
    use crate::parser::parser::test_helpers::*;

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
                "fn main(x: pub u8, y: pub u8) -> distinct pub [u8; 2] { [x, y] }",
                "fn f(f: pub Field, y : Field, z : comptime Field) -> u8 { x + a }",
                "fn f<T>(f: pub Field, y : T, z : comptime Field) -> u8 { x + a }",
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
            ],
        );
    }
}
