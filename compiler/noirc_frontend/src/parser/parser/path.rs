use crate::ast::{AsTraitPath, Path, PathKind, PathSegment, UnresolvedType};
use crate::parser::{NoirParser, ParserError, ParserErrorReason};

use crate::token::{Keyword, Token};

use chumsky::prelude::*;

use super::keyword;
use super::primitives::{ident, path_segment, path_segment_no_turbofish};
use super::types::generic_type_args;

pub(super) fn path<'a>(
    type_parser: impl NoirParser<UnresolvedType> + 'a,
) -> impl NoirParser<Path> + 'a {
    path_inner(path_segment(type_parser))
}

pub fn path_no_turbofish() -> impl NoirParser<Path> {
    path_inner(path_segment_no_turbofish())
}

fn path_inner<'a>(segment: impl NoirParser<PathSegment> + 'a) -> impl NoirParser<Path> + 'a {
    let segments = segment
        .separated_by(just(Token::DoubleColon))
        .at_least(1)
        .then(just(Token::DoubleColon).then_ignore(none_of(Token::LeftBrace).rewind()).or_not())
        .validate(|(path_segments, trailing_colons), span, emit_error| {
            if trailing_colons.is_some() {
                emit_error(ParserError::with_reason(
                    ParserErrorReason::ExpectedIdentifierAfterColons,
                    span,
                ));
            }
            path_segments
        });
    let make_path = |kind| move |segments, span| Path { segments, kind, span };

    let prefix = |key| keyword(key).ignore_then(just(Token::DoubleColon));
    let path_kind =
        |key, kind| prefix(key).ignore_then(segments.clone()).map_with_span(make_path(kind));

    choice((
        path_kind(Keyword::Crate, PathKind::Crate),
        path_kind(Keyword::Dep, PathKind::Dep),
        path_kind(Keyword::Super, PathKind::Super),
        segments.map_with_span(make_path(PathKind::Plain)),
    ))
}

/// Parses `<MyType as Trait>::path_segment`
/// These paths only support exactly two segments.
pub(super) fn as_trait_path<'a>(
    type_parser: impl NoirParser<UnresolvedType> + 'a,
) -> impl NoirParser<AsTraitPath> + 'a {
    just(Token::Less)
        .ignore_then(type_parser.clone())
        .then_ignore(keyword(Keyword::As))
        .then(path(type_parser.clone()))
        .then(generic_type_args(type_parser))
        .then_ignore(just(Token::Greater))
        .then_ignore(just(Token::DoubleColon))
        .then(ident())
        .map(|(((typ, trait_path), trait_generics), impl_item)| AsTraitPath {
            typ,
            trait_path,
            trait_generics,
            impl_item,
        })
}

fn empty_path() -> impl NoirParser<Path> {
    let make_path = |kind| move |_, span| Path { segments: Vec::new(), kind, span };
    let path_kind = |key, kind| keyword(key).map_with_span(make_path(kind));

    choice((path_kind(Keyword::Crate, PathKind::Crate), path_kind(Keyword::Dep, PathKind::Plain)))
}

pub(super) fn maybe_empty_path() -> impl NoirParser<Path> {
    path_no_turbofish().or(empty_path())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::{
        parse_type,
        parser::test_helpers::{parse_all_failing, parse_recover, parse_with},
    };

    #[test]
    fn parse_path() {
        let cases = vec![
            ("std", vec!["std"]),
            ("std::hash", vec!["std", "hash"]),
            ("std::hash::collections", vec!["std", "hash", "collections"]),
            ("foo::bar", vec!["foo", "bar"]),
            ("crate::std::hash", vec!["std", "hash"]),
        ];

        for (src, expected_segments) in cases {
            let path: Path = parse_with(path(parse_type()), src).unwrap();
            for (segment, expected) in path.segments.into_iter().zip(expected_segments) {
                assert_eq!(segment.ident.0.contents, expected);
            }
        }

        parse_all_failing(path(parse_type()), vec!["std::", "::std", "std::hash::", "foo::1"]);
    }

    #[test]
    fn parse_path_kinds() {
        let cases = vec![
            ("std", PathKind::Plain),
            ("hash::collections", PathKind::Plain),
            ("crate::std::hash", PathKind::Crate),
            ("super::foo", PathKind::Super),
        ];

        for (src, expected_path_kind) in cases {
            let path = parse_with(path(parse_type()), src).unwrap();
            assert_eq!(path.kind, expected_path_kind);
        }

        parse_all_failing(
            path(parse_type()),
            vec!["crate", "crate::std::crate", "foo::bar::crate", "foo::dep"],
        );
    }

    #[test]
    fn parse_path_with_trailing_colons() {
        let src = "foo::bar::";

        let (path, errors) = parse_recover(path_no_turbofish(), src);
        let path = path.unwrap();
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[0].ident.0.contents, "foo");
        assert_eq!(path.segments[1].ident.0.contents, "bar");

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "expected an identifier after ::");
    }
}
