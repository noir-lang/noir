use chumsky::prelude::*;
use noirc_errors::Span;

use crate::{
    macros_api::SecondaryAttribute,
    parser::{
        parser::{
            attributes::attributes,
            function, parse_type,
            primitives::{ident, keyword},
        },
        NoirParser, ParserError, ParserErrorReason, TopLevelStatement,
    },
    token::{Attribute, Keyword, Token},
    Ident, NoirStruct, UnresolvedType,
};

pub(super) fn struct_definition() -> impl NoirParser<TopLevelStatement> {
    use self::Keyword::Struct;
    use Token::*;

    let fields = struct_fields()
        .delimited_by(just(LeftBrace), just(RightBrace))
        .recover_with(nested_delimiters(
            LeftBrace,
            RightBrace,
            [(LeftParen, RightParen), (LeftBracket, RightBracket)],
            |_| vec![],
        ))
        .or(just(Semicolon).to(Vec::new()));

    attributes()
        .then_ignore(keyword(Struct))
        .then(ident())
        .then(function::generics())
        .then(fields)
        .validate(|(((raw_attributes, name), generics), fields), span, emit| {
            let attributes = validate_struct_attributes(raw_attributes, span, emit);
            TopLevelStatement::Struct(NoirStruct { name, attributes, generics, fields, span })
        })
}

fn struct_fields() -> impl NoirParser<Vec<(Ident, UnresolvedType)>> {
    ident()
        .then_ignore(just(Token::Colon))
        .then(parse_type())
        .separated_by(just(Token::Comma))
        .allow_trailing()
}

fn validate_struct_attributes(
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::parser::test_helpers::*;

    #[test]
    fn parse_structs() {
        let cases = vec![
            "struct Foo;",
            "struct Foo { }",
            "struct Bar { ident: Field, }",
            "struct Baz { ident: Field, other: Field }",
            "#[attribute] struct Baz { ident: Field, other: Field }",
        ];
        parse_all(struct_definition(), cases);

        let failing = vec![
            "struct {  }",
            "struct Foo { bar: pub Field }",
            "struct Foo { bar: pub Field }",
            "#[oracle(some)] struct Foo { bar: Field }",
        ];
        parse_all_failing(struct_definition(), failing);
    }
}
