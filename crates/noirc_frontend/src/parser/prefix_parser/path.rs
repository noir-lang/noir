use crate::{Path, PathKind};

use super::*;
pub struct PathParser;

impl PathParser {
    /// Parses a Path
    ///
    /// std::hash
    /// std
    /// core::foo::bar
    ///
    /// Cursor Start : `FIRST_PATH_SEGMENT`
    ///
    /// Cursor End : `LAST_PATH_SEGMENT`
    pub fn parse(parser: &mut Parser) -> ParserExprKindResult {
        let mut parsed_path = Vec::new();

        // Parse the first path segment as a keyword or an identifier
        let span_first_segment = parser.curr_token.to_span();
        let (ident, path_kind) = path_identifer(&parser.curr_token)?;
        if let PathKind::Plain = path_kind {
            parsed_path.push(ident.expect("plain paths should contain their identifiers"));
        }

        while parser.peek_token == Token::DoubleColon {
            // Current token is `IDENT`
            //
            // Bump Cursor.
            parser.advance_tokens();
            // Current token is `::` (peeked)
            //
            // Bump Cursor.
            parser.advance_tokens();

            // If the path is valid, we should now be on an Identifier token
            let segment = PrefixParser::Name.parse(parser)?.into_ident().unwrap();
            parsed_path.push(segment);
        }

        // This only happens in cases such as `use dep;` or `use crate`
        if parsed_path.is_empty() {
            return Err(ParserErrorKind::SingleKeywordSegmentNotAllowed {
                span: span_first_segment,
                path_kind,
            });
        }

        Ok(ExpressionKind::Path(Path {
            segments: parsed_path,
            kind: path_kind,
        }))
    }
}

/// Checks the token and returns the identifier along with the path kind
/// only plain paths return identifiers, as the other path kinds implicitly
/// contain the keyword used.
fn path_identifer(
    tok: &crate::token::SpannedToken,
) -> Result<(Option<Ident>, PathKind), ParserErrorKind> {
    use noirc_errors::Spanned;
    match tok.token() {
        Token::Ident(x) => Ok((
            Some(Spanned::from(tok.to_span(), x.to_owned()).into()),
            PathKind::Plain,
        )),
        Token::Keyword(Keyword::Crate) => Ok((None, PathKind::Crate)),
        Token::Keyword(Keyword::Dep) => Ok((None, PathKind::Dep)),
        _ => {
            return Err(ParserErrorKind::UnstructuredError {
                span: tok.to_span(),
                message: format!(
                    "expected an identifier, `dep` or `crate`. found {} ",
                    tok.token().to_string()
                ),
            })
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{parser::test_parse, ExpressionKind, Path, PathKind};

    use super::PathParser;

    fn expr_to_path(expr: ExpressionKind) -> Path {
        match expr {
            ExpressionKind::Path(pth) => pth,
            _ => unreachable!(),
        }
    }

    #[test]
    fn valid_syntax() {
        let vectors = vec![
            ("std", vec!["std"]),
            ("std::hash", vec!["std", "hash"]),
            ("std::hash::collections", vec!["std", "hash", "collections"]),
            ("crate::std::hash", vec!["std", "hash"]),
        ];

        for (src, expected_seg) in vectors {
            let expr = PathParser::parse(&mut test_parse(src)).unwrap();
            let path = expr_to_path(expr);
            for (got, expected) in path.segments.into_iter().zip(expected_seg) {
                assert_eq!(&got.0.contents, expected)
            }
        }
    }
    #[test]
    fn valid_path_kinds() {
        let vectors = vec![
            ("std", PathKind::Plain),
            ("dep::hash::collections", PathKind::Dep),
            ("crate::std::hash", PathKind::Crate),
        ];

        for (src, expected_path_kind) in vectors {
            let expr = PathParser::parse(&mut test_parse(src)).unwrap();
            let path = expr_to_path(expr);
            assert_eq!(path.kind, expected_path_kind)
        }
    }
    #[test]
    fn invalid_path_kinds() {
        let vectors = vec![
            "dep",
            "crate",
            "crate::std::crate",
            "foo::bar::crate",
            "foo::dep",
        ];

        for path in vectors {
            assert!(PathParser::parse(&mut test_parse(path)).is_err());
        }
    }
    #[test]
    fn invalid_syntax() {
        let vectors = vec!["std::", "::std", "std::hash::", "foo::1::"];

        for src in vectors {
            PathParser::parse(&mut test_parse(src)).unwrap_err();
        }
    }
}
