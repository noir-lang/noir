use crate::{
    ast::{Ident, Path, Pattern},
    token::{Keyword, Token, TokenKind},
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn parse_pattern(&mut self) -> Pattern {
        let start_span = self.current_token_span;

        let mutable = self.eat_keyword(Keyword::Mut);
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

            patterns.push(self.parse_pattern());

            self.eat_commas();
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
        let typ = Parser::for_str(src).parse_pattern();
        let Pattern::Identifier(ident) = typ else { panic!("Expected an identifier pattern") };
        assert_eq!(ident.to_string(), "foo");
    }

    #[test]
    fn parses_mutable_pattern() {
        let src = "mut foo";
        let typ = Parser::for_str(src).parse_pattern();
        let Pattern::Mutable(pattern, _, _) = typ else { panic!("Expected a mutable pattern") };
        let pattern: &Pattern = &pattern;
        let Pattern::Identifier(ident) = pattern else { panic!("Expected an identifier pattern") };
        assert_eq!(ident.to_string(), "foo");
    }

    #[test]
    fn parses_tuple_pattern() {
        let src = "(foo, bar)";
        let typ = Parser::for_str(src).parse_pattern();
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
    fn parses_struct_pattern_no_fields() {
        let src = "foo::Bar {}";
        let typ = Parser::for_str(src).parse_pattern();
        let Pattern::Struct(path, patterns, _) = typ else { panic!("Expected a struct pattern") };
        assert_eq!(path.to_string(), "foo::Bar");
        assert!(patterns.is_empty());
    }

    #[test]
    fn parses_struct_pattern() {
        let src = "foo::Bar { x: one, y }";
        let typ = Parser::for_str(src).parse_pattern();
        let Pattern::Struct(path, mut patterns, _) = typ else {
            panic!("Expected a struct pattern")
        };
        assert_eq!(path.to_string(), "foo::Bar");
    }
}
