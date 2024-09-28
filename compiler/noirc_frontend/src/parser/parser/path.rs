use noirc_errors::Span;

use crate::{
    ast::{AsTraitPath, Ident, Path, PathKind, PathSegment, UnresolvedType},
    parser::ParserErrorReason,
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

        self.parse_path_after_kind(kind, allow_turbofish, start_span)
    }

    pub(super) fn parse_path_after_kind(
        &mut self,
        kind: PathKind,
        allow_turbofish: bool,
        start_span: Span,
    ) -> (Path, bool) {
        let mut trailing_double_colon = false;
        let mut segments = Vec::new();

        if self.token.kind() == TokenKind::Ident {
            while let Some(ident) = self.eat_ident() {
                let span = ident.span();

                let mut has_double_colon = self.eat_double_colon();

                let generics = if has_double_colon && allow_turbofish {
                    if let Some(generics) = self.parse_path_generics() {
                        has_double_colon = self.eat_double_colon();
                        Some(generics)
                    } else {
                        None
                    }
                } else {
                    None
                };

                segments.push(PathSegment { ident, generics, span });

                if has_double_colon {
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

    pub(super) fn parse_path_no_turbofish_after_ident(&mut self, ident: Ident) -> Path {
        let start_span = ident.span();
        let mut segments = vec![PathSegment::from(ident)];

        while self.eat_double_colon() {
            if let Some(ident) = self.eat_ident() {
                segments.push(PathSegment::from(ident));
            } else {
                // TODO: error (trailing double colon in path)
                break;
            }
        }

        Path { segments, kind: PathKind::Plain, span: self.span_since(start_span) }
    }

    pub(super) fn parse_path_generics(&mut self) -> Option<Vec<UnresolvedType>> {
        if !self.eat_less() {
            return None;
        }

        let mut generics = Vec::new();
        let mut trailing_comma = false;

        if self.eat_greater() {
            // TODO: error
        } else {
            loop {
                let star_span = self.current_token_span;
                let Some(typ) = self.parse_type() else {
                    self.eat_greater();
                    break;
                };

                if !trailing_comma && !generics.is_empty() {
                    self.push_error(ParserErrorReason::MissingCommaSeparatingGenerics, star_span);
                }

                generics.push(typ);
                trailing_comma = self.eat_commas();

                if self.eat_greater() {
                    break;
                }
            }
        }
        Some(generics)
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

    pub(super) fn parse_as_trait_path(&mut self) -> Option<AsTraitPath> {
        if !self.eat_less() {
            return None;
        }

        let typ = self.parse_type_or_error();
        if !self.eat_keyword(Keyword::As) {
            // TODO: error (expected `as`)
        }
        let trait_path = self.parse_path_no_turbofish();
        let trait_generics = self.parse_generic_type_args();
        if !self.eat_greater() {
            // TODO: error (expected `>`)
        }
        if !self.eat_double_colon() {
            // TODO: error (expected `::`)
        }
        let impl_item = if let Some(ident) = self.eat_ident() {
            ident
        } else {
            // TODO: error (expected identifier)
            Ident::new(String::new(), self.span_at_previous_token_end())
        };

        Some(AsTraitPath { typ, trait_path, trait_generics, impl_item })
    }
}

#[cfg(test)]
mod tests {

    use crate::{ast::PathKind, parser::Parser};

    #[test]
    fn parses_plain_one_segment() {
        let src = "foo";
        let mut parser = Parser::for_str(src);
        let path = parser.parse_path();
        assert!(parser.errors.is_empty());
        assert_eq!(path.kind, PathKind::Plain);
        assert_eq!(path.segments.len(), 1);
        assert_eq!(path.segments[0].ident.to_string(), "foo");
        assert!(path.segments[0].generics.is_none());
    }

    #[test]
    fn parses_plain_two_segments() {
        let src = "foo::bar";
        let mut parser = Parser::for_str(src);
        let path = parser.parse_path();
        assert!(parser.errors.is_empty());
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
        let mut parser = Parser::for_str(src);
        let path = parser.parse_path();
        assert!(parser.errors.is_empty());
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
        let mut parser = Parser::for_str(src);
        let path = parser.parse_path();
        assert!(parser.errors.is_empty());
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
        let mut parser = Parser::for_str(src);
        let path = parser.parse_path();
        assert!(parser.errors.is_empty());
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

    #[test]
    fn parses_with_turbofish() {
        let src = "foo::<T, i32>::bar";
        let mut parser = Parser::for_str(src);
        let mut path = parser.parse_path();
        assert!(parser.errors.is_empty());
        assert_eq!(path.kind, PathKind::Plain);
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[0].ident.to_string(), "foo");

        let generics = path.segments.remove(0).generics;
        assert_eq!(generics.unwrap().len(), 2);

        let generics = path.segments.remove(0).generics;
        assert!(generics.is_none());
    }
}
