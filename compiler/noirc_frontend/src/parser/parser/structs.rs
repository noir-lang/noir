use chumsky::prelude::*;

use crate::ast::{Documented, NoirStruct, StructField};
use crate::{
    parser::{
        parser::{
            attributes::{attributes, validate_secondary_attributes},
            function, parse_type,
            primitives::{ident, keyword},
        },
        NoirParser, TopLevelStatementKind,
    },
    token::{Keyword, Token},
};

use super::doc_comments::outer_doc_comments;

pub(super) fn struct_definition() -> impl NoirParser<TopLevelStatementKind> {
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
        .validate(|(((attributes, name), generics), fields), span, emit| {
            let attributes = validate_secondary_attributes(attributes, span, emit);
            TopLevelStatementKind::Struct(NoirStruct { name, attributes, generics, fields, span })
        })
}

fn struct_fields() -> impl NoirParser<Vec<Documented<StructField>>> {
    let field = ident().then_ignore(just(Token::Colon)).then(parse_type());
    let field = outer_doc_comments().then(field).map(|(doc_comments, (name, typ))| {
        Documented::new(StructField { name, typ }, doc_comments)
    });
    field.separated_by(just(Token::Comma)).allow_trailing()
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
