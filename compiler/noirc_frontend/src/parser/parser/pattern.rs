use noirc_errors::Span;

use crate::{
    ast::{Ident, Path, Pattern},
    parser::{labels::ParsingRuleLabel, ParserErrorReason},
    token::{Keyword, Token, TokenKind},
};

use super::{
    parse_many::{separated_by_comma_until_right_brace, separated_by_comma_until_right_paren},
    Parser,
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

impl<'a> Parser<'a> {
    pub(crate) fn parse_pattern_or_error(&mut self) -> Pattern {
        if let Some(pattern) = self.parse_pattern() {
            return pattern;
        }

        self.expected_label(ParsingRuleLabel::Pattern);
        Pattern::Identifier(Ident::new(String::new(), self.span_at_previous_token_end()))
    }

    /// Pattern
    ///     = 'mut' PatternNoMut
    pub(crate) fn parse_pattern(&mut self) -> Option<Pattern> {
        let start_span = self.current_token_span;
        let mutable = self.eat_keyword(Keyword::Mut);
        self.parse_pattern_after_modifiers(mutable, start_span)
    }

    /// PatternOrSelf
    ///     = Pattern
    ///     | SelfPattern
    pub(crate) fn parse_pattern_or_self(&mut self) -> Option<PatternOrSelf> {
        let start_span = self.current_token_span;

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
                    self.parse_pattern_after_modifiers(true, start_span)?,
                ));
            }
        }

        if self.at(Token::Ampersand) && self.next_is(Token::Keyword(Keyword::Mut)) {
            self.bump();
            self.bump();
            if !self.next_is_colon() && self.eat_self() {
                return Some(PatternOrSelf::SelfPattern(SelfPattern {
                    reference: true,
                    mutable: true,
                }));
            } else {
                self.push_error(
                    ParserErrorReason::RefMutCanOnlyBeUsedWithSelf,
                    self.current_token_span,
                );
                return Some(PatternOrSelf::Pattern(
                    self.parse_pattern_after_modifiers(true, start_span)?,
                ));
            }
        }

        Some(PatternOrSelf::Pattern(self.parse_pattern_after_modifiers(false, start_span)?))
    }

    fn next_is_colon(&self) -> bool {
        self.next_is(Token::Colon)
    }

    pub(crate) fn parse_pattern_after_modifiers(
        &mut self,
        mutable: bool,
        start_span: Span,
    ) -> Option<Pattern> {
        let pattern = self.parse_pattern_no_mut()?;
        Some(if mutable {
            Pattern::Mutable(
                Box::new(pattern),
                self.span_since(start_span),
                false, // is synthesized
            )
        } else {
            pattern
        })
    }

    /// PatternNoMut
    ///     = InternedPattern
    ///     | TuplePattern
    ///     | StructPattern
    ///     | IdentifierPattern
    ///
    /// IdentifierPattern = identifier
    fn parse_pattern_no_mut(&mut self) -> Option<Pattern> {
        let start_span = self.current_token_span;

        if let Some(pattern) = self.parse_interned_pattern() {
            return Some(pattern);
        }

        if let Some(pattern) = self.parse_tuple_pattern() {
            return Some(pattern);
        }

        let Some(mut path) = self.parse_path() else {
            if self.at_built_in_type() {
                self.push_error(
                    ParserErrorReason::ExpectedPatternButFoundType(self.token.token().clone()),
                    self.current_token_span,
                );
            }
            return None;
        };

        if self.eat_left_brace() {
            return Some(self.parse_struct_pattern(path, start_span));
        }

        if !path.is_ident() {
            self.push_error(ParserErrorReason::InvalidPattern, path.span);

            let ident = path.segments.pop().unwrap().ident;
            return Some(Pattern::Identifier(ident));
        }

        let ident = path.segments.remove(0).ident;
        Some(Pattern::Identifier(ident))
    }

    /// InternedPattern = interned_pattern
    fn parse_interned_pattern(&mut self) -> Option<Pattern> {
        let Some(token) = self.eat_kind(TokenKind::InternedPattern) else {
            return None;
        };

        match token.into_token() {
            Token::InternedPattern(pattern) => {
                Some(Pattern::Interned(pattern, self.previous_token_span))
            }
            _ => unreachable!(),
        }
    }

    /// TuplePattern = '(' PatternList? ')'
    ///
    /// PatternList = Pattern ( ',' Pattern )* ','?
    fn parse_tuple_pattern(&mut self) -> Option<Pattern> {
        let start_span = self.current_token_span;

        if !self.eat_left_paren() {
            return None;
        }

        let patterns = self.parse_many(
            "tuple elements",
            separated_by_comma_until_right_paren(),
            Self::parse_tuple_pattern_element,
        );

        Some(Pattern::Tuple(patterns, self.span_since(start_span)))
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
    fn parse_struct_pattern(&mut self, path: Path, start_span: Span) -> Pattern {
        let fields = self.parse_many(
            "struct fields",
            separated_by_comma_until_right_brace(),
            Self::parse_struct_pattern_field,
        );

        Pattern::Struct(path, fields, self.span_since(start_span))
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

    fn at_built_in_type(&self) -> bool {
        matches!(
            self.token.token(),
            Token::Bool(..)
                | Token::IntType(..)
                | Token::Keyword(Keyword::Bool)
                | Token::Keyword(Keyword::CtString)
                | Token::Keyword(Keyword::Expr)
                | Token::Keyword(Keyword::Field)
                | Token::Keyword(Keyword::FunctionDefinition)
                | Token::Keyword(Keyword::Module)
                | Token::Keyword(Keyword::Quoted)
                | Token::Keyword(Keyword::StructDefinition)
                | Token::Keyword(Keyword::TraitConstraint)
                | Token::Keyword(Keyword::TraitDefinition)
                | Token::Keyword(Keyword::TraitImpl)
                | Token::Keyword(Keyword::TypedExpr)
                | Token::Keyword(Keyword::TypeType)
                | Token::Keyword(Keyword::UnresolvedType)
        )
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::Pattern,
        parser::{
            parser::tests::{
                expect_no_errors, get_single_error, get_single_error_reason,
                get_source_with_error_span,
            },
            Parser, ParserErrorReason,
        },
        token::{Keyword, Token},
    };

    fn parse_pattern_no_errors(src: &str) -> Pattern {
        let mut parser = Parser::for_str(src);
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
    fn parses_tuple_pattern() {
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
        let mut parser = Parser::for_str(src);
        let pattern = parser.parse_pattern_or_error();
        assert_eq!(parser.errors.len(), 1);
        let Pattern::Tuple(patterns, _) = pattern else { panic!("Expected a tuple pattern") };
        assert_eq!(patterns.len(), 1);
    }

    #[test]
    fn parses_struct_pattern_no_fields() {
        let src = "foo::Bar {}";
        let mut parser = Parser::for_str(src);
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
        let mut parser = Parser::for_str(&src);
        let pattern = parser.parse_pattern_or_error();

        let error = get_single_error(&parser.errors, span);
        assert_eq!(error.to_string(), "Expected a ':' but found '='");

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
        let mut parser = Parser::for_str(src);
        let pattern = parser.parse_pattern_or_error();
        assert_eq!(parser.errors.len(), 1);
        let Pattern::Struct(path, _, _) = pattern else { panic!("Expected a struct pattern") };
        assert_eq!(path.to_string(), "foo::Bar");
    }

    #[test]
    fn errors_on_reserved_type() {
        let src = "
        Field
        ^^^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        let pattern = parser.parse_pattern();
        assert!(pattern.is_none());

        let reason = get_single_error_reason(&parser.errors, span);
        assert!(matches!(
            reason,
            ParserErrorReason::ExpectedPatternButFoundType(Token::Keyword(Keyword::Field))
        ));
    }
}
