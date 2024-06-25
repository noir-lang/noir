use chumsky::Parser;
use noirc_errors::Span;

use crate::{
    macros_api::SecondaryAttribute,
    parser::{NoirParser, ParserError, ParserErrorReason},
    token::{Attribute, Attributes, Token, TokenKind},
};

use super::primitives::token_kind;

fn attribute() -> impl NoirParser<Attribute> {
    token_kind(TokenKind::Attribute).map(|token| match token {
        Token::Attribute(attribute) => attribute,
        _ => unreachable!("Parser should have already errored due to token not being an attribute"),
    })
}

pub(super) fn attributes() -> impl NoirParser<Vec<Attribute>> {
    attribute().repeated()
}

pub(super) fn validate_attributes(
    attributes: Vec<Attribute>,
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
            Attribute::Secondary(attr) => secondary.push(attr),
        }
    }

    Attributes { function: primary, secondary }
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
