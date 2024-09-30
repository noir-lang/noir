use noirc_errors::Span;

use crate::{
    ast::{Ident, Path, Pattern},
    parser::ParserErrorReason,
    token::{Keyword, Token, TokenKind},
};

use super::Parser;

pub(crate) enum PatternOrSelf {
    Pattern(Pattern),
    SelfPattern(SelfPattern),
}

pub(crate) struct SelfPattern {
    pub(crate) reference: bool,
    pub(crate) mutable: bool,
}

impl<'a> Parser<'a> {
    pub(crate) fn parse_pattern_or_error(&mut self) -> Pattern {
        if let Some(pattern) = self.parse_pattern() {
            return pattern;
        }

        self.push_error(ParserErrorReason::ExpectedPattern, self.current_token_span);
        Pattern::Identifier(Ident::new(String::new(), self.span_at_previous_token_end()))
    }

    pub(crate) fn parse_pattern(&mut self) -> Option<Pattern> {
        let start_span = self.current_token_span;
        let mutable = self.eat_keyword(Keyword::Mut);
        self.parse_pattern_after_modifiers(mutable, start_span)
    }

    pub(crate) fn parse_pattern_or_self(&mut self) -> Option<PatternOrSelf> {
        let start_span = self.current_token_span;

        let reference = self.eat(Token::Ampersand);
        let mutable = self.eat_keyword(Keyword::Mut);

        if self.eat_self() {
            // TODO: error if reference but not mutable
            Some(PatternOrSelf::SelfPattern(SelfPattern { reference, mutable }))
        } else {
            // TODO: error if reference is true
            Some(PatternOrSelf::Pattern(self.parse_pattern_after_modifiers(mutable, start_span)?))
        }
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

    fn parse_pattern_no_mut(&mut self) -> Option<Pattern> {
        if let Some(pattern) = self.parse_interned_pattern() {
            return Some(pattern);
        }

        if let Some(pattern) = self.parse_tuple_pattern() {
            return Some(pattern);
        }

        let Some(mut path) = self.parse_path() else {
            return None;
        };

        if self.eat_left_brace() {
            return Some(self.parse_struct_pattern(path));
        }

        if !path.is_ident() {
            // TODO: error
            let ident = path.segments.pop().unwrap().ident;
            return Some(Pattern::Identifier(ident));
        }

        let ident = path.segments.remove(0).ident;
        Some(Pattern::Identifier(ident))
    }

    fn parse_tuple_pattern(&mut self) -> Option<Pattern> {
        let start_span = self.current_token_span;

        if !self.eat_left_paren() {
            return None;
        }

        let mut patterns = Vec::new();
        let mut trailing_comma = false;
        loop {
            if self.eat_right_paren() {
                break;
            }

            let start_span = self.current_token_span;
            let Some(pattern) = self.parse_pattern() else {
                self.push_error(ParserErrorReason::ExpectedPattern, self.current_token_span);
                self.eat_right_paren();
                break;
            };

            if !trailing_comma && !patterns.is_empty() {
                self.expected_token_separating_items(",", "tuple elements", start_span);
            }

            patterns.push(pattern);

            trailing_comma = self.eat_commas();
        }

        Some(Pattern::Tuple(patterns, self.span_since(start_span)))
    }

    fn parse_struct_pattern(&mut self, path: Path) -> Pattern {
        let start_span = path.span();

        let mut patterns = Vec::new();
        let mut trailing_comma = false;

        loop {
            if self.eat_right_brace() {
                break;
            }

            let start_span = self.current_token_span;

            let Some(ident) = self.eat_ident() else {
                self.expected_identifier();
                self.eat_right_brace();
                break;
            };

            if !trailing_comma && !patterns.is_empty() {
                self.expected_token_separating_items(",", "struct fields", start_span);
            }

            if self.eat_colon() {
                patterns.push((ident, self.parse_pattern_or_error()));
            } else {
                patterns.push((ident.clone(), Pattern::Identifier(ident)));
            }

            trailing_comma = self.eat_commas();
        }

        Pattern::Struct(path, patterns, self.span_since(start_span))
    }

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
}

#[cfg(test)]
mod tests {

    use crate::{ast::Pattern, parser::Parser};

    #[test]
    fn parses_identifier_pattern() {
        let src = "foo";
        let mut parser = Parser::for_str(src);
        let pattern = parser.parse_pattern_or_error();
        assert!(parser.errors.is_empty());
        let Pattern::Identifier(ident) = pattern else { panic!("Expected an identifier pattern") };
        assert_eq!(ident.to_string(), "foo");
    }

    #[test]
    fn parses_mutable_pattern() {
        let src = "mut foo";
        let mut parser = Parser::for_str(src);
        let pattern = parser.parse_pattern_or_error();
        assert!(parser.errors.is_empty());
        let Pattern::Mutable(pattern, _, _) = pattern else { panic!("Expected a mutable pattern") };
        let pattern: &Pattern = &pattern;
        let Pattern::Identifier(ident) = pattern else { panic!("Expected an identifier pattern") };
        assert_eq!(ident.to_string(), "foo");
    }

    #[test]
    fn parses_tuple_pattern() {
        let src = "(foo, bar)";
        let mut parser = Parser::for_str(src);
        let pattern = parser.parse_pattern_or_error();
        assert!(parser.errors.is_empty());
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
        assert!(parser.errors.is_empty());
        let Pattern::Struct(path, patterns, _) = pattern else {
            panic!("Expected a struct pattern")
        };
        assert_eq!(path.to_string(), "foo::Bar");
        assert!(patterns.is_empty());
    }

    #[test]
    fn parses_struct_pattern() {
        let src = "foo::Bar { x: one, y }";
        let mut parser = Parser::for_str(src);
        let pattern = parser.parse_pattern_or_error();
        assert!(parser.errors.is_empty());
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
}
