use crate::ast::{AsTraitPath, Ident, Path, PathKind, PathSegment, UnresolvedType};
use crate::parser::ParserErrorReason;

use crate::token::{Keyword, Token};

use noirc_errors::Location;

use crate::{parser::labels::ParsingRuleLabel, token::TokenKind};

use super::Parser;

impl Parser<'_> {
    #[cfg(test)]
    pub(crate) fn parse_path_or_error(&mut self) -> Path {
        if let Some(path) = self.parse_path() {
            path
        } else {
            self.expected_label(ParsingRuleLabel::Path);

            Path::plain(Vec::new(), self.location_at_previous_token_end())
        }
    }

    /// Tries to parse a Path.
    /// Note that `crate::`, `super::`, etc., are not valid paths on their own.
    ///
    /// Path = PathKind identifier Turbofish? ( '::' identifier Turbofish? )*
    ///
    /// Turbofish = '::' PathGenerics
    pub(crate) fn parse_path(&mut self) -> Option<Path> {
        self.parse_path_impl(
            true, // allow turbofish
            true, // allow trailing double colon
        )
    }

    pub(crate) fn parse_path_no_turbofish_or_error(&mut self) -> Path {
        if let Some(path) = self.parse_path_no_turbofish() {
            path
        } else {
            self.expected_label(ParsingRuleLabel::Path);

            Path::plain(Vec::new(), self.location_at_previous_token_end())
        }
    }

    /// PathNoTurbofish = PathKind identifier ( '::' identifier )*
    pub fn parse_path_no_turbofish(&mut self) -> Option<Path> {
        self.parse_path_impl(
            false, // allow turbofish
            true,  // allow trailing double colon
        )
    }

    pub(super) fn parse_path_for_named_type(&mut self) -> Option<Path> {
        let allow_turbofish = false;
        let allow_trailing_double_colon = false;
        self.parse_path_impl(allow_turbofish, allow_trailing_double_colon)
    }

    pub(super) fn parse_path_impl(
        &mut self,
        allow_turbofish: bool,
        allow_trailing_double_colon: bool,
    ) -> Option<Path> {
        let start_location = self.current_token_location;

        let kind = self.parse_path_kind();

        let path = self.parse_optional_path_after_kind(
            kind,
            allow_turbofish,
            allow_trailing_double_colon,
            start_location,
        )?;
        if path.segments.is_empty() {
            if path.kind != PathKind::Plain {
                self.expected_identifier();
            }
            None
        } else {
            Some(path)
        }
    }

    pub(super) fn parse_optional_path_after_kind(
        &mut self,
        kind: PathKind,
        allow_turbofish: bool,
        allow_trailing_double_colon: bool,
        start_location: Location,
    ) -> Option<Path> {
        let path = self.parse_path_after_kind(
            kind,
            allow_turbofish,
            allow_trailing_double_colon,
            start_location,
        );

        if path.segments.is_empty() && path.kind == PathKind::Plain { None } else { Some(path) }
    }

    /// Parses a path assuming the path's kind (plain, `crate::`, `super::`, etc.)
    /// was already parsed. Note that this method always returns a Path, even if it
    /// ends up being just `crate::` or an empty path.
    pub(super) fn parse_path_after_kind(
        &mut self,
        kind: PathKind,
        allow_turbofish: bool,
        allow_trailing_double_colon: bool,
        start_location: Location,
    ) -> Path {
        let mut segments = Vec::new();

        if self.token.kind() == TokenKind::Ident {
            loop {
                let ident = self.eat_ident().unwrap();
                let location = ident.location();

                let generics = if allow_turbofish
                    && self.at(Token::DoubleColon)
                    && self.next_is(Token::Less)
                {
                    self.bump();
                    self.parse_path_generics(ParserErrorReason::AssociatedTypesNotAllowedInPaths)
                } else {
                    None
                };

                segments.push(PathSegment {
                    ident,
                    generics,
                    location: self.location_since(location),
                });

                if self.at(Token::DoubleColon)
                    && matches!(self.next_token.token(), Token::Ident(..))
                {
                    // Skip the double colons
                    self.bump();
                } else {
                    if allow_trailing_double_colon && self.eat_double_colon() {
                        self.expected_identifier();
                        break;
                    }

                    break;
                }
            }
        }

        let location = self.location_since(start_location);
        Path { segments, kind, kind_location: start_location, location }
    }

    /// PathGenerics = GenericTypeArgs
    pub(super) fn parse_path_generics(
        &mut self,
        on_named_arg_error: ParserErrorReason,
    ) -> Option<Vec<UnresolvedType>> {
        if self.token.token() != &Token::Less {
            return None;
        }

        let generics = self.parse_generic_type_args();
        for (name, _typ) in &generics.named_args {
            self.push_error(on_named_arg_error.clone(), name.location());
        }

        Some(generics.ordered_args)
    }

    /// PathKind
    ///     = '::'
    ///     | 'dep' '::'
    ///     | 'crate' '::'
    ///     | 'super' '::'
    ///     | nothing
    pub(super) fn parse_path_kind(&mut self) -> PathKind {
        let start_location = self.current_token_location;
        let mut deprecated_dep_found = false;

        let kind = if self.at(Token::DoubleColon) {
            PathKind::Absolute
        } else if self.eat_keyword(Keyword::Dep) {
            deprecated_dep_found = true;
            PathKind::Absolute
        } else if self.eat_keyword(Keyword::Crate) {
            PathKind::Crate
        } else if self.eat_keyword(Keyword::Super) {
            PathKind::Super
        } else if let Token::InternedCrate(crate_id) = self.token.token() {
            let crate_id = *crate_id;
            self.bump();
            PathKind::Resolved(crate_id)
        } else {
            PathKind::Plain
        };
        if kind != PathKind::Plain {
            self.eat_or_error(Token::DoubleColon);
        }

        if deprecated_dep_found {
            // In the error message, try to include the actual dependency being imported
            let dependency_name = if let Token::Ident(name) = self.token.token() {
                name.clone()
            } else {
                "dependency".to_string()
            };
            self.push_error(ParserErrorReason::DeprecatedDep(dependency_name), start_location);
        }

        kind
    }

    /// AsTraitPath = '<' Type 'as' PathNoTurbofish GenericTypeArgs '>' '::' identifier
    pub(super) fn parse_as_trait_path(&mut self) -> Option<AsTraitPath> {
        if !self.eat_less() {
            return None;
        }

        let typ = self.parse_type_or_error();
        self.eat_keyword_or_error(Keyword::As);

        Some(self.parse_as_trait_path_for_type_after_as_keyword(typ))
    }

    pub(super) fn parse_as_trait_path_for_type_after_as_keyword(
        &mut self,
        typ: UnresolvedType,
    ) -> AsTraitPath {
        let trait_path = self.parse_path_no_turbofish_or_error();
        let trait_generics = self.parse_generic_type_args();
        self.eat_or_error(Token::Greater);
        self.eat_or_error(Token::DoubleColon);
        let impl_item = if let Some(ident) = self.eat_non_underscore_ident() {
            ident
        } else {
            self.expected_identifier();
            Ident::new(String::new(), self.location_at_previous_token_end())
        };

        AsTraitPath { typ, trait_path, trait_generics, impl_item }
    }
}

#[cfg(test)]
mod tests {

    use insta::assert_snapshot;

    use crate::{
        ast::{Path, PathKind},
        parser::{
            Parser,
            parser::tests::{expect_no_errors, get_single_error, get_source_with_error_span},
        },
    };

    fn parse_path_no_errors(src: &str) -> Path {
        let mut parser = Parser::for_str_with_dummy_file(src);
        let path = parser.parse_path_or_error();
        expect_no_errors(&parser.errors);
        path
    }

    #[test]
    fn parses_plain_one_segment() {
        let src = "foo";
        let path = parse_path_no_errors(src);
        assert_eq!(path.kind, PathKind::Plain);
        assert_eq!(path.segments.len(), 1);
        assert_eq!(path.segments[0].ident.to_string(), "foo");
        assert!(path.segments[0].generics.is_none());
    }

    #[test]
    fn parses_plain_two_segments() {
        let src = "foo::bar";
        let path = parse_path_no_errors(src);
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
        let path = parse_path_no_errors(src);
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
        let path = parse_path_no_errors(src);
        assert_eq!(path.kind, PathKind::Super);
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[0].ident.to_string(), "foo");
        assert!(path.segments[0].generics.is_none());
        assert_eq!(path.segments[1].ident.to_string(), "bar");
        assert!(path.segments[1].generics.is_none());
    }

    #[test]
    fn parses_absolute_two_segments() {
        let src = "::foo::bar";
        let path = parse_path_no_errors(src);
        assert_eq!(path.kind, PathKind::Absolute);
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[0].ident.to_string(), "foo");
        assert!(path.segments[0].generics.is_none());
        assert_eq!(path.segments[1].ident.to_string(), "bar");
        assert!(path.segments[1].generics.is_none());
    }

    #[test]
    fn parses_plain_one_segment_with_trailing_colons() {
        let src = "foo::";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let path = parser.parse_path_or_error();
        assert_eq!(path.location.span.end() as usize, src.len());
        assert_eq!(parser.errors.len(), 1);
        assert_eq!(path.kind, PathKind::Plain);
        assert_eq!(path.segments.len(), 1);
        assert_eq!(path.segments[0].ident.to_string(), "foo");
        assert!(path.segments[0].generics.is_none());
    }

    #[test]
    fn parses_with_turbofish() {
        let src = "foo::<T, i32>::bar";
        let mut path = parse_path_no_errors(src);
        assert_eq!(path.kind, PathKind::Plain);
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[0].ident.to_string(), "foo");

        let generics = path.segments.remove(0).generics;
        assert_eq!(generics.unwrap().len(), 2);

        let generics = path.segments.remove(0).generics;
        assert!(generics.is_none());
    }

    #[test]
    fn parses_path_stops_before_trailing_double_colon() {
        let src = "foo::bar::";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let path = parser.parse_path_or_error();
        assert_eq!(path.location.span.end() as usize, src.len());
        assert_eq!(parser.errors.len(), 1);
        assert_eq!(path.to_string(), "foo::bar");
    }

    #[test]
    fn parses_path_with_turbofish_stops_before_trailing_double_colon() {
        let src = "foo::bar::<1>::";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let path = parser.parse_path_or_error();
        assert_eq!(path.location.span.end() as usize, src.len());
        assert_eq!(parser.errors.len(), 1);
        assert_eq!(path.to_string(), "foo::bar::<1>");
    }

    #[test]
    fn errors_on_crate_double_colons() {
        let src = "
        crate:: 
               ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str_with_dummy_file(&src);
        let path = parser.parse_path();
        assert!(path.is_none());

        let error = get_single_error(&parser.errors, span);
        assert_snapshot!(error.to_string(), @"Expected an identifier but found end of input");
    }
}
