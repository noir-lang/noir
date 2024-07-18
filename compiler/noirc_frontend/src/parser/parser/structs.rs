use chumsky::prelude::*;

use crate::ast::{Ident, NoirStruct, UnresolvedType};
use crate::parser::parser::types::maybe_comp_time;
use crate::{
    parser::{
        parser::{
            attributes::{attributes, validate_secondary_attributes},
            function, parse_type,
            primitives::{ident, keyword},
        },
        NoirParser, TopLevelStatement,
    },
    token::{Keyword, Token},
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
        .then(maybe_comp_time())
        .then_ignore(keyword(Struct))
        .then(ident())
        .then(function::generics())
        .then(fields)
        .validate(|((((attributes, is_comptime), name), generics), fields), span, emit| {
            let attributes = validate_secondary_attributes(attributes, span, emit);
            TopLevelStatement::Struct(NoirStruct {
                name,
                attributes,
                generics,
                fields,
                span,
                is_comptime,
            })
        })
}

fn struct_fields() -> impl NoirParser<Vec<(Ident, UnresolvedType)>> {
    ident()
        .then_ignore(just(Token::Colon))
        .then(parse_type())
        .separated_by(just(Token::Comma))
        .allow_trailing()
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
