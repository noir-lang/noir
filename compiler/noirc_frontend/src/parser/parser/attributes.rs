use chumsky::{prelude::just, Parser};
use noirc_errors::Span;

use crate::{
    macros_api::SecondaryAttribute,
    parser::{parenthesized, NoirParser, ParserError, ParserErrorReason},
    token::{Attribute, Attributes, CustomAtrribute, FormalVerificationAttribute, Token, TokenKind},
};

use super::{expression, primitives::token_kind};

fn attribute() -> impl NoirParser<Attribute> {
    token_kind(TokenKind::Attribute).map(|token| match token {
        Token::Attribute(attribute) => attribute,
        _ => unreachable!("Parser should have already errored due to token not being an attribute"),
    })
}

fn fv_attribute() -> impl NoirParser<FormalVerificationAttribute> {
    token_kind(TokenKind::Token(Token::Ensures))
        .or(token_kind(TokenKind::Token(Token::Requires)))
        .then(
            parenthesized(expression())
        )
        .then_ignore(just(Token::RightBracket))
        .map(|(token, expr)| match token {
            Token::Requires => FormalVerificationAttribute::Requires(expr),
            Token::Ensures => FormalVerificationAttribute::Ensures(expr),
            _ => unreachable!(
                "Parser should have already errored due to token not being an attribute"
            ),
        })
}

/// Represents any attribute that is accepted by the language.
/// This includes normal upstream attributes and formal verification attributes.
pub(super) enum AnyAttribute { 
    FvAttribute(FormalVerificationAttribute),
    NormalAttribute(Attribute),
}

pub(super) fn all_attributes() -> impl NoirParser<Vec<AnyAttribute>> {
    fv_attribute().map(|x| AnyAttribute::FvAttribute(x))
    .or(attribute().map(|x| AnyAttribute::NormalAttribute(x)))
    .repeated()
}

pub(super) fn split_attributes_in_two(
    all_attributes: Vec<AnyAttribute>
) -> (Vec<FormalVerificationAttribute>, Vec<Attribute>) {
    let mut fv_attributes: Vec<FormalVerificationAttribute> = Vec::new();
    let mut attributes: Vec<Attribute> = Vec::new();

    all_attributes.into_iter().for_each(|attr| {
        match attr {
            AnyAttribute::FvAttribute(fv_attr) => fv_attributes.push(fv_attr),
            AnyAttribute::NormalAttribute(normal_attr) => attributes.push(normal_attr),
        }
    });

    (fv_attributes, attributes)
}

pub(super) fn attributes() -> impl NoirParser<Vec<Attribute>> {
    attribute().repeated()
}

fn is_valid_custom_attribute(custom_attr: CustomAtrribute) -> bool {
    !(custom_attr.contents.starts_with("ensures") || custom_attr.contents.starts_with("requires"))
}

pub(super) fn validate_attributes(
    attributes: Vec<Attribute>,
    fv_attributes: Vec<FormalVerificationAttribute>,
    span: Span,
    emit: &mut dyn FnMut(ParserError),
) -> Attributes {
    let mut primary = None;
    let mut secondary = Vec::new();

    for attribute in attributes {
        match attribute {
            Attribute::Function(attr) => {
                if primary.is_some() {
                    emit(ParserError::with_reason(
                        ParserErrorReason::MultipleFunctionAttributesFound,
                        span,
                    ));
                }
                primary = Some(attr);
            }
            Attribute::Secondary(attr) => {
                match attr.clone() {
                    SecondaryAttribute::Custom(custom_attr) => {
                        if is_valid_custom_attribute(custom_attr) {
                            secondary.push(attr)   
                        } else {
                            emit(ParserError::with_reason(
                                ParserErrorReason::ReservedAttributeName,
                                span,
                            ));
                        }
                    },
                    _ => secondary.push(attr)
                }
            },
        }
    }

    Attributes { function: primary, secondary, fv_attributes}
}

pub(super) fn validate_secondary_attributes(
    attributes: Vec<Attribute>,
    span: Span,
    emit: &mut dyn FnMut(ParserError),
) -> Vec<SecondaryAttribute> {
    let mut struct_attributes = vec![];

    for attribute in attributes {
        match attribute {
            Attribute::Function(..) => {
                emit(ParserError::with_reason(
                    ParserErrorReason::NoFunctionAttributesAllowedOnStruct,
                    span,
                ));
            }
            Attribute::Secondary(attr) => struct_attributes.push(attr),
        }
    }

    struct_attributes
}

pub(super) fn inner_attribute() -> impl NoirParser<SecondaryAttribute> {
    token_kind(TokenKind::InnerAttribute).map(|token| match token {
        Token::InnerAttribute(attribute) => attribute,
        _ => unreachable!(
            "Parser should have already errored due to token not being an inner attribute"
        ),
    })
}
