use noirc_errors::Location;

use crate::{
    ast::{Ident, Path, Pattern},
    parser::{ParserErrorReason, labels::ParsingRuleLabel},
    token::{Keyword, Token, TokenKind},
};

use super::{
    Parser,
    parse_many::{separated_by_comma_until_right_brace, separated_by_comma_until_right_paren},
};

pub(crate) enum PatternOrSelf {
    Pattern(Pattern),
    SelfPattern(SelfPattern),
}

/// SelfPattern is guaranteed to be `self`, `&self` or `&mut self` without a colon following it.
pub(crate) struct SelfPattern {
    pub(crate) reference: bool,
    pub(crate) mutable: bool,
}

impl Parser<'_> {
    pub(crate) fn parse_pattern_or_error(&mut self) -> Pattern {
        if let Some(pattern) = self.parse_pattern() {
            return pattern;
        }

        self.expected_label(ParsingRuleLabel::Pattern);
        Pattern::Identifier(Ident::new(String::new(), self.location_at_previous_token_end()))
    }

    /// Pattern
    ///     = 'mut' PatternNoMut
    pub(crate) fn parse_pattern(&mut self) -> Option<Pattern> {
        let start_location = self.current_token_location;
        let mutable = self.eat_keyword(Keyword::Mut);
        self.parse_pattern_after_modifiers(mutable, start_location)
    }

    /// PatternOrSelf
    ///     = Pattern
    ///     | SelfPattern
    pub(crate) fn parse_pattern_or_self(&mut self) -> Option<PatternOrSelf> {
        let start_location = self.current_token_location;

        if !self.next_is_colon() && self.eat_self() {
            return Some(PatternOrSelf::SelfPattern(SelfPattern {
                reference: false,
                mutable: false,
            }));
        }

        if self.eat_keyword(Keyword::Mut) {
            if !self.next_is_colon() && self.eat_self() {
                return Some(PatternOrSelf::SelfPattern(SelfPattern {
                    reference: false,
                    mutable: true,
                }));
            } else {
                return Some(PatternOrSelf::Pattern(
                    self.parse_pattern_after_modifiers(true, start_location)?,
                ));
            }
        }

        if self.at(Token::Ampersand) {
            self.bump();

            let mutable = self.eat_keyword(Keyword::Mut);
            if !self.next_is_colon() && self.eat_self() {
                return Some(PatternOrSelf::SelfPattern(SelfPattern { reference: true, mutable }));
            } else {
                self.push_error(
                    ParserErrorReason::RefMutCanOnlyBeUsedWithSelf,
                    self.current_token_location,
                );
                return Some(PatternOrSelf::Pattern(
                    self.parse_pattern_after_modifiers(true, start_location)?,
                ));
            }
        }

        Some(PatternOrSelf::Pattern(self.parse_pattern_after_modifiers(false, start_location)?))
    }

    fn next_is_colon(&self) -> bool {
        self.next_is(Token::Colon)
    }

    pub(crate) fn parse_pattern_after_modifiers(
        &mut self,
        mutable: bool,
        start_location: Location,
    ) -> Option<Pattern> {
        let pattern = self.parse_pattern_no_mut()?;
        Some(if mutable {
            Pattern::Mutable(
                Box::new(pattern),
                self.location_since(start_location),
                false, // is synthesized
            )
        } else {
            pattern
        })
    }

    /// PatternNoMut
    ///     = InternedPattern
    ///     | ParenthesizedPattern
    ///     | TuplePattern
    ///     | StructPattern
    ///     | IdentifierPattern
    ///
    /// IdentifierPattern = identifier
    fn parse_pattern_no_mut(&mut self) -> Option<Pattern> {
        let start_location = self.current_token_location;

        if let Some(pattern) = self.parse_interned_pattern() {
            return Some(pattern);
        }

        if let Some(pattern) = self.parse_parenthesized_or_tuple_pattern() {
            return Some(pattern);
        }

        let mut path = self.parse_path()?;

        if self.eat_left_brace() {
            return Some(self.parse_struct_pattern(path, start_location));
        }

        if !path.is_ident() {
            self.push_error(ParserErrorReason::InvalidPattern, path.location);

            let ident = path.segments.pop().unwrap().ident;
            return Some(Pattern::Identifier(ident));
        }

        let ident = path.segments.remove(0).ident;
        Some(Pattern::Identifier(ident))
    }

    /// InternedPattern = interned_pattern
    fn parse_interned_pattern(&mut self) -> Option<Pattern> {
        let token = self.eat_kind(TokenKind::InternedPattern)?;

        match token.into_token() {
            Token::InternedPattern(pattern) => {
                Some(Pattern::Interned(pattern, self.previous_token_location))
            }
            _ => unreachable!(),
        }
    }

    /// ParenthesizedPattern = '(' Pattern ')'
    /// TuplePattern = '(' PatternList? ')'
    ///
    /// PatternList = Pattern ( ',' Pattern )* ','?
    fn parse_parenthesized_or_tuple_pattern(&mut self) -> Option<Pattern> {
        let start_location = self.current_token_location;

        if !self.eat_left_paren() {
            return None;
        }

        let (mut patterns, has_trailing_comma) = self.parse_many_return_trailing_separator_if_any(
            "tuple elements",
            separated_by_comma_until_right_paren(),
            Self::parse_tuple_pattern_element,
        );

        let location = self.location_since(start_location);

        Some(if patterns.len() == 1 && !has_trailing_comma {
            Pattern::Parenthesized(Box::new(patterns.remove(0)), location)
        } else {
            Pattern::Tuple(patterns, location)
        })
    }

    fn parse_tuple_pattern_element(&mut self) -> Option<Pattern> {
        if let Some(pattern) = self.parse_pattern() {
            Some(pattern)
        } else {
            self.expected_label(ParsingRuleLabel::Pattern);
            None
        }
    }

    /// StructPattern = Path '{' StructPatternFields? '}'
    ///
    /// StructPatternFields = StructPatternField ( ',' StructPatternField )? ','?
    ///
    /// StructPatternField = identifier ( ':' Pattern )?
    fn parse_struct_pattern(&mut self, path: Path, start_location: Location) -> Pattern {
        let fields = self.parse_many(
            "struct fields",
            separated_by_comma_until_right_brace(),
            Self::parse_struct_pattern_field,
        );

        Pattern::Struct(path, fields, self.location_since(start_location))
    }

    fn parse_struct_pattern_field(&mut self) -> Option<(Ident, Pattern)> {
        let Some(ident) = self.eat_ident() else {
            self.expected_identifier();
            return None;
        };

        Some(if self.eat_colon() {
            (ident, self.parse_pattern_or_error())
        } else if self.at(Token::Assign) {
            // If we find '=' instead of ':', assume the user meant ':`, error and continue
            self.expected_token(Token::Colon);
            self.bump();
            (ident, self.parse_pattern_or_error())
        } else {
            (ident.clone(), Pattern::Identifier(ident))
        })
    }
}

#[cfg(test)]
mod tests {

    use insta::assert_snapshot;

    use crate::{
        ast::Pattern,
        parser::{
            Parser,
            parser::tests::{expect_no_errors, get_single_error, get_source_with_error_span},
        },
    };

    fn parse_pattern_no_errors(src: &str) -> Pattern {
        let mut parser = Parser::for_str_with_dummy_file(src);
        let pattern = parser.parse_pattern_or_error();
        expect_no_errors(&parser.errors);
        pattern
    }

    #[test]
    fn parses_identifier_pattern() {
        let src = "foo";
        let pattern = parse_pattern_no_errors(src);
        let Pattern::Identifier(ident) = pattern else { panic!("Expected an identifier pattern") };
        assert_eq!(ident.to_string(), "foo");
    }

    #[test]
    fn parses_mutable_pattern() {
        let src = "mut foo";
        let pattern = parse_pattern_no_errors(src);
        let Pattern::Mutable(pattern, _, _) = pattern else { panic!("Expected a mutable pattern") };
        let pattern: &Pattern = &pattern;
        let Pattern::Identifier(ident) = pattern else { panic!("Expected an identifier pattern") };
        assert_eq!(ident.to_string(), "foo");
    }

    #[test]
    fn parses_parenthesized_pattern() {
        let src = "(foo)";
        let pattern = parse_pattern_no_errors(src);
        let Pattern::Parenthesized(pattern, _) = pattern else {
            panic!("Expected a tuple pattern")
        };

        let Pattern::Identifier(ident) = *pattern else { panic!("Expected an identifier pattern") };
        assert_eq!(ident.to_string(), "foo");
    }

    #[test]
    fn parses_tuple_pattern_one_element() {
        let src = "(foo,)";
        let pattern = parse_pattern_no_errors(src);
        let Pattern::Tuple(mut patterns, _) = pattern else { panic!("Expected a tuple pattern") };
        assert_eq!(patterns.len(), 1);

        let pattern = patterns.remove(0);
        let Pattern::Identifier(ident) = pattern else { panic!("Expected an identifier pattern") };
        assert_eq!(ident.to_string(), "foo");
    }

    #[test]
    fn parses_tuple_pattern_two_elements() {
        let src = "(foo, bar)";
        let pattern = parse_pattern_no_errors(src);
        let Pattern::Tuple(mut patterns, _) = pattern else { panic!("Expected a tuple pattern") };
        assert_eq!(patterns.len(), 2);

        let pattern = patterns.remove(0);
        let Pattern::Identifier(ident) = pattern else { panic!("Expected an identifier pattern") };
        assert_eq!(ident.to_string(), "foo");

        let pattern = patterns.remove(0);
        let Pattern::Identifier(ident) = pattern else { panic!("Expected an identifier pattern") };
        assert_eq!(ident.to_string(), "bar");
    }

    #[test]
    fn parses_unclosed_tuple_pattern() {
        let src = "(foo,";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let pattern = parser.parse_pattern_or_error();
        assert_eq!(parser.errors.len(), 1);
        let Pattern::Tuple(patterns, _) = pattern else { panic!("Expected a tuple pattern") };
        assert_eq!(patterns.len(), 1);
    }

    #[test]
    fn parses_struct_pattern_no_fields() {
        let src = "foo::Bar {}";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let pattern = parser.parse_pattern_or_error();
        expect_no_errors(&parser.errors);
        let Pattern::Struct(path, patterns, _) = pattern else {
            panic!("Expected a struct pattern")
        };
        assert_eq!(path.to_string(), "foo::Bar");
        assert!(patterns.is_empty());
    }

    #[test]
    fn parses_struct_pattern() {
        let src = "foo::Bar { x: one, y }";
        let pattern = parse_pattern_no_errors(src);
        let Pattern::Struct(path, mut patterns, _) = pattern else {
            panic!("Expected a struct pattern")
        };
        assert_eq!(path.to_string(), "foo::Bar");
        assert_eq!(patterns.len(), 2);

        let (ident, pattern) = patterns.remove(0);
        assert_eq!(ident.to_string(), "x");
        assert_eq!(pattern.to_string(), "one");

        let (ident, pattern) = patterns.remove(0);
        assert_eq!(ident.to_string(), "y");
        assert_eq!(pattern.to_string(), "y");
    }

    #[test]
    fn parses_struct_pattern_recovers_if_assign_instead_of_colon() {
        let src = "
        foo::Bar { x = one, y }
                     ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str_with_dummy_file(&src);
        let pattern = parser.parse_pattern_or_error();

        let error = get_single_error(&parser.errors, span);
        assert_snapshot!(error.to_string(), @"Expected a ':' but found '='");

        let Pattern::Struct(path, mut patterns, _) = pattern else {
            panic!("Expected a struct pattern")
        };
        assert_eq!(path.to_string(), "foo::Bar");
        assert_eq!(patterns.len(), 2);

        let (ident, pattern) = patterns.remove(0);
        assert_eq!(ident.to_string(), "x");
        assert_eq!(pattern.to_string(), "one");

        let (ident, pattern) = patterns.remove(0);
        assert_eq!(ident.to_string(), "y");
        assert_eq!(pattern.to_string(), "y");
    }

    #[test]
    fn parses_unclosed_struct_pattern() {
        let src = "foo::Bar { x";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let pattern = parser.parse_pattern_or_error();
        assert_eq!(parser.errors.len(), 1);
        let Pattern::Struct(path, _, _) = pattern else { panic!("Expected a struct pattern") };
        assert_eq!(path.to_string(), "foo::Bar");
    }
}
