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

        // Parse the first path segment
        let segment = PrefixParser::Name.parse(parser)?.into_ident().unwrap();
        parsed_path.push(segment);

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

        let path_kind = path_kind(
            parsed_path
                .first()
                .expect("ice: this function triggers when there is at least one ident"),
        );

        Ok(ExpressionKind::Path(Path {
            segments: parsed_path,
            kind: path_kind,
        }))
    }
}

fn path_kind(ident: &Ident) -> PathKind {
    let contents = &ident.0.contents;
    // XXX: modify the lexer and parser to have multiple conditions
    // for peek_check_kind_advance so we cna check for keyword and Ident
    // then make "crate" a keyword
    // Still undecided about "dep"
    // We could just use double colon like rust and have absolute paths
    if contents == "crate" {
        PathKind::Crate
    } else if contents == "dep" {
        PathKind::Dep
    } else {
        PathKind::Plain
    }
}

#[cfg(test)]
mod test {
    use crate::{parser::test_parse, ExpressionKind, Path};

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
            ("crate::std::hash", vec!["crate", "std", "hash"]),
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
    fn invalid_syntax() {
        let vectors = vec!["std::", "::std", "std::hash::", "foo::1::"];

        for src in vectors {
            PathParser::parse(&mut test_parse(src)).unwrap_err();
        }
    }
}
