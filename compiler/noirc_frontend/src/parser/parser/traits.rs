use chumsky::prelude::*;

use super::attributes::{attributes, validate_secondary_attributes};
use super::function::function_return_type;
use super::types::maybe_comp_time;
use super::{block, expression, fresh_statement, function, function_declaration_parameters};

use crate::ast::{
    Expression, ItemVisibility, NoirTrait, NoirTraitImpl, TraitBound, TraitImplItem, TraitItem,
    UnresolvedTraitConstraint, UnresolvedType,
};
use crate::{
    parser::{
        ignore_then_commit, parenthesized, parser::primitives::keyword, NoirParser, ParserError,
        ParserErrorReason, TopLevelStatement,
    },
    token::{Keyword, Token},
};

use super::{generic_type_args, parse_type, path, primitives::ident};

pub(super) fn trait_definition() -> impl NoirParser<TopLevelStatement> {
    attributes()
        .then_ignore(keyword(Keyword::Trait))
        .then(ident())
        .then(function::generics())
        .then(where_clause())
        .then_ignore(just(Token::LeftBrace))
        .then(trait_body())
        .then_ignore(just(Token::RightBrace))
        .validate(|((((attributes, name), generics), where_clause), items), span, emit| {
            let attributes = validate_secondary_attributes(attributes, span, emit);
            TopLevelStatement::Trait(NoirTrait {
                name,
                generics,
                where_clause,
                span,
                items,
                attributes,
            })
        })
}

fn trait_body() -> impl NoirParser<Vec<TraitItem>> {
    trait_function_declaration()
        .or(trait_type_declaration())
        .or(trait_constant_declaration())
        .repeated()
}

fn optional_default_value() -> impl NoirParser<Option<Expression>> {
    ignore_then_commit(just(Token::Assign), expression()).or_not()
}

fn trait_constant_declaration() -> impl NoirParser<TraitItem> {
    keyword(Keyword::Let)
        .ignore_then(ident())
        .then_ignore(just(Token::Colon))
        .then(parse_type())
        .then(optional_default_value())
        .then_ignore(just(Token::Semicolon))
        .validate(|((name, typ), default_value), span, emit| {
            emit(ParserError::with_reason(
                ParserErrorReason::ExperimentalFeature("Associated constants"),
                span,
            ));
            TraitItem::Constant { name, typ, default_value }
        })
}

/// trait_function_declaration: 'fn' ident generics '(' declaration_parameters ')' function_return_type
fn trait_function_declaration() -> impl NoirParser<TraitItem> {
    let trait_function_body_or_semicolon =
        block(fresh_statement()).map(Option::from).or(just(Token::Semicolon).to(Option::None));

    keyword(Keyword::Fn)
        .ignore_then(ident())
        .then(function::generics())
        .then(parenthesized(function_declaration_parameters()))
        .then(function_return_type().map(|(_, typ)| typ))
        .then(where_clause())
        .then(trait_function_body_or_semicolon)
        .map(|(((((name, generics), parameters), return_type), where_clause), body)| {
            TraitItem::Function { name, generics, parameters, return_type, where_clause, body }
        })
}

/// trait_type_declaration: 'type' ident generics
fn trait_type_declaration() -> impl NoirParser<TraitItem> {
    keyword(Keyword::Type).ignore_then(ident()).then_ignore(just(Token::Semicolon)).validate(
        |name, span, emit| {
            emit(ParserError::with_reason(
                ParserErrorReason::ExperimentalFeature("Associated types"),
                span,
            ));
            TraitItem::Type { name }
        },
    )
}

/// Parses a trait implementation, implementing a particular trait for a type.
/// This has a similar syntax to `implementation`, but the `for type` clause is required,
/// and an optional `where` clause is also useable.
///
/// trait_implementation: 'impl' generics ident generic_args for type '{' trait_implementation_body '}'
pub(super) fn trait_implementation() -> impl NoirParser<TopLevelStatement> {
    maybe_comp_time()
        .then_ignore(keyword(Keyword::Impl))
        .then(function::generics())
        .then(path())
        .then(generic_type_args(parse_type()))
        .then_ignore(keyword(Keyword::For))
        .then(parse_type())
        .then(where_clause())
        .then_ignore(just(Token::LeftBrace))
        .then(trait_implementation_body())
        .then_ignore(just(Token::RightBrace))
        .map(|args| {
            let (((other_args, object_type), where_clause), items) = args;
            let (((is_comptime, impl_generics), trait_name), trait_generics) = other_args;

            TopLevelStatement::TraitImpl(NoirTraitImpl {
                impl_generics,
                trait_name,
                trait_generics,
                object_type,
                items,
                where_clause,
                is_comptime,
            })
        })
}

fn trait_implementation_body() -> impl NoirParser<Vec<TraitImplItem>> {
    let function = function::function_definition(true).validate(|mut f, span, emit| {
        if f.def().is_unconstrained || f.def().visibility != ItemVisibility::Private {
            emit(ParserError::with_reason(ParserErrorReason::TraitImplFunctionModifiers, span));
        }
        // Trait impl functions are always public
        f.def_mut().visibility = ItemVisibility::Public;
        TraitImplItem::Function(f)
    });

    let alias = keyword(Keyword::Type)
        .ignore_then(ident())
        .then_ignore(just(Token::Assign))
        .then(parse_type())
        .then_ignore(just(Token::Semicolon))
        .map(|(name, alias)| TraitImplItem::Type { name, alias });

    function.or(alias).repeated()
}

fn where_clause() -> impl NoirParser<Vec<UnresolvedTraitConstraint>> {
    struct MultiTraitConstraint {
        typ: UnresolvedType,
        trait_bounds: Vec<TraitBound>,
    }

    let constraints = parse_type()
        .then_ignore(just(Token::Colon))
        .then(trait_bounds())
        .map(|(typ, trait_bounds)| MultiTraitConstraint { typ, trait_bounds });

    keyword(Keyword::Where)
        .ignore_then(constraints.separated_by(just(Token::Comma)))
        .or_not()
        .map(|option| option.unwrap_or_default())
        .map(|x: Vec<MultiTraitConstraint>| {
            let mut result: Vec<UnresolvedTraitConstraint> = Vec::new();
            for constraint in x {
                for bound in constraint.trait_bounds {
                    result.push(UnresolvedTraitConstraint {
                        typ: constraint.typ.clone(),
                        trait_bound: bound,
                    });
                }
            }
            result
        })
}

fn trait_bounds() -> impl NoirParser<Vec<TraitBound>> {
    trait_bound().separated_by(just(Token::Plus)).at_least(1).allow_trailing()
}

fn trait_bound() -> impl NoirParser<TraitBound> {
    path().then(generic_type_args(parse_type())).map(|(trait_path, trait_generics)| TraitBound {
        trait_path,
        trait_generics,
        trait_id: None,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::parser::test_helpers::*;

    #[test]
    fn parse_trait() {
        parse_all(
            trait_definition(),
            vec![
                // Empty traits are legal in Rust and sometimes used as a way to whitelist certain types
                // for a particular operation. Also known as `tag` or `marker` traits:
                // https://stackoverflow.com/questions/71895489/what-is-the-purpose-of-defining-empty-impl-in-rust
                "trait Empty {}",
                "trait TraitWithDefaultBody { fn foo(self) {} }",
                "trait TraitAcceptingMutableRef { fn foo(&mut self); }",
                "trait TraitWithTypeBoundOperation { fn identity() -> Self; }",
                "trait TraitWithAssociatedType { type Element; fn item(self, index: Field) -> Self::Element; }",
                "trait TraitWithAssociatedConstant { let Size: Field; }",
                "trait TraitWithAssociatedConstantWithDefaultValue { let Size: Field = 10; }",
                "trait GenericTrait<T> { fn elem(&mut self, index: Field) -> T; }",
                "trait GenericTraitWithConstraints<T> where T: SomeTrait { fn elem(self, index: Field) -> T; }",
                "trait TraitWithMultipleGenericParams<A, B, C> where A: SomeTrait, B: AnotherTrait<C> { let Size: Field; fn zero() -> Self; }",
            ],
        );

        parse_all_failing(
            trait_definition(),
            vec!["trait MissingBody", "trait WrongDelimiter { fn foo() -> u8, fn bar() -> u8 }"],
        );
    }
}
