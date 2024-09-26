use noirc_errors::Span;

use crate::{
    ast::{Path, PathKind, PathSegment},
    token::{Keyword, TokenKind},
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_path(&mut self) -> Path {
        let (path, trailing_double_colon) = self.parse_path_impl(true);
        if trailing_double_colon {
            // TODO: error
        }
        path
    }

    pub(crate) fn parse_path_no_turbofish(&mut self) -> Path {
        let (path, trailing_double_colon) = self.parse_path_impl(false);
        if trailing_double_colon {
            // TODO: error
        }
        path
    }

    pub(super) fn parse_path_impl(&mut self, allow_turbofish: bool) -> (Path, bool) {
        let start_span = self.current_token_span;

        let kind = self.parse_path_kind();
        if kind != PathKind::Plain && !self.eat_double_colon() {
            // TODO: error
        }

        self.parse_path_after_kind(kind, start_span)
    }

    pub(super) fn parse_path_after_kind(
        &mut self,
        kind: PathKind,
        start_span: Span,
    ) -> (Path, bool) {
        let mut trailing_double_colon = false;
        let mut segments = Vec::new();

        if self.token.kind() == TokenKind::Ident {
            while let Some(ident) = self.eat_ident() {
                let span = ident.span();
                segments.push(PathSegment { ident, generics: None, span });
                if self.eat_double_colon() {
                    trailing_double_colon = true;
                } else {
                    trailing_double_colon = false;
                    break;
                }
            }
        } else {
            // TODO: error
        }

        let span = self.span_since(start_span);

        (Path { segments, kind, span }, trailing_double_colon)
    }

    pub(super) fn parse_path_kind(&mut self) -> PathKind {
        if self.eat_keyword(Keyword::Crate) {
            PathKind::Crate
        } else if self.eat_keyword(Keyword::Dep) {
            PathKind::Dep
        } else if self.eat_keyword(Keyword::Super) {
            PathKind::Super
        } else {
            PathKind::Plain
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{ast::PathKind, parser::Parser};

    #[test]
    fn parses_plain_one_segment() {
        let src = "foo";
        let path = Parser::for_str(src).parse_path();
        assert_eq!(path.kind, PathKind::Plain);
        assert_eq!(path.segments.len(), 1);
        assert_eq!(path.segments[0].ident.to_string(), "foo");
        assert!(path.segments[0].generics.is_none());
    }

    #[test]
    fn parses_plain_two_segments() {
        let src = "foo::bar";
        let path = Parser::for_str(src).parse_path();
        assert_eq!(path.kind, PathKind::Plain);
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[0].ident.to_string(), "foo");
        assert!(path.segments[0].generics.is_none());
        assert_eq!(path.segments[1].ident.to_string(), "bar");
        assert!(path.segments[1].generics.is_none());
    }

    #[test]
    fn parses_crate_two_segments() {
        let src = "crate::foo::bar";
        let path = Parser::for_str(src).parse_path();
        assert_eq!(path.kind, PathKind::Crate);
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[0].ident.to_string(), "foo");
        assert!(path.segments[0].generics.is_none());
        assert_eq!(path.segments[1].ident.to_string(), "bar");
        assert!(path.segments[1].generics.is_none());
    }

    #[test]
    fn parses_super_two_segments() {
        let src = "super::foo::bar";
        let path = Parser::for_str(src).parse_path();
        assert_eq!(path.kind, PathKind::Super);
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[0].ident.to_string(), "foo");
        assert!(path.segments[0].generics.is_none());
        assert_eq!(path.segments[1].ident.to_string(), "bar");
        assert!(path.segments[1].generics.is_none());
    }

    #[test]
    fn parses_dep_two_segments() {
        let src = "dep::foo::bar";
        let path = Parser::for_str(src).parse_path();
        assert_eq!(path.kind, PathKind::Dep);
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[0].ident.to_string(), "foo");
        assert!(path.segments[0].generics.is_none());
        assert_eq!(path.segments[1].ident.to_string(), "bar");
        assert!(path.segments[1].generics.is_none());
    }

    #[test]
    fn parses_plain_one_segment_with_trailing_colons() {
        let src = "foo::";
        let mut parser = Parser::for_str(src);
        let path = parser.parse_path();
        assert_eq!(parser.errors.len(), 0); // TODO: this should be 1
        assert_eq!(path.kind, PathKind::Plain);
        assert_eq!(path.segments.len(), 1);
        assert_eq!(path.segments[0].ident.to_string(), "foo");
        assert!(path.segments[0].generics.is_none());
        assert_eq!(path.span.end() as usize, src.len());
    }
}
