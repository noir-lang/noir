use crate::ast::{Path, PathKind};
use crate::parser::NoirParser;

use crate::token::{Keyword, Token};

use chumsky::prelude::*;

use super::{ident, keyword};

pub(super) fn path() -> impl NoirParser<Path> {
    let idents = || ident().separated_by(just(Token::DoubleColon)).at_least(1);
    let make_path = |kind| move |segments, span| Path { segments, kind, span };

    let prefix = |key| keyword(key).ignore_then(just(Token::DoubleColon));
    let path_kind = |key, kind| prefix(key).ignore_then(idents()).map_with_span(make_path(kind));

    choice((
        path_kind(Keyword::Crate, PathKind::Crate),
        path_kind(Keyword::Dep, PathKind::Dep),
        path_kind(Keyword::Super, PathKind::Super),
        idents().map_with_span(make_path(PathKind::Plain)),
    ))
}

fn empty_path() -> impl NoirParser<Path> {
    let make_path = |kind| move |_, span| Path { segments: Vec::new(), kind, span };
    let path_kind = |key, kind| keyword(key).map_with_span(make_path(kind));

    choice((path_kind(Keyword::Crate, PathKind::Crate), path_kind(Keyword::Dep, PathKind::Plain)))
}

pub(super) fn maybe_empty_path() -> impl NoirParser<Path> {
    path().or(empty_path())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::parser::test_helpers::{parse_all_failing, parse_with};

    #[test]
    fn parse_path() {
        let cases = vec![
            ("std", vec!["std"]),
            ("std::hash", vec!["std", "hash"]),
            ("std::hash::collections", vec!["std", "hash", "collections"]),
            ("foo::bar", vec!["foo", "bar"]),
            ("foo::bar", vec!["foo", "bar"]),
            ("crate::std::hash", vec!["std", "hash"]),
        ];

        for (src, expected_segments) in cases {
            let path: Path = parse_with(path(), src).unwrap();
            for (segment, expected) in path.segments.into_iter().zip(expected_segments) {
                assert_eq!(segment.0.contents, expected);
            }
        }

        parse_all_failing(path(), vec!["std::", "::std", "std::hash::", "foo::1"]);
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
            let path = parse_with(path(), src).unwrap();
            assert_eq!(path.kind, expected_path_kind);
        }

        parse_all_failing(
            path(),
            vec!["crate", "crate::std::crate", "foo::bar::crate", "foo::dep"],
        );
    }
}
