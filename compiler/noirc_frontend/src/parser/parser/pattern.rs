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
    pub(crate) fn parse_pattern(&mut self) -> Pattern {
        let start_span = self.current_token_span;
        let mutable = self.eat_keyword(Keyword::Mut);
        self.parse_pattern_after_modifiers(mutable, start_span)
    }

    pub(crate) fn parse_pattern_or_self(&mut self) -> PatternOrSelf {
        let start_span = self.current_token_span;

        let reference = self.eat(Token::Ampersand);
        let mutable = self.eat_keyword(Keyword::Mut);

        if self.eat_self() {
            // TODO: error if reference but not mutable
            PatternOrSelf::SelfPattern(SelfPattern { reference, mutable })
        } else {
            // TODO: error if reference is true
            PatternOrSelf::Pattern(self.parse_pattern_after_modifiers(mutable, start_span))
        }
    }

    pub(crate) fn parse_pattern_after_modifiers(
        &mut self,
        mutable: bool,
        start_span: Span,
    ) -> Pattern {
        let pattern = self.parse_pattern_no_mut();
        if mutable {
            Pattern::Mutable(
                Box::new(pattern),
                self.span_since(start_span),
                false, // is synthesized
            )
        } else {
            pattern
        }
    }

    fn parse_pattern_no_mut(&mut self) -> Pattern {
        if let Some(pattern) = self.parse_interned_pattern() {
            return pattern;
        }

        if let Some(pattern) = self.parse_tuple_pattern() {
            return pattern;
        }

        let mut path = self.parse_path();
        if path.is_empty() {
            self.push_error(ParserErrorReason::ExpectedPattern, self.current_token_span);

            // TODO: error
            return Pattern::Identifier(Ident::default());
        }

        if self.eat_left_brace() {
            return self.parse_struct_pattern(path);
        }

        if !path.is_ident() {
            // TODO: error
            let ident = path.segments.pop().unwrap().ident;
            return Pattern::Identifier(ident);
        }

        let ident = path.segments.remove(0).ident;
        Pattern::Identifier(ident)
    }

    fn parse_tuple_pattern(&mut self) -> Option<Pattern> {
        let start_span = self.current_token_span;

        if !self.eat_left_paren() {
            return None;
        }

        let mut patterns = Vec::new();
        loop {
            if self.eat_right_paren() {
                break;
            }

            let start_span = self.current_token_span;
            let pattern = self.parse_pattern();
            if self.current_token_span == start_span {
                // TODO: error
                self.eat_right_paren();
                break;
            }

            patterns.push(pattern);

            self.eat_commas();
            // TODO: error if no commas between patterns
        }

        Some(Pattern::Tuple(patterns, self.span_since(start_span)))
    }

    fn parse_struct_pattern(&mut self, path: Path) -> Pattern {
        let start_span = path.span();

        let mut patterns = Vec::new();

        loop {
            if self.eat_right_brace() {
                break;
            }

            let Some(ident) = self.eat_ident() else {
                // TODO: error
                break;
            };

            if self.eat_colon() {
                patterns.push((ident, self.parse_pattern()));
            } else {
                patterns.push((ident.clone(), Pattern::Identifier(ident)));
            }

            self.eat_commas();
            // TODO: error if no comma between patterns
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
        let typ = parser.parse_pattern();
        assert!(parser.errors.is_empty());
        let Pattern::Identifier(ident) = typ else { panic!("Expected an identifier pattern") };
        assert_eq!(ident.to_string(), "foo");
    }

    #[test]
    fn parses_mutable_pattern() {
        let src = "mut foo";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_pattern();
        assert!(parser.errors.is_empty());
        let Pattern::Mutable(pattern, _, _) = typ else { panic!("Expected a mutable pattern") };
        let pattern: &Pattern = &pattern;
        let Pattern::Identifier(ident) = pattern else { panic!("Expected an identifier pattern") };
        assert_eq!(ident.to_string(), "foo");
    }

    #[test]
    fn parses_tuple_pattern() {
        let src = "(foo, bar)";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_pattern();
        assert!(parser.errors.is_empty());
        let Pattern::Tuple(mut patterns, _) = typ else { panic!("Expected a tuple pattern") };
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
        let typ = parser.parse_pattern();
        assert_eq!(parser.errors.len(), 1);
        let Pattern::Tuple(patterns, _) = typ else { panic!("Expected a tuple pattern") };
        assert_eq!(patterns.len(), 1);
    }

    #[test]
    fn parses_struct_pattern_no_fields() {
        let src = "foo::Bar {}";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_pattern();
        assert!(parser.errors.is_empty());
        let Pattern::Struct(path, patterns, _) = typ else { panic!("Expected a struct pattern") };
        assert_eq!(path.to_string(), "foo::Bar");
        assert!(patterns.is_empty());
    }

    #[test]
    fn parses_struct_pattern() {
        let src = "foo::Bar { x: one, y }";
        let mut parser = Parser::for_str(src);
        let typ = parser.parse_pattern();
        assert!(parser.errors.is_empty());
        let Pattern::Struct(path, mut patterns, _) = typ else {
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
        let typ = parser.parse_pattern();
        assert!(parser.errors.is_empty());
        let Pattern::Struct(path, _, _) = typ else { panic!("Expected a struct pattern") };
        assert_eq!(path.to_string(), "foo::Bar");
    }
}
